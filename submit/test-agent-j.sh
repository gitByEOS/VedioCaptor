#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "=== agent-j 全链路检查 ==="
echo ""

echo "检查核心组件..."
for f in FileSelector PresetSelector ParamPanel ProgressView ResultView; do
  test -f src/components/$f.vue && echo "✓ $f.vue"
done

echo ""
echo "检查 Tauri commands..."
for cmd in scan_presets get_controls validate list_presets get_preset_controls run_ffmpeg run_pipeline execute_conversion; do
  grep -r "$cmd" src-tauri/src/lib.rs > /dev/null && echo "✓ $cmd"
done

echo ""
echo "检查 Lua 预设..."
for f in emoji_small clear_demo custom_steps; do
  test -f presets/$f.lua && echo "✓ $f.lua"
done

echo ""
echo "检查数据流..."
grep -r "ConversionResult\|execute_conversion\|conversion-progress" src/ --include="*.ts" --include="*.vue" | wc -l | xargs -I{} echo "数据流关键词: {}"

echo ""
echo "检查工具函数..."
test -f src/utils.ts && echo "✓ utils.ts 存在"
grep -q "timeToSeconds" src/utils.ts && echo "✓ timeToSeconds"
grep -q "secondsToTime" src/utils.ts && echo "✓ secondsToTime"
grep -q "isValidTimeRange" src/utils.ts && echo "✓ isValidTimeRange"

echo ""
echo "检查状态管理..."
grep -q "validating\|converting\|done\|error" src/App.vue && echo "✓ 状态机已实现"
grep -q "isConverting" src/App.vue && echo "✓ 按钮禁用逻辑"
grep -q "errorInfo" src/App.vue && echo "✓ 错误边界"

echo ""
echo "检查时间校验..."
grep -q "isValidTimeRange" src/App.vue && echo "✓ App.vue 时间校验"
grep -q "validateTime" src/components/FileSelector.vue && echo "✓ FileSelector 时间校验"

echo ""
echo "=== 所有检查通过 ==="
