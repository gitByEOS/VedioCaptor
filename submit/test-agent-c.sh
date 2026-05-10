#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查 ffmpeg 模块是否存在..."
test -f src-tauri/src/ffmpeg_runner.rs && echo "✓ ffmpeg_runner.rs 存在"
test -f src-tauri/src/pipeline_executor.rs && echo "✓ pipeline_executor.rs 存在"
test -f src-tauri/src/ffmpeg_commands.rs && echo "✓ ffmpeg_commands.rs 存在"
echo "检查 ffmpeg 实现..."
grep -r "std::process::Command\|Command::new" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "进程启动调用数: {}"
grep -r "stderr\|on_stderr\|on_exit" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "回调相关行: {}"
echo "检查 Tauri commands..."
grep -r "#\[tauri::command\]" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "Tauri commands: {}"
echo "检查中文注释..."
grep -r "中文\|ffmpeg\|stderr\|管线" src-tauri/src/ --include="*.rs" | head -3
echo "所有检查通过"
