use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, Manager};

use crate::ffmpeg_runner::FfmpegRunner;
use crate::lua_runtime::LuaRuntime;
use crate::types::ProgressEvent;

/// 获取应用内 ffmpeg 资源路径
fn resolve_ffmpeg_path(app: &AppHandle) -> Option<String> {
    // 生产模式：从资源目录获取
    if let Ok(resource_dir) = app.path().resource_dir() {
        let ffmpeg = resource_dir.join("ffmpeg");
        if ffmpeg.exists() {
            return Some(ffmpeg.to_string_lossy().to_string());
        }
    }

    // 开发模式：从 node_modules 获取
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        let project_dir = manifest_path.parent().unwrap();
        let npm_ffmpeg = project_dir.join("node_modules/@ffmpeg-installer/darwin-arm64/ffmpeg");
        if npm_ffmpeg.exists() {
            return Some(npm_ffmpeg.to_string_lossy().to_string());
        }
    }

    None
}

/// 管线步骤
#[derive(Clone)]
pub struct Step {
    pub step_name: String,
    pub command: String,
}

/// 执行管线配置
pub struct PipelineConfig {
    pub duration_sec: f64, // 视频时长（秒）
}

/// 异步执行管线，逐步骤执行
pub fn execute_pipeline(
    steps: Vec<Step>,
    on_stderr: impl Fn(String) + Send + Sync + 'static,
    on_exit: impl Fn(bool) + Send + Sync + 'static,
) {
    let on_stderr = std::sync::Arc::new(on_stderr);
    let on_exit = std::sync::Arc::new(on_exit);

    std::thread::spawn(move || {
        for step in &steps {
            let runner = FfmpegRunner::new();
            let (tx, rx) = mpsc::channel();

            let on_stderr = std::sync::Arc::clone(&on_stderr);
            let tx_for_exit = tx.clone();

            runner.start_command(
                &step.command,
                move |line| {
                    on_stderr(line.to_string());
                },
                move |success| {
                    let _ = tx_for_exit.send(success);
                },
            );

            match rx.recv() {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    on_exit(false);
                    return;
                }
            }
        }

        on_exit(true);
    });
}

/// 同步执行管线，带进度事件推送
/// 每收到 stderr 行时调用 parse_progress 并发送进度事件
/// 返回 (是否成功, 所有错误日志)
pub fn execute_pipeline_sync_with_progress(
    app_handle: AppHandle,
    lua_runtime: &LuaRuntime,
    steps: Vec<Step>,
    config: PipelineConfig,
) -> (bool, Vec<String>) {
    let mut error_log = Vec::new();
    let total = steps.len();
    let ffmpeg_path = resolve_ffmpeg_path(&app_handle);

    for (i, step) in steps.iter().enumerate() {
        let mut runner = FfmpegRunner::new();
        if let Some(path) = ffmpeg_path.clone() {
            runner.set_ffmpeg_path(path);
        }
        let (tx, rx) = mpsc::channel();
        let app_handle = app_handle.clone();
        let tx_for_exit = tx.clone();

        let step_active = Arc::new(AtomicBool::new(true));
        let active_ref = Arc::clone(&step_active);
        let step_total = total;
        let estimated = Duration::from_secs_f64((config.duration_sec / 60.0) * 6.0);
        let start_time = Instant::now();

        std::thread::spawn(move || {
            while active_ref.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(300));
            }
        });

        // 发送步骤开始事件
        let start_progress = (i as f64 / total as f64) * 100.0;
        let start_event = ProgressEvent {
            step_name: step.step_name.clone(),
            step_index: i,
            total_steps: total,
            progress: start_progress,
            fake_progress: 0.0,
            message: format!("开始 {}...", step.step_name),
        };
        let _ = (&app_handle).emit("conversion-progress", &start_event);

        runner.start_command(
            &step.command,
            move |line| {
                let _ = tx.send(("line".to_string(), line.to_string()));
            },
            move |success| {
                let _ = tx_for_exit.send(("exit".to_string(), success.to_string()));
            },
        );

        // FFmpeg 启动后立即发一条 fake 进度，避免无 stderr 的步骤（如 palettegen）空白
        let _ = (&app_handle).emit(
            "conversion-progress",
            &ProgressEvent {
                step_name: step.step_name.clone(),
                step_index: i,
                total_steps: total,
                progress: 0.0,
                fake_progress: (i as f64 / total as f64) * 100.0 + 0.1,
                message: "".to_string(),
            },
        );

        loop {
            let elapsed = start_time.elapsed();
            let fake_pct = (elapsed.as_secs_f64() / estimated.as_secs_f64() * 98.0).min(98.0);
            let fake_progress = ((i as f64 / total as f64) * 100.0 + fake_pct / total as f64).min(100.0);

            // 用超时接收，让没 stderr 的步骤也能定期 emit fake 进度
            match rx.recv_timeout(Duration::from_millis(300)) {
                Ok((msg_type, data)) => {
                    if msg_type == "line" {
                        error_log.push(data.clone());
                        let event = lua_runtime.parse_progress(&data, i, &step.step_name, config.duration_sec);
                        if event.progress > 0.0 || !event.message.is_empty() {
                            let step_base = (i as f64 / total as f64) * 100.0;
                            let step_progress = event.progress / (total as f64);
                            let overall_progress = step_base + step_progress;
                            let event = ProgressEvent {
                                total_steps: total,
                                progress: overall_progress.min(100.0).max(0.0),
                                fake_progress,
                                ..event
                            };
                            eprintln!("[PROG] real step={i} real={overall_progress:.1}% fake={fake_progress:.1}%");
                            let _ = (&app_handle).emit("conversion-progress", &event);
                        }
                    } else if msg_type == "exit" {
                        if data != "true" {
                            step_active.store(false, Ordering::SeqCst);
                            return (false, error_log);
                        }
                        step_active.store(false, Ordering::SeqCst);
                        let step_done_base = ((i + 1) as f64 / total as f64) * 100.0;
                        let done_event = ProgressEvent {
                            step_name: step.step_name.clone(),
                            step_index: i,
                            total_steps: total,
                            progress: step_done_base.min(100.0),
                            fake_progress: 0.0,
                            message: format!("完成 {}", step.step_name),
                        };
                        let _ = (&app_handle).emit("conversion-progress", &done_event);
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // 300ms 没收到 stderr，发 fake 进度
                    eprintln!("[PROG] fake step={i} fake={fake_progress:.1}%");
                    let _ = (&app_handle).emit(
                        "conversion-progress",
                        &ProgressEvent {
                            step_name: step.step_name.clone(),
                            step_index: i,
                            total_steps: total,
                            progress: 0.0,
                            fake_progress,
                            message: "".to_string(),
                        },
                    );
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    step_active.store(false, Ordering::SeqCst);
                    return (false, error_log);
                }
            }
        }
    }

    (true, error_log)
}
