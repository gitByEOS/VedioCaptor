use tauri::AppHandle;

use crate::commands_presets::resolve_presets_dir;
use crate::lua_runtime::LuaRuntime;
use crate::pipeline_executor::Step;
use crate::pipeline_executor::execute_pipeline_sync_with_progress;
use crate::types::ConversionResult;
use serde_json::Value;
use std::collections::HashMap;

/// 执行完整转换管线：加载预设 -> 构建命令 -> 执行
#[tauri::command]
pub async fn execute_conversion(
    app_handle: AppHandle,
    preset_name: String,
    params: HashMap<String, Value>,
    input_path: String,
    start_time: String,
    end_time: String,
    output_path: String,
) -> Result<ConversionResult, String> {
    let dir = resolve_presets_dir(&app_handle);
    let preset_path = format!("{}/{}.lua", dir, preset_name);

    log::info("开始转换", &format!("预设={}, 输入={}", preset_path, input_path));

    // 1. 加载 Lua 预设
    let runtime = LuaRuntime::load_preset(&preset_path)?;

    // 2. 调用 build_command_pipeline 获取步骤
    let mut steps = runtime.build_command_pipeline(&params, &input_path, &output_path)?;
    if steps.is_empty() {
        return Err("build_command_pipeline 返回空步骤".to_string());
    }

    // 3. 为所有步骤添加 ffmpeg 前缀，对使用原始输入的步骤注入时间参数
    ensure_ffmpeg_prefix(&mut steps);
    inject_time_range(&mut steps, &input_path, &start_time, &end_time);

    // 打印每步命令到终端日志
    for (i, step) in steps.iter().enumerate() {
        log::info(&format!("步骤{}", i + 1), &step.command);
    }

    // 4. 同步执行管线（带进度推送）
    let (success, error_log) = execute_pipeline_sync_with_progress(app_handle.clone(), &runtime, steps);

    if !success {
        let error_msg = if error_log.is_empty() {
            "管线执行失败（无错误日志）".to_string()
        } else {
            error_log.join("\n")
        };
        log::error("管线执行失败", &error_msg);
        return Err(error_msg);
    }

    // 5. 调用 on_complete 后处理
    let message = runtime.on_complete(&output_path, &params).unwrap_or(None);

    log::info("转换完成", &output_path);
    Ok(ConversionResult {
        output_path,
        message,
    })
}

/// 为所有步骤添加 ffmpeg 前缀
fn ensure_ffmpeg_prefix(steps: &mut Vec<Step>) {
    for step in steps {
        if !step.command.starts_with("ffmpeg") && !step.command.starts_with("ffprobe") {
            step.command = format!("ffmpeg {}", step.command);
        }
    }
}

/// 对所有使用原始输入文件的步骤注入 ffmpeg 时间参数（-ss/-to）
fn inject_time_range(steps: &mut Vec<Step>, input_path: &str, start: &str, end: &str) {
    if start.is_empty() && end.is_empty() {
        return;
    }

    let mut time_args = String::new();
    if !start.is_empty() {
        time_args.push_str(&format!("-ss {}", start));
    }
    if !end.is_empty() {
        if !time_args.is_empty() {
            time_args.push(' ');
        }
        time_args.push_str(&format!("-to {}", end));
    }

    for step in steps.iter_mut() {
        // 只对包含原始输入文件的步骤注入时间参数
        if step.command.contains(&format!("-i {}", input_path)) {
            if step.command.starts_with("ffmpeg ") {
                step.command = format!("ffmpeg {} {}", time_args, &step.command[7..]);
            } else {
                step.command = format!("ffmpeg {} {}", time_args, step.command);
            }
        }
    }
}

/// 日志模块
mod log {
    pub fn info(msg: &str, detail: &str) {
        eprintln!("[INFO] {}: {}", msg, detail);
    }
    pub fn error(msg: &str, detail: &str) {
        eprintln!("[ERROR] {}: {}", msg, detail);
    }
}
