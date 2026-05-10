mod commands;
mod commands_presets;
mod execute_commands;
mod ffmpeg_commands;
mod ffmpeg_runner;
mod lua_runtime;
mod pipeline_executor;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_presets,
            commands::get_controls,
            commands::validate,
            commands_presets::list_presets,
            commands_presets::get_preset_controls,
            ffmpeg_commands::run_ffmpeg,
            ffmpeg_commands::run_pipeline,
            execute_commands::execute_conversion,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
