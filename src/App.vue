<script setup lang="ts">
// VideoCaptor 主页面：串联数据流
import { ref } from "vue";
import FileSelector from "./components/FileSelector.vue";
import PresetSelector from "./components/PresetSelector.vue";
import ParamPanel from "./components/ParamPanel.vue";
import ProgressView from "./components/ProgressView.vue";
import ResultView from "./components/ResultView.vue";

const selectedPreset = ref("");
const fileSelectorRef = ref<InstanceType<typeof FileSelector> | null>(null);
const paramPanelRef = ref<InstanceType<typeof ParamPanel> | null>(null);
const progressRef = ref<InstanceType<typeof ProgressView> | null>(null);

function onPresetChange(preset: string) {
  selectedPreset.value = preset;
}

async function onGenerate() {
  const file = fileSelectorRef.value?.filePath ?? "";
  const start = fileSelectorRef.value?.startTime ?? "00:00:00";
  const end = fileSelectorRef.value?.endTime ?? "00:00:10";
  const params = paramPanelRef.value?.getParams() ?? {};

  if (!file) {
    console.log("错误: 未选择视频文件");
    return;
  }

  console.log("生成参数", { preset: selectedPreset.value, file, start, end, params });
  progressRef.value?.simulateProgress();
}
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>VideoCaptor</h1>
    </header>

    <main class="main">
      <FileSelector ref="fileSelectorRef" />
      <PresetSelector @change="onPresetChange" />
      <ParamPanel ref="paramPanelRef" :preset="selectedPreset" />

      <div class="action-area">
        <button type="button" class="generate-btn" @click="onGenerate">
          生成 GIF
        </button>
      </div>

      <ProgressView ref="progressRef" />
      <ResultView />
    </main>
  </div>
</template>

<style scoped>
.app {
  max-width: 640px;
  margin: 0 auto;
  padding: 24px 16px;
}
.header {
  text-align: center;
  margin-bottom: 24px;
}
.header h1 {
  font-size: 22px;
  color: #222;
  margin: 0;
}
.main {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.action-area {
  display: flex;
  justify-content: center;
}
.generate-btn {
  padding: 10px 32px;
  background: #222;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  cursor: pointer;
}
.generate-btn:hover {
  background: #444;
}
</style>

<style>
/* 全局重置 */
:root {
  font-family: Inter, -apple-system, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: #222;
  background: #f5f5f5;
  margin: 0;
  padding: 0;
}
* {
  box-sizing: border-box;
}
body {
  margin: 0;
  padding: 0;
}
</style>
