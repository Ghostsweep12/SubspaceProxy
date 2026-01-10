use std::time::Duration;
use tauri::Manager;

mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Vue to Rust FFI here
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::save_profile,
            commands::setup_namespace,
            commands::run,
            commands::cleanup
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
