use std::sync::Arc;
use std::sync::mpsc;

use crate::ffmpeg_runner::FfmpegRunner;

/// 管线步骤
#[derive(Clone)]
pub struct Step {
    pub step_name: String,
    pub command: String,
}

/// 多步骤管线执行器
/// 依次执行所有步骤，某一步失败立即终止
pub fn execute_pipeline(
    steps: Vec<Step>,
    on_stderr: impl Fn(String) + Send + Sync + 'static,
    on_exit: impl Fn(bool) + Send + Sync + 'static,
) {
    let on_stderr = Arc::new(on_stderr);
    let on_exit = Arc::new(on_exit);

    std::thread::spawn(move || {
        for step in &steps {
            let runner = FfmpegRunner::new();
            let (tx, rx) = std::sync::mpsc::channel();

            let on_stderr = Arc::clone(&on_stderr);
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

            // 等待当前步骤完成
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

/// 同步执行管线，阻塞直到所有步骤完成
/// 返回 (是否成功, 所有错误日志)
pub fn execute_pipeline_sync(steps: Vec<Step>) -> (bool, Vec<String>) {
    let mut error_log = Vec::new();

    for step in &steps {
        let runner = FfmpegRunner::new();
        let (tx, rx) = mpsc::channel();

        let tx_for_stderr = tx.clone();
        runner.start_command(
            &step.command,
            move |line| {
                let _ = tx_for_stderr.send(format!("STDERR:{}", line));
            },
            move |success| {
                let _ = tx.send(format!("EXIT:{}", success));
            },
        );

        loop {
            match rx.recv() {
                Ok(msg) => {
                    if let Some(line) = msg.strip_prefix("STDERR:") {
                        error_log.push(line.to_string());
                    } else if let Some(success_str) = msg.strip_prefix("EXIT:") {
                        if success_str != "true" {
                            return (false, error_log);
                        }
                        break;
                    }
                }
                Err(_) => {
                    return (false, error_log);
                }
            }
        }
    }

    (true, error_log)
}
