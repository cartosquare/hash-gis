#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!("../bindings.rs");

use crate::errors::MapnikError;
use std::ffi::CStr;
use std::ffi::CString;

pub struct MapnikMap {
    pub map: *mut mapnik_map_t,
}

impl MapnikMap {
    fn check_error(&self) -> Result<(), MapnikError> {
        unsafe {
            let err = mapnik_map_last_error(self.map);
            if !err.is_null() {
                let c_str: &CStr = CStr::from_ptr(err);
                let str_slice: &str = c_str.to_str().unwrap();
                let str_buf: String = str_slice.to_owned();
                return Err(MapnikError::Msg(str_buf));
            }
            Ok(())
        }
    }

    pub fn new(width: u32, height: u32, style: String) -> Result<Self, MapnikError> {
        unsafe {
            let map = mapnik_map(width, height);
            let err = mapnik_map_last_error(map);
            if !err.is_null() {
                println!("create map fail: {:?}", err);
                return Err(MapnikError::Msg(String::from("Create map fail")));
            } else {
                println!("create map success");
            }

            let xml = CString::new(style).unwrap();
            mapnik_map_load(map, xml.as_ptr());
            let err = mapnik_map_last_error(map);
            if !err.is_null() {
                println!("load map fail: {:?}", err);
                return Err(MapnikError::Msg(String::from("Load map fail")));
            } else {
                println!("load map success");
            }

            Ok(Self { map })
        }
    }

    pub fn read_extent(
        &self,
        minx: f64,
        miny: f64,
        maxx: f64,
        maxy: f64,
    ) -> Result<Vec<u8>, MapnikError> {
        unsafe {
            let bbox = mapnik_bbox(minx, miny, maxx, maxy);
            mapnik_map_zoom_to_box(self.map, bbox);
            self.check_error()?;

            let image = mapnik_map_render_to_image(self.map);
            self.check_error()?;

            let blob = mapnik_image_to_png_blob(image);
            self.check_error()?;

            let data_slice = std::slice::from_raw_parts(
                (*blob).ptr as *const u8,
                (*blob).len.try_into().unwrap(),
            );
            let mut data = Vec::<u8>::with_capacity((*blob).len.try_into().unwrap());

            data.extend_from_slice(data_slice);

            mapnik_image_blob_free(blob);
            mapnik_bbox_free(bbox);
            self.check_error()?;

            Ok(data)
        }
    }

    pub fn free(&mut self) -> Result<(), MapnikError> {
        unsafe {
            mapnik_map_free(self.map);
        }
        self.check_error()?;

        Ok(())
    }

    pub fn mapnik_register(plugin_dir: String, font_dir: String) {
        unsafe {
            let input_plugin = CString::new(plugin_dir).unwrap();
            let mut s: *mut std::os::raw::c_char = std::ptr::null_mut();
            mapnik_register_datasources(input_plugin.as_ptr(), &mut s);

            println!("loading input plugins: {:?}", s);

            let fonts = CString::new(font_dir).unwrap();
            mapnik_register_fonts(fonts.as_ptr(), &mut s);
            println!("loading fonts: {:?}", s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        unsafe {
            println!("mapnik_bbox: {:?}", mapnik_bbox(0.0, 0.0, 90.0, 90.5));
            assert_eq!(1, 1);
        }
    }

    #[test]
    fn create_map() {
        unsafe {
            let input_plugin =
                CString::new("D:\\Mirror\\vcpkg\\installed\\x64-windows\\bin\\mapnik\\input")
                    .unwrap();
            let mut s: *mut std::os::raw::c_char = std::ptr::null_mut();
            mapnik_register_datasources(input_plugin.as_ptr(), &mut s);

            println!("loading input plugins: {:?}", s);

            let fonts =
                CString::new("D:\\Mirror\\vcpkg\\installed\\x64-windows\\share\\mapnik\\fonts")
                    .unwrap();
            mapnik_register_fonts(fonts.as_ptr(), &mut s);
            println!("loading fonts: {:?}", s);

            let map = mapnik_map(512, 512);
            let err = mapnik_map_last_error(map);
            if !err.is_null() {
                println!("create map fail: {:?}", err);
            } else {
                println!("create map success");
            }

            let xml = CString::new("./styles/test.xml").unwrap();
            mapnik_map_load(map, xml.as_ptr());
            let err = mapnik_map_last_error(map);
            if !err.is_null() {
                println!("load map fail: {:?}", err);
            } else {
                println!("load map success");
            }

            mapnik_map_zoom_all(map);

            let output = CString::new("./output.png").unwrap();
            mapnik_map_render_to_file(map, output.as_ptr());
            println!("render file: {:?}", mapnik_map_last_error(map));

            mapnik_map_free(map);
        }
    }
}
