use tauri::Manager;
use std::time::Duration;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())

        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                tauri::async_runtime::spawn(async move {
                    // Wait for the GPU
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    // Reload
                    let _ = window.eval("window.location.reload()");
                });
            }
            Ok(())
        })

        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
