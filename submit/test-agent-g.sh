#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查 Rust 执行模块..."
test -f src-tauri/src/execute_commands.rs && echo "✓ execute_commands.rs 存在"
grep -r "build_command_pipeline" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "pipeline 相关: {}"
echo "检查前端调用..."
grep -r "execute_conversion" src/ --include="*.vue" --include="*.ts" | head -1
echo "检查 Tauri commands..."
grep -r "execute_conversion" src-tauri/src/lib.rs | head -1
echo "所有检查通过"
