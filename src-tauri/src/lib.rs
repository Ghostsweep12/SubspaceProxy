mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Disable the DMABUF renderer (fixes Linux rendering)
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    tauri::Builder::default()
        // Vue to Rust FFI here
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::get_active_namespaces,
            commands::setup_namespace,
            commands::run,
            commands::cleanup,
            commands::save_profile,
            commands::delete_profile,
            commands::fetch_profile,
            commands::ping,
            commands::port,
            
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
