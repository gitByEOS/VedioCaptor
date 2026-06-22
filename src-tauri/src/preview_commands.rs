use crate::types::{PreviewVideo, VideoInfo};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

const MAX_PREVIEW_SECONDS: f64 = 12.0;

fn resolve_ffmpeg_path(app: &tauri::AppHandle) -> String {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let ffmpeg = resource_dir.join("ffmpeg");
        if ffmpeg.exists() {
            return ffmpeg.to_string_lossy().to_string();
        }
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        let dev_ffmpeg = manifest_path.join("resources/ffmpeg");
        if dev_ffmpeg.exists() {
            return dev_ffmpeg.to_string_lossy().to_string();
        }

        let project_dir = manifest_path.parent().unwrap();
        let npm_ffmpeg = project_dir.join("node_modules/@ffmpeg-installer/darwin-arm64/ffmpeg");
        if npm_ffmpeg.exists() {
            return npm_ffmpeg.to_string_lossy().to_string();
        }
    }

    "ffmpeg".to_string()
}

fn parse_duration(stderr: &str) -> f64 {
    let Some(pos) = stderr.find("Duration: ") else {
        return 0.0;
    };
    let value = &stderr[pos + "Duration: ".len()..];
    let time = value.split(',').next().unwrap_or("").trim();
    let parts = time.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return 0.0;
    }
    let h = parts[0].parse::<f64>().unwrap_or(0.0);
    let m = parts[1].parse::<f64>().unwrap_or(0.0);
    let s = parts[2].parse::<f64>().unwrap_or(0.0);
    h * 3600.0 + m * 60.0 + s
}

fn parse_dimensions(stderr: &str) -> (u32, u32) {
    for line in stderr.lines() {
        if !line.contains("Video:") {
            continue;
        }

        for token in line.split(|c: char| c == ',' || c.is_whitespace()) {
            let Some((w, h)) = token.split_once('x') else {
                continue;
            };
            let width = w.parse::<u32>().unwrap_or(0);
            let height = h.parse::<u32>().unwrap_or(0);
            if width > 0 && height > 0 {
                return (width, height);
            }
        }
    }
    (0, 0)
}

#[tauri::command]
pub async fn prepare_video_preview(
    app: tauri::AppHandle,
    input_path: String,
    start_sec: f64,
    end_sec: f64,
) -> Result<PreviewVideo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        prepare_video_preview_sync(app, input_path, start_sec, end_sec)
    })
    .await
    .map_err(|e| format!("预览任务执行失败: {}", e))?
}

fn prepare_video_preview_sync(
    app: tauri::AppHandle,
    input_path: String,
    start_sec: f64,
    end_sec: f64,
) -> Result<PreviewVideo, String> {
    let duration_sec = (end_sec - start_sec).clamp(1.0, MAX_PREVIEW_SECONDS);
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("获取缓存目录失败: {}", e))?;
    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("创建缓存目录失败: {}", e))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let output_path = cache_dir.join(format!(
        "preview-{}-{:.0}.mp4",
        timestamp,
        start_sec.max(0.0) * 1000.0
    ));
    let ffmpeg = resolve_ffmpeg_path(&app);
    let output = Command::new(&ffmpeg)
        .args([
            "-y",
            "-ss",
            &format!("{:.3}", start_sec.max(0.0)),
            "-i",
            &input_path,
            "-map",
            "0:v:0",
            "-t",
            &format!("{:.3}", duration_sec),
            "-an",
            "-vf",
            "scale=min(1280\\,iw):-2",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-preset",
            "ultrafast",
            "-movflags",
            "+faststart",
            output_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|e| format!("启动预览转码失败: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let duration = parse_duration(&stderr);
    let (width, height) = parse_dimensions(&stderr);

    if !output.status.success() {
        return Err(stderr);
    }

    Ok(PreviewVideo {
        preview_path: output_path.to_string_lossy().to_string(),
        video_info: VideoInfo {
            duration,
            width,
            height,
        },
    })
}
