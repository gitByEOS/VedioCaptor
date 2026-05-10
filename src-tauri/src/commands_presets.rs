use crate::lua_runtime::LuaRuntime;
use crate::types::ControlDef;
use std::path::PathBuf;

/// 解析预设目录的绝对路径
/// dev 模式下从 CARGO_MANIFEST_DIR 推导项目根目录
pub fn resolve_presets_dir() -> String {
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

/// 列出目录下所有预设名（不含 .lua 后缀）
#[tauri::command]
pub fn list_presets() -> Vec<String> {
    let dir = resolve_presets_dir();
    LuaRuntime::scan_presets(&dir)
}

/// 获取指定预设的控件定义（复用已有的 get_controls 逻辑）
#[tauri::command]
pub fn get_preset_controls(preset_name: String) -> Vec<ControlDef> {
    let dir = resolve_presets_dir();
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
