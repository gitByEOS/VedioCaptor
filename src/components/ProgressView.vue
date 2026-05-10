<script setup lang="ts">
// 进度展示组件：进度条 + 步骤名称 + 百分比 + 命令行日志
import { ref } from "vue";

interface ProgressData {
  step_name: string;
  step_index: number;
  total_steps: number;
  progress: number;
  message: string;
}

const progress = ref(0);
const stepName = ref("等待中");
const statusText = ref("等待任务启动...");
const commandLog = ref<string[]>([]);
const isRunning = ref(false);

let currentStepIndex = -1;

// 接收真实进度事件
function updateProgress(data: ProgressData) {
  if (!isRunning.value) {
    isRunning.value = true;
    commandLog.value = [];
  }

  if (data.step_index !== currentStepIndex) {
    currentStepIndex = data.step_index;
    commandLog.value.push(`[${data.step_index + 1}/${data.total_steps}] ${data.step_name}`);
  }

  progress.value = Math.round(data.progress);
  stepName.value = data.step_name;
  statusText.value = `${data.step_name} (${progress}%)`;

  if (data.message) {
    commandLog.value.push(data.message);
  }
}

function resetProgress() {
  progress.value = 0;
  stepName.value = "等待中";
  statusText.value = "等待任务启动...";
  commandLog.value = [];
  isRunning.value = false;
  currentStepIndex = -1;
}

function markComplete() {
  progress.value = 100;
  statusText.value = "完成";
  commandLog.value.push("转换完成");
  isRunning.value = false;
}

defineExpose({ updateProgress, resetProgress, markComplete, progress, statusText, commandLog });
</script>

<template>
  <section class="panel">
    <h3>进度</h3>
    <div class="step-info">
      <span class="step-name">{{ stepName }}</span>
      <span class="percent">{{ progress }}%</span>
    </div>
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
.step-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
}
.step-name {
  font-size: 13px;
  font-weight: 600;
  color: #333;
}
.percent {
  font-size: 13px;
  color: #666;
  font-weight: 600;
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
