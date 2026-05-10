use std::sync::Arc;

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
