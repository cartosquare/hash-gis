// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use futures::executor::block_on;
use map_engine_server::app::run;
use std::thread;

mod rs;
use rs::{app_config, app_dir, get_cuda_info, predict};

fn main() {
    thread::spawn(move || {
        let exe_path = std::env::current_exe().unwrap();
        let appdir = exe_path.parent().unwrap();
        println!("app dir: {:?}", appdir);

        // TODO: deal with return
        let _ = block_on(run(
            "".into(),
            "localhost".into(),
            "28904".into(),
            format!("{}/mapnik/input", appdir.display()),
            format!("{}/mapnik/fonts", appdir.display()),
        ));
    });

    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            app_config,
            predict,
            get_cuda_info,
            app_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
