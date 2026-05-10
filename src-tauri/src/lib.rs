mod commands;
mod ffmpeg_commands;
mod ffmpeg_runner;
mod lua_runtime;
mod pipeline_executor;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_presets,
            commands::get_controls,
            commands::validate,
            ffmpeg_commands::run_ffmpeg,
            ffmpeg_commands::run_pipeline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
