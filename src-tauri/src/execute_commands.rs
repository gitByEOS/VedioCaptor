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
    inject_crop_filter(&mut steps, &params);

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
        if !step.command.starts_with("ffmpeg") {
            step.command = format!("ffmpeg {}", step.command);
        }
    }
}

/// 对所有使用原始输入文件的步骤注入 ffmpeg 时间参数（-ss/-t）
fn inject_time_range(steps: &mut Vec<Step>, input_path: &str, start: &str, end: &str) {
    if start.is_empty() && end.is_empty() {
        return;
    }

    let duration_sec = parse_time_to_sec(end) - parse_time_to_sec(start);

    for step in steps.iter_mut() {
        let parts = split_command(&step.command);
        if let Some(new_parts) = inject_time_args(parts, input_path, start, duration_sec) {
            step.command = join_command(&new_parts);
        }
    }
}

fn inject_time_args(
    parts: Vec<String>,
    input_path: &str,
    start: &str,
    duration_sec: f64,
) -> Option<Vec<String>> {
    let mut new_parts = Vec::with_capacity(parts.len() + 4);
    let mut i = 0;
    let mut injected = false;

    while i < parts.len() {
        if !injected && parts[i] == "-i" && parts.get(i + 1).is_some_and(|p| p == input_path) {
            if !start.is_empty() {
                new_parts.push("-ss".to_string());
                new_parts.push(start.to_string());
            }
            if duration_sec > 0.0 {
                new_parts.push("-t".to_string());
                new_parts.push(format_seconds(duration_sec));
            }
            new_parts.push(parts[i].clone());
            new_parts.push(parts[i + 1].clone());
            injected = true;
            i += 2;
            continue;
        }

        new_parts.push(parts[i].clone());
        i += 1;
    }

    injected.then_some(new_parts)
}

fn format_seconds(seconds: f64) -> String {
    let formatted = format!("{:.3}", seconds);
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn split_command(command: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '\0';

    for ch in command.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            '"' | '\'' if in_quote && ch == quote_char => {
                in_quote = false;
                quote_char = '\0';
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    result.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

fn join_command(parts: &[String]) -> String {
    parts
        .iter()
        .map(|part| quote_command_part(part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_command_part(part: &str) -> String {
    if part.is_empty() || part.chars().any(char::is_whitespace) {
        format!("\"{}\"", part.replace('"', "\\\""))
    } else {
        part.to_string()
    }
}

/// 注入裁剪滤镜：若 params 包含 crop_x/y/w/h，在所有滤镜链最前面 prepend crop
fn inject_crop_filter(steps: &mut Vec<Step>, params: &HashMap<String, Value>) {
    let crop_x = params.get("crop_x").and_then(|v| v.as_i64());
    let crop_y = params.get("crop_y").and_then(|v| v.as_i64());
    let crop_w = params.get("crop_w").and_then(|v| v.as_i64());
    let crop_h = params.get("crop_h").and_then(|v| v.as_i64());

    let (Some(cx), Some(cy), Some(cw), Some(ch)) = (crop_x, crop_y, crop_w, crop_h) else {
        return;
    };

    let crop_filter = format!("crop={}:{}:{}:{}", cw, ch, cx, cy);

    for step in steps.iter_mut() {
        if let Some(new_command) = prepend_to_filter_chain(&step.command, &crop_filter) {
            step.command = new_command;
        }
    }
}

/// 在命令的 -vf 或 -lavfi 滤镜值最前面 prepend 一个滤镜
fn prepend_to_filter_chain(command: &str, filter: &str) -> Option<String> {
    for flag in &["-vf ", "-lavfi "] {
        if let Some(pos) = command.find(flag) {
            let after_flag = &command[pos + flag.len()..];
            if after_flag.starts_with('"') {
                // 引号内的滤镜值
                if let Some(end_quote) = after_flag[1..].find('"') {
                    let filter_value = &after_flag[1..end_quote + 1];
                    let new_filter = format!("{},{}", filter, filter_value);
                    return Some(format!(
                        "{}\"{}\"{}",
                        &command[..pos + flag.len()],
                        new_filter,
                        &command[pos + flag.len() + 1 + end_quote + 1..]
                    ));
                }
            } else {
                // 无引号的单 token 滤镜值
                let token_end = after_flag.find(' ').unwrap_or(after_flag.len());
                let filter_value = &after_flag[..token_end];
                let new_filter = format!("{},{}", filter, filter_value);
                return Some(format!(
                    "{}{}{}",
                    &command[..pos + flag.len()],
                    new_filter,
                    &command[pos + flag.len() + token_end..]
                ));
            }
        }
    }
    None
}
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

    Some(format!("大小: {}", size_str))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_selected_range_for_quoted_input_path() {
        let input_path = "/tmp/movie clips/电影.mkv";
        let mut steps = vec![Step {
            step_name: "生成调色板".to_string(),
            command: format!("ffmpeg -i \"{}\" -vf fps=10 -y /tmp/out.gif", input_path),
        }];

        inject_time_range(&mut steps, input_path, "00:01:10", "00:01:25");

        assert_eq!(
            steps[0].command,
            "ffmpeg -ss 00:01:10 -t 15 -i \"/tmp/movie clips/电影.mkv\" -vf fps=10 -y /tmp/out.gif"
        );
    }

    #[test]
    fn injects_selected_range_only_for_original_video_input() {
        let input_path = "/tmp/movie clips/电影.mkv";
        let palette_path = "/tmp/palette image.png";
        let mut steps = vec![Step {
            step_name: "使用调色板生成 GIF".to_string(),
            command: format!(
                "ffmpeg -i \"{}\" -i \"{}\" -lavfi paletteuse -y /tmp/out.gif",
                input_path, palette_path
            ),
        }];

        inject_time_range(&mut steps, input_path, "00:00:03.5", "00:00:14");

        assert_eq!(
            steps[0].command,
            "ffmpeg -ss 00:00:03.5 -t 10.5 -i \"/tmp/movie clips/电影.mkv\" -i \"/tmp/palette image.png\" -lavfi paletteuse -y /tmp/out.gif"
        );
    }
}
