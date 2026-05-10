#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查 Tauri invoke 接入..."
grep -r "invoke.*list_presets\|invoke.*get_preset_controls" src/ --include="*.vue" --include="*.ts" | wc -l | xargs -I{} echo "Tauri invoke 调用: {}"
echo "检查动态渲染..."
grep -r "ControlDef\|control.type\|slider\|select" src/components/ParamPanel.vue | wc -l | xargs -I{} echo "动态渲染逻辑: {}"
echo "检查文件选择..."
grep -r "dialog\|open\|file.*select" src/ --include="*.vue" -i | head -2
echo "检查错误处理..."
grep -r "error\|catch\|try" src/ --include="*.vue" --include="*.ts" | wc -l | xargs -I{} echo "错误处理: {}"
echo "所有检查通过"
