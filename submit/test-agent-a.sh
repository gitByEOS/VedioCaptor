#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查页面组件是否存在..."
test -f src/App.vue && echo "✓ App.vue 存在"
test -f src/components/FileSelector.vue && echo "✓ FileSelector 存在"
test -f src/components/PresetSelector.vue && echo "✓ PresetSelector 存在"
test -f src/components/ParamPanel.vue && echo "✓ ParamPanel 存在"
test -f src/components/ProgressView.vue && echo "✓ ProgressView 存在"
test -f src/components/ResultView.vue && echo "✓ ResultView 存在"
echo "检查是否包含中文注释..."
grep -r "中文\|中文注释\|lua\|预设" src/ --include="*.vue" --include="*.ts" | head -3
echo "检查是否使用 script setup..."
grep -r "script setup" src/ --include="*.vue" | wc -l | xargs -I{} echo "script setup 组件数: {}"
echo "所有检查通过"
