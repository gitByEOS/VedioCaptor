mod commands;
mod lua_runtime;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_presets,
            commands::get_controls,
            commands::validate,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
