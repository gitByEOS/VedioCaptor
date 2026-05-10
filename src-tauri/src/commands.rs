use serde_json::Value;
use std::collections::HashMap;

use crate::commands_presets::resolve_presets_dir;
use crate::lua_runtime::LuaRuntime;
use crate::types::{ControlDef, ValidateResult, VideoInfo};

/// 扫描预设目录，返回预设名列表
#[tauri::command]
pub fn scan_presets(app: tauri::AppHandle) -> Vec<String> {
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
            let info = VideoInfo {
                duration: 0.0,
                width: 0,
                height: 0,
            };
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
}
