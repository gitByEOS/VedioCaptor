use tauri::AppHandle;

use crate::commands_presets::resolve_presets_dir;
use crate::lua_runtime::LuaRuntime;
use crate::pipeline_executor::{Step, PipelineConfig, execute_pipeline_sync_with_progress};
use crate::types::ConversionResult;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// 解析时间字符串为秒数 (格式: HH:MM:SS 或 HH:MM:SS.ss)
fn parse_time_to_sec(time: &str) -> f64 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let h: f64 = parts[0].parse().unwrap_or(0.0);
    let m: f64 = parts[1].parse().unwrap_or(0.0);
    let s: f64 = parts[2].parse().unwrap_or(0.0);
    h * 3600.0 + m * 60.0 + s
}

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

    // 计算时长
    let duration_sec = parse_time_to_sec(&end_time) - parse_time_to_sec(&start_time);

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
    let config = PipelineConfig { duration_sec };
    let (success, error_log) = execute_pipeline_sync_with_progress(app_handle.clone(), &runtime, steps, config);

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

    // 6. 获取输出文件信息
    let file_info = get_gif_file_info(&output_path);

    log::info("转换完成", &output_path);
    Ok(ConversionResult {
        output_path,
        message,
        file_info,
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

/// 获取 GIF 文件信息：尺寸 + 文件大小
fn get_gif_file_info(path: &str) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let size_bytes = metadata.len();
    let size_str = if size_bytes < 1024 {
        format!("{}B", size_bytes)
    } else if size_bytes < 1024 * 1024 {
        format!("{:.1}KB", size_bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", size_bytes as f64 / (1024.0 * 1024.0))
    };

    // 用 ffprobe 获取 GIF 尺寸
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=p=0:s=x",
            path,
        ])
        .output()
        .ok()?;

    let dimensions = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if dimensions.is_empty() {
        return Some(format!("大小: {}", size_str));
    }

    Some(format!("{} · {}", dimensions, size_str))
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
