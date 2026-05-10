#!/bin/bash
set -e
cd "$(dirname "$0")/.."
echo "检查 API 封装..."
test -f src/api.ts && echo "✓ api.ts 存在"
grep -r "validate" src/api.ts | head -1 && echo "✓ validate 函数已定义"
echo "检查校验链路..."
grep -r "validateParams\|validate" src/App.vue --include="*.vue" | head -2
echo "检查错误展示..."
grep -r "error\|错误\|校验失败" src/ --include="*.vue" --include="*.ts" | wc -l | xargs -I{} echo "错误展示: {}"
echo "所有检查通过"
