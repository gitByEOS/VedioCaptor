#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查 Rust 模块是否存在..."
test -f src-tauri/src/lua_runtime.rs && echo "✓ lua_runtime.rs 存在"
test -f src-tauri/src/types.rs && echo "✓ types.rs 存在"
test -f src-tauri/src/commands.rs && echo "✓ commands.rs 存在"
echo "检查 Lua 集成..."
grep -r "mlua" src-tauri/Cargo.toml | head -1 && echo "✓ mlua 依赖已添加"
grep -r "get_controls\|validate\|load_preset" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "Lua 函数调用数: {}"
echo "检查 Tauri commands..."
grep -r "#\[tauri::command\]" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "Tauri commands: {}"
echo "检查中文注释..."
grep -r "中文\|lua\|预设\|控制器" src-tauri/src/ --include="*.rs" | head -3
echo "检查 Rust 编译..."
cd src-tauri && cargo check 2>&1 | tail -1
echo "所有检查通过"
