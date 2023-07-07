// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use map_engine_server::app::run;
use futures::executor::block_on;
use std::thread;

mod rs;
use rs::{app_config, predict};

fn main() {
    // TODO: hard-code => env
    thread::spawn(move || {
        block_on(run(
            "".into(),
            "localhost".into(),
            "8080".into(),
            "D:\\Mirror\\vcpkg\\installed\\x64-windows\\bin\\mapnik\\input".into(),
            "D:\\Mirror\\vcpkg\\installed\\x64-windows\\share\\mapnik\\fonts".into(),
        ))
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_config, predict])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
