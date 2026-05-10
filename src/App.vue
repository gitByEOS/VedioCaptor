<script setup lang="ts">
// VideoCaptor 主页面：组装各子组件
import { ref } from "vue";
import FileSelector from "./components/FileSelector.vue";
import PresetSelector from "./components/PresetSelector.vue";
import ParamPanel from "./components/ParamPanel.vue";
import ProgressView from "./components/ProgressView.vue";
import ResultView from "./components/ResultView.vue";

const selectedPreset = ref("");
const progressRef = ref<InstanceType<typeof ProgressView> | null>(null);

function onGenerate() {
  // 占位：触发 Tauri invoke 生成 GIF
  console.log("占位：调用 Tauri invoke('generate_gif')");
  progressRef.value?.simulateProgress();
}
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>VideoCaptor</h1>
    </header>

    <main class="main">
      <FileSelector />
      <PresetSelector @change="(p: string) => (selectedPreset = p)" />
      <ParamPanel :preset="selectedPreset" />

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
