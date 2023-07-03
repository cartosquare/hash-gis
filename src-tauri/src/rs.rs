#[tauri::command]
pub fn hello_world() {
    println!("command hello world!");
}


#[tauri::command]
pub async fn predict(params: String) -> Result<(), String> {
    println!("predict {}", params);

    Ok(())
}