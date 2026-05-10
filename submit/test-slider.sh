#!/bin/bash
cd /Users/eos./dev/VideoCaptor/master/worktrees/agent-c
npm run build 2>&1 | tail -5
echo "时间滑块组件已更新"