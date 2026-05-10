use crate::ffmpeg_runner::FfmpegRunner;
use crate::pipeline_executor::{Step, execute_pipeline};
use serde::Deserialize;
use std::sync::mpsc;

/// 管线步骤的 Tauri 传输格式
#[derive(Deserialize)]
pub struct PipelineStep {
    pub step_name: String,
    pub command: String,
}

/// 同步执行单个 ffmpeg 命令
#[tauri::command]
pub fn run_ffmpeg(cmd: String) -> bool {
    let runner = FfmpegRunner::new();
    let (tx, rx) = mpsc::channel();

    runner.start_command(
        &cmd,
        |_line| {},
        move |success| {
            let _ = tx.send(success);
        },
    );

    rx.recv().unwrap_or(false)
}

/// 执行多步骤管线
#[tauri::command]
pub fn run_pipeline(steps: Vec<PipelineStep>) -> bool {
    let internal_steps: Vec<Step> = steps
        .into_iter()
        .map(|s| Step {
            step_name: s.step_name,
            command: s.command,
        })
        .collect();

    let (tx, rx) = mpsc::channel();

    execute_pipeline(
        internal_steps,
        |_line| {},
        move |success| {
            let _ = tx.send(success);
        },
    );

    rx.recv().unwrap_or(false)
}
