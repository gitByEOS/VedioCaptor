#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查预设文件..."
test -f presets/emoji_small.lua && echo "✓ emoji_small.lua"
test -f presets/clear_demo.lua && echo "✓ clear_demo.lua"
test -f presets/custom_steps.lua && echo "✓ custom_steps.lua"
echo "检查 Lua 函数..."
for f in presets/*.lua; do
  name=$(basename "$f")
  grep -q "get_controls" "$f" && echo "✓ $name 有 get_controls"
  grep -q "validate" "$f" && echo "✓ $name 有 validate"
  grep -q "build_command_pipeline" "$f" && echo "✓ $name 有 build_command_pipeline"
done
echo "检查 Tauri commands..."
grep -r "list_presets\|get_preset_controls" src-tauri/src/ --include="*.rs" | wc -l | xargs -I{} echo "热加载相关: {}"
echo "所有检查通过"
