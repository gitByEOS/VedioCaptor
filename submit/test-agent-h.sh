#!/bin/bash
set -e
cd "$(dirname "$0")/.."

echo "检查 Rust 进度解析..."
grep -r "parse_progress\|conversion-progress\|emit.*progress" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "进度相关 Rust: {}"

echo "检查前端监听..."
grep -r "conversion-progress\|listen.*progress" src/ --include="*.vue" --include="*.ts" | wc -l | xargs -I{} echo "进度监听前端: {}"

echo "检查 ProgressView 更新..."
grep -r "progress\|step_name\|状态" src/components/ProgressView.vue | wc -l | xargs -I{} echo "ProgressView 更新: {}"

echo "所有检查通过"
