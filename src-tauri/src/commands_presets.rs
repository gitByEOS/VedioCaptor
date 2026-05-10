use crate::lua_runtime::LuaRuntime;
use crate::types::ControlDef;

/// 列出目录下所有预设名（不含 .lua 后缀）
#[tauri::command]
pub fn list_presets(dir: String) -> Vec<String> {
    LuaRuntime::scan_presets(&dir)
}

/// 获取指定预设的控件定义（复用已有的 get_controls 逻辑）
#[tauri::command]
pub fn get_preset_controls(preset_path: String) -> Vec<ControlDef> {
    match LuaRuntime::load_preset(&preset_path) {
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
