use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::env;

/// ffmpeg 可执行文件路径（运行时检测并缓存）
static FFMPEG_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// 检测 ffmpeg/ffprobe 可执行文件路径
fn resolve_executable(name: &str) -> String {
    // 优先使用缓存
    if let Some(cached) = FFMPEG_PATH.get() {
        if name == "ffmpeg" {
            return cached.clone();
        }
    }

    // 尝试 PATH 环境变量
    if let Ok(path_var) = env::var("PATH") {
        for path_dir in path_var.split(':') {
            let full_path = format!("{}/{}", path_dir, name);
            if std::path::Path::new(&full_path).exists() {
                if name == "ffmpeg" {
                    let _ = FFMPEG_PATH.set(full_path.clone());
                }
                return full_path;
            }
        }
    }

    // macOS Homebrew 常见路径
    let homebrew_paths = ["/opt/homebrew/bin", "/usr/local/bin"];
    for dir in homebrew_paths {
        let full_path = format!("{}/{}", dir, name);
        if std::path::Path::new(&full_path).exists() {
            if name == "ffmpeg" {
                let _ = FFMPEG_PATH.set(full_path.clone());
            }
            return full_path;
        }
    }

    // fallback: 直接使用命令名（依赖系统 PATH）
    name.to_string()
}

/// ffmpeg 执行器，封装 ffmpeg 进程生命周期
pub struct FfmpegRunner {
    child: Arc<Mutex<Option<Child>>>,
    running: Arc<AtomicBool>,
}

impl FfmpegRunner {
    pub fn new() -> Self {
        FfmpegRunner {
            child: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 启动 ffmpeg 命令，逐行捕获 stderr，完成后调用 on_exit
    pub fn start_command(
        &self,
        cmd: &str,
        on_stderr: impl Fn(&str) + Send + Sync + 'static,
        on_exit: impl Fn(bool) + Send + Sync + 'static,
    ) {
        let parts = shell_split(cmd);
        if parts.is_empty() {
            on_exit(false);
            return;
        }

        // 动态检测 ffmpeg/ffprobe 可执行文件路径
        let executable = resolve_executable(&parts[0]);
        let mut command = Command::new(&executable);
        command.args(&parts[1..]);
        command.stderr(Stdio::piped());
        command.stdout(Stdio::null());

        let child_result = command.spawn();

        match child_result {
            Ok(child) => {
                self.running.store(true, Ordering::SeqCst);
                let child_ref = Arc::clone(&self.child);
                let running = Arc::clone(&self.running);

                // 存储 child 到 Mutex
                {
                    let mut guard = child_ref.lock().unwrap();
                    *guard = Some(child);
                }

                // 启动 stderr 读取线程
                let child_ref = Arc::clone(&self.child);

                std::thread::spawn(move || {
                    let stderr_opt = {
                        let mut guard = child_ref.lock().unwrap();
                        guard.as_mut().and_then(|c| c.stderr.take())
                    };

                    if let Some(stderr) = stderr_opt {
                        let reader = BufReader::new(stderr);
                        for line_result in reader.lines() {
                            if !running.load(Ordering::SeqCst) {
                                break;
                            }
                            if let Ok(line) = line_result {
                                on_stderr(&line);
                            }
                        }
                    }

                    // 等待进程退出
                    let success = {
                        let mut guard = child_ref.lock().unwrap();
                        if let Some(child) = guard.as_mut() {
                            match child.wait() {
                                Ok(status) => status.success(),
                                Err(_) => false,
                            }
                        } else {
                            false
                        }
                    };

                    running.store(false, Ordering::SeqCst);
                    on_exit(success);
                });
            }
            Err(e) => {
                eprintln!("启动 ffmpeg 失败: {}", e);
                on_exit(false);
            }
        }
    }

    /// 终止正在运行的 ffmpeg 进程
    pub fn kill(&self) {
        self.running.store(false, Ordering::SeqCst);
        let mut guard = self.child.lock().unwrap();
        if let Some(ref mut child) = *guard {
            let _ = child.kill();
            let _ = child.wait();
        }
        *guard = None;
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

/// 简单的命令行分词，按空格分割（不处理引号内的空格）
fn shell_split(cmd: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '\0';

    for ch in cmd.chars() {
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
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}
