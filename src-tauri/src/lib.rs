use tauri::Manager;
use std::time::Duration;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Vue to Rust FFI here
        .invoke_handler(tauri::generate_handler![
            commands::configure_environment,
            commands::ping,
            commands::save_profile
        ])

        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let _ = window.eval("window.location.reload()");
                });
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}