#!/bin/bash
cd /Users/eos./dev/VideoCaptor/master/worktrees/agent-b
npm run build 2>&1 | tail -5
echo "日志组件已更新"