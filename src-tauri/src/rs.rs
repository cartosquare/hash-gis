use sr::{AlgorithmType, SenseRemote};
use tauri::Window;

#[derive(Clone, serde::Serialize)]
struct PredictStatus {
    stage: String,
    progress: f64,
    fail: bool,
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
                    .emit("predict-status", PredictStatus { progress, stage, fail: false })
                    .unwrap();
            },
            // String::from("D:\\windows-common-libs-v4.1.x\\4bands-testoutput.shp"),
            params.output_path,
            None,
        );
        if status.is_err() {
            w.emit("predict-status", PredictStatus{
                progress: -1.0,
                stage: "结束".into(),
                fail: true,
            })
        } else {
            w.emit("predict-status", PredictStatus{
                progress: 1.0,
                stage: "结束".into(),
                fail: false,
            })
        }
    });
}
