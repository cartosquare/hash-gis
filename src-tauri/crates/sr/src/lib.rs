#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("../bindings.rs");

pub mod errors;

use crate::errors::SenseRemoteError;
use libffi::high::ClosureMut3;
use std::ffi::{c_void, c_char, CStr, CString};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum AlgorithmType {
    SEG_POST,
    BUILDING_POST,
    ROAD_POST,
    SKIP,
}

pub struct SenseRemote {}

impl SenseRemote {
    pub fn execute<F: FnMut(f64, String) + Send + Sync + 'static>(
    // pub fn execute<F: FnMut(f64, String) + 'static>(
        alg_type: AlgorithmType,
        model_path: String,
        data_sources: Vec<String>,
        options: Vec<String>,
        mut callback: F,
        output_path: String,
        log_path: Option<String>,
    ) -> Result<(), SenseRemoteError> {
        unsafe {
            callback(0.0, "loading-model".into());

            // process options
            let mut options_arr: Vec<_> = options
                .iter()
                .map(|x| CString::new(x.clone()).unwrap().into_raw())
                .collect();
            options_arr.push(std::ptr::null_mut());

            // create handle
            
            let log_str = match log_path {
                Some(p) => CString::new(p).unwrap().as_ptr(),
                None => std::ptr::null(),
            };
            let handle = sr_init(0, options_arr.as_mut_ptr(), log_str);
            if handle == std::ptr::null_mut() {
                return Err(SenseRemoteError::Msg(String::from(
                    "SenseRemote create handle fail",
                )));
            }

            // post handle
            let post_handle = sr_post_init(log_str);
            if post_handle == std::ptr::null_mut() {
                return Err(SenseRemoteError::Msg(String::from(
                    "SenseRemote create post handle fail",
                )));
            }

            // load model
            let model_path_str = CString::new(model_path).unwrap();
            let status = sr_load_model(handle, model_path_str.as_ptr());
            if status != 0 {
                return Err(SenseRemoteError::Msg(String::from(
                    "SenseRemote load model fail",
                )));
            }
            callback(1.0, "loading-model".into());

            // add datasource
            for data_source in data_sources {
                let ds_str = CString::new(data_source).unwrap();
                let status = sr_add_datasource(
                    handle,
                    ds_str.as_ptr(),
                    std::ptr::null_mut(),
                    0,
                    -1,
                    ds_str.as_ptr(),
                );

                if status != 0 {
                    return Err(SenseRemoteError::Msg(String::from(
                        "SenseRemote add datasource fail",
                    )));
                }
            }

            // infence
            // callback
            let closure: &'static mut _ = Box::leak(Box::new(
                move |progress: f64,
                      msg: *const ::std::os::raw::c_char,
                      _userdata: *mut ::std::os::raw::c_void|
                      -> ::std::os::raw::c_int {
                    // println!("progress: {}", progress);
                    callback(progress, CStr::from_ptr(msg).to_str().unwrap().to_owned());
                    1
                },
            ));

            let callback = ClosureMut3::new(closure);
            let code = callback.code_ptr();
            let ptr: &_ = &*(code as *const libffi::high::FnPtr3<f64, *const i8, *mut c_void, i32>)
                .cast::<unsafe extern "C" fn(f64, *const i8, *mut c_void) -> i32>();

            std::mem::forget(callback);

            // tmp output
            let probility_output = Path::with_extension(&PathBuf::from(output_path.clone()), "vrt");
            let probility_output_str =
                CString::new(probility_output.into_os_string().into_string().unwrap()).unwrap();
            let output_str = CString::new(output_path).unwrap();

            let infence_output_str = match alg_type.clone() {
                AlgorithmType::SKIP => probility_output_str.clone(),
                _ => output_str.clone(),
            };
            sr_infence(
                handle,
                infence_output_str.as_ptr(),
                Some(*ptr),
                std::ptr::null_mut(),
            );

            if status != 0 {
                return Err(SenseRemoteError::Msg(String::from(
                    "SenseRemote infence fail",
                )));
            }

            // post process
            if alg_type == AlgorithmType::SKIP {
                return Ok(());
            }

            // segmentation alg post-processing
            let post_processing_method = match alg_type {
                AlgorithmType::SEG_POST => String::from("seg-post"),
                AlgorithmType::BUILDING_POST => String::from("building-post"),
                AlgorithmType::ROAD_POST => String::from("road-post"),
                _ => String::from(""),
            };
            let post_processing_str = CString::new(post_processing_method).unwrap();
            let status = sr_post_processing(
                post_handle,
                probility_output_str.as_ptr(),
                post_processing_str.as_ptr(),
                output_str.as_ptr(),
                options_arr.as_mut_ptr(),
                Some(*ptr),
                std::ptr::null_mut(),
            );

            if status != 0 {
                return Err(SenseRemoteError::Msg(String::from(
                    "SenseRemote postprocessing fail",
                )));
            }

            // clean
            sr_destroy(handle);
            sr_post_destroy(post_handle);

            Ok(())
        }
    }

    pub fn cuda_info() -> Result<Vec<String>, SenseRemoteError> {
        unsafe {
            let mut cuda_names: *mut *const c_char = std::ptr::null_mut();
            let mut cuda_sizes: *mut usize = &mut 0;
            let len = get_cuda_information(&mut cuda_names, &mut cuda_sizes);
            if len < 0 {
                Err(SenseRemoteError::Msg("get cuda info fail".into()))
            } else {
                let mut names: Vec<String> = vec![];
                for i in 0..len {
                    names.push(CStr::from_ptr(*cuda_names.offset(i as isize)).to_str().unwrap().to_owned());
                }
                Ok(names)
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sr() {

        SenseRemote::execute(
            AlgorithmType::SEG_POST,
            String::from("D:\\atlas\\model\\sense-layers\\agri\\corn_rgbnir8bit_2m_221223.m"),
            vec![String::from("D:\\windows-common-libs-v4.1.x\\4bands.tif")],
            vec![
                String::from("license_server=10.112.60.244:8181"),
                String::from("verbose=debug"),
            ],
            |progress, msg| println!("progress: {}", progress),
            String::from("D:\\windows-common-libs-v4.1.x\\4bands-testoutput.shp"),
            None,
        ).unwrap()
    }

    #[test]
    fn test_cuda_info() {
        println!("{:?}", SenseRemote::cuda_info().unwrap());
    }

}
