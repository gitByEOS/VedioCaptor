use crate::lua_runtime::LuaRuntime;
use crate::types::{ControlDef, PresetInfo};
use std::path::PathBuf;
use tauri::Manager;

/// 解析预设目录的绝对路径
/// 使用 Tauri 资源目录，兼容 dev 和 production
pub fn resolve_presets_dir(app: &tauri::AppHandle) -> String {
    // 尝试 Tauri 资源目录（生产模式）
    if let Ok(resource_dir) = app.path().resource_dir() {
        let presets = resource_dir.join("presets");
        if presets.exists() {
            return presets.to_string_lossy().to_string();
        }
    }

    // 开发模式：src-tauri/../presets
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let project_dir = PathBuf::from(&manifest_dir).parent().unwrap().to_path_buf();
        let presets = project_dir.join("presets");
        if presets.exists() {
            return presets.to_string_lossy().to_string();
        }
    }

    // fallback: 当前工作目录下的 presets
    "presets".to_string()
}

/// 列出预设信息（id + 显示名）
#[tauri::command]
pub fn list_presets(app: tauri::AppHandle) -> Vec<PresetInfo> {
    let dir = resolve_presets_dir(&app);
    LuaRuntime::scan_presets(&dir)
}

/// 获取指定预设的控件定义（复用已有的 get_controls 逻辑）
#[tauri::command]
pub fn get_preset_controls(app: tauri::AppHandle, preset_name: String) -> Vec<ControlDef> {
    let dir = resolve_presets_dir(&app);
    let path = format!("{}/{}.lua", dir, preset_name);
    match LuaRuntime::load_preset(&path) {
        Ok(runtime) => runtime.get_controls().unwrap_or_default(),
        Err(e) => {
            log::error("获取预设控件失败", &e);
            vec![]
        }
    }
}

mod log {
    pub fn error(msg: &str, detail: &str) {
        eprintln!("[ERROR] {}: {}", msg, detail);
    }
}
