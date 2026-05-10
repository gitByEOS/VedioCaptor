use tauri::AppHandle;

use crate::lua_runtime::LuaRuntime;
use crate::pipeline_executor::Step;
use crate::pipeline_executor::execute_pipeline_sync_with_progress;
use serde_json::Value;
use std::collections::HashMap;

/// 执行完整转换管线：加载预设 -> 构建命令 -> 执行
#[tauri::command]
pub async fn execute_conversion(
    app_handle: AppHandle,
    preset_path: String,
    params: HashMap<String, Value>,
    input_path: String,
    start_time: String,
    end_time: String,
    output_path: String,
) -> Result<String, String> {
    log::info("开始转换", &format!("预设={}, 输入={}", preset_path, input_path));

    // 1. 加载 Lua 预设
    let runtime = LuaRuntime::load_preset(&preset_path)?;

    // 2. 调用 build_command_pipeline 获取步骤
    let mut steps = runtime.build_command_pipeline(&params, &input_path, &output_path)?;
    if steps.is_empty() {
        return Err("build_command_pipeline 返回空步骤".to_string());
    }

    // 3. 注入时间截取到第一步
    inject_time_range(&mut steps, &start_time, &end_time);

    // 4. 同步执行管线（带进度推送）
    let (success, _log) = execute_pipeline_sync_with_progress(app_handle.clone(), &runtime, steps);

    if !success {
        return Err("管线执行失败".to_string());
    }

    log::info("转换完成", &output_path);
    Ok(output_path)
}

/// 将起止时间注入为 ffmpeg 的 -ss/-to 参数
fn inject_time_range(steps: &mut Vec<Step>, start: &str, end: &str) {
    if start.is_empty() && end.is_empty() {
        return;
    }

    let first = &mut steps[0];
    let mut prefix = String::new();
    if !start.is_empty() {
        prefix.push_str(&format!("-ss {}", start));
    }
    if !end.is_empty() {
        if !prefix.is_empty() {
            prefix.push(' ');
        }
        prefix.push_str(&format!("-to {}", end));
    }
    first.command = format!("ffmpeg {} {}", prefix, first.command);
}

/// 日志模块
mod log {
    pub fn info(msg: &str, detail: &str) {
        eprintln!("[INFO] {}: {}", msg, detail);
    }
}
