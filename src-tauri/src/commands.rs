use serde_json::Value;
use std::collections::HashMap;

use crate::commands_presets::resolve_presets_dir;
use crate::lua_runtime::LuaRuntime;
use crate::types::{ControlDef, PresetInfo, ValidateResult, VideoInfo};

/// 通过 ffprobe 获取视频信息（时长、宽高）
fn get_video_info(input_path: &str) -> VideoInfo {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,duration",
            "-of", "json",
            input_path,
        ])
        .output();

    match output {
        Ok(o) => {
            let json_str = String::from_utf8_lossy(&o.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                if let Some(streams) = json.get("streams").and_then(|v| v.as_array()) {
                    if let Some(stream) = streams.first() {
                        let width = stream.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let height = stream.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        let duration = stream.get("duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        return VideoInfo { duration, width, height };
                    }
                }
            }
        }
        Err(e) => {
            log::warn("ffprobe 获取视频信息失败", &e.to_string());
        }
    }
    VideoInfo { duration: 0.0, width: 0, height: 0 }
}

/// 扫描预设目录，返回预设信息列表
#[tauri::command]
pub fn scan_presets(app: tauri::AppHandle) -> Vec<PresetInfo> {
    let dir = resolve_presets_dir(&app);
    LuaRuntime::scan_presets(&dir)
}

/// 获取指定预设的控件定义
#[tauri::command]
pub fn get_controls(app: tauri::AppHandle, preset_name: String) -> Vec<ControlDef> {
    let dir = resolve_presets_dir(&app);
    let path = format!("{}/{}.lua", dir, preset_name);
    match LuaRuntime::load_preset(&path) {
        Ok(runtime) => runtime.get_controls().unwrap_or_default(),
        Err(e) => {
            log::error("加载预设失败", &e);
            vec![]
        }
    }
}

/// 校验预设参数
#[tauri::command]
pub fn validate(
    app: tauri::AppHandle,
    preset_name: String,
    params: HashMap<String, Value>,
    _input_path: String,
) -> ValidateResult {
    let dir = resolve_presets_dir(&app);
    let path = format!("{}/{}.lua", dir, preset_name);
    match LuaRuntime::load_preset(&path) {
        Ok(runtime) => {
            let info = get_video_info(&_input_path);
            runtime.validate(&params, &info).unwrap_or(ValidateResult {
                ok: false,
                error: Some("校验执行失败".to_string()),
            })
        }
        Err(e) => {
            log::error("校验时加载预设失败", &e);
            ValidateResult {
                ok: false,
                error: Some(e),
            }
        }
    }
}

/// 日志模块
mod log {
    pub fn error(msg: &str, detail: &str) {
        eprintln!("[ERROR] {}: {}", msg, detail);
    }
    pub fn warn(msg: &str, detail: &str) {
        eprintln!("[WARN] {}: {}", msg, detail);
    }
}
