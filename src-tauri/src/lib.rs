mod commands;
mod commands_presets;
mod execute_commands;
mod ffmpeg_commands;
mod ffmpeg_runner;
mod lua_runtime;
mod pipeline_executor;
mod types;

use tauri::{Emitter, Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.clone().on_window_event(move |event| {
                if let WindowEvent::DragDrop(drag_event) = event {
                    if let tauri::DragDropEvent::Drop { paths, .. } = drag_event {
                        if let Some(path) = paths.first() {
                            let p = path.to_string_lossy().to_string();
                            let _ = window.emit("file-dropped", p);
                        }
                    }
                }
            });
            Ok(())
        })
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
