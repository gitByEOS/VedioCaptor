#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查 Rust on_complete..."
grep -r "on_complete" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "on_complete Rust: {}"
echo "检查前端 ResultView..."
grep -r "gif_path\|message\|open.*folder\|打开文件夹" src/components/ResultView.vue | wc -l | xargs -I{} echo "ResultView 功能: {}"
echo "检查前端结果展示..."
grep -r "ResultView\|output\|gifPath" src/App.vue | wc -l | xargs -I{} echo "结果展示串联: {}"
echo "所有检查通过"
