<script setup lang="ts">
// 进度展示组件：进度条 + 状态文本 + 命令行预览
import { ref } from "vue";

const progress = ref(0);
const statusText = ref("等待中");
const commandLog = ref(["$ 等待任务启动..."]);

// 占位：模拟进度推进
function simulateProgress() {
  console.log("占位：触发 Tauri 生成 GIF 命令");
  statusText.value = "处理中...";
  commandLog.value.push("$ ffmpeg -i input.mp4 -vf fps=10 output.gif");
  let p = 0;
  const timer = setInterval(() => {
    p += 10;
    progress.value = Math.min(p, 100);
    if (p >= 100) {
      clearInterval(timer);
      statusText.value = "完成";
      commandLog.value.push("完成: output.gif");
    }
  }, 300);
}

defineExpose({ simulateProgress, progress, statusText, commandLog });
</script>

<template>
  <section class="panel">
    <h3>进度</h3>
    <div class="status">{{ statusText }}</div>
    <div class="progress-bar">
      <div class="progress-fill" :style="{ width: progress + '%' }"></div>
    </div>
    <div class="log-box">
      <div v-for="(line, i) in commandLog" :key="i" class="log-line">{{ line }}</div>
    </div>
  </section>
</template>

<style scoped>
.panel {
  padding: 16px;
  border: 1px solid #ccc;
  border-radius: 8px;
  background: #fafafa;
}
h3 {
  margin: 0 0 12px;
  font-size: 14px;
  color: #333;
}
.status {
  font-size: 13px;
  color: #555;
  margin-bottom: 8px;
}
.progress-bar {
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 12px;
}
.progress-fill {
  height: 100%;
  background: #333;
  transition: width 0.3s;
}
.log-box {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 10px;
  border-radius: 6px;
  font-family: monospace;
  font-size: 12px;
  max-height: 120px;
  overflow-y: auto;
}
.log-line {
  line-height: 1.5;
}
</style>
