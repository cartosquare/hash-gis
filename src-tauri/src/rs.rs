use sr::{AlgorithmType, SenseRemote};
use std::{fs, path::Path};
use tauri::Window;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PredictStatus {
    stage: String,
    progress: f64,
    fail: bool,
    params: Option<PredictParams>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictParams {
    algorithm_type: String,
    model_path: String,
    datasources: Vec<String>,
    output_path: String,
    options: Vec<String>,
}

#[tauri::command(rename_all = "snake_case")]
pub fn predict(window: Window, params: PredictParams) {
    println!("params: {:?}", params);

    let w = window.clone();
    let p = params.clone();

    std::thread::spawn(move || {
        let at = match params.algorithm_type.as_str() {
            "seg-post" => AlgorithmType::SEG_POST,
            "building-post" => AlgorithmType::BUILDING_POST,
            "road-post" => AlgorithmType::ROAD_POST,
            _ => AlgorithmType::SKIP,
        };

        let status = SenseRemote::execute(
            // AlgorithmType::SEG_POST,
            at,
            // String::from("D:\\atlas\\model\\sense-layers\\agri\\corn_rgbnir8bit_2m_221223.m"),
            params.model_path,
            // vec![String::from("D:\\windows-common-libs-v4.1.x\\4bands.tif")],
            params.datasources,
            // vec![
            //     String::from("license_server=10.112.60.244:8181"),
            //     String::from("verbose=debug"),
            // ],
            params.options,
            move |progress, stage: String| {
                window
                    .emit(
                        "predict-status",
                        PredictStatus {
                            progress,
                            stage,
                            fail: false,
                            params: None,
                        },
                    )
                    .unwrap();
            },
            // String::from("D:\\windows-common-libs-v4.1.x\\4bands-testoutput.shp"),
            params.output_path,
            None,
        );
        if status.is_err() {
            w.emit(
                "predict-status",
                PredictStatus {
                    progress: -1.0,
                    stage: "finish".into(),
                    fail: true,
                    params: Some(p),
                },
            )
        } else {
            w.emit(
                "predict-status",
                PredictStatus {
                    progress: 1.0,
                    stage: "finish".into(),
                    fail: false,
                    params: Some(p),
                },
            )
        }
    });
}

#[tauri::command]
pub fn get_cuda_info() -> Result<Vec<String>, String> {
    let ret = SenseRemote::cuda_info();
    if ret.is_err() {
        Err("get cuda info fail".into())
    } else {
        Ok(ret.unwrap())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelOption {
    pub input_type: String,
    pub name: String,
    pub label: String,
    pub choices: Option<Vec<String>>,
    pub style: Option<Vec<String>>,
    pub min: Option<u32>,
    pub max: Option<u32>,
    pub value: Option<u32>,
    pub scale: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Model {
    pub name: String,
    pub icon: String,
    pub model_path: String,
    pub input_files: u32,
    pub input_bands: u32,
    pub input_range: Vec<f64>,
    pub post_type: String,
    pub description: String,
    pub tags: Vec<String>,
    pub options: Vec<ModelOption>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub license_server: String,
    pub models: Vec<Model>,
}

//#[tauri::command(rename_all = "snake_case")]
#[tauri::command]
pub fn app_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    let config_path = app_handle
        .path_resolver()
        .resolve_resource("app_config.toml");
    if config_path.is_none() {
        return Err(String::from("resolve app_config.toml resource fail!"));
    }

    let contents = fs::read_to_string(&config_path.unwrap());
    if contents.is_err() {
        return Err(String::from("read app_config.toml fail!"));
    }

    let data = toml::from_str::<AppConfig>(&contents.unwrap());
    if data.is_err() {
        return Err(String::from("parse app_config.toml fail!"));
    }

    let exe_path = std::env::current_exe().unwrap();
    let appdir = exe_path.parent().unwrap();

    let mut config = data.unwrap();
    for model in config.models.iter_mut() {
        model.icon = appdir.join(model.icon.clone()).to_str().unwrap().into();
        model.model_path = appdir.join(model.model_path.clone()).to_str().unwrap().into();
    }

    // println!("model config: {:?}", config);
    Ok(config)
}

#[tauri::command]
pub fn app_dir() -> String {
    let exe_path = std::env::current_exe().unwrap();
    let appdir = exe_path.parent().unwrap();
    appdir.to_str().unwrap().into()
}
