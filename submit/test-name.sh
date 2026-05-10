#!/bin/bash
cd /Users/eos./dev/VideoCaptor/master/worktrees/agent-d
grep -r "VideoCaptor\|video_captor" package.json src-tauri/tauri.conf.json src/App.vue
echo "名字已统一"