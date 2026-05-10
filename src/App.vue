<script setup lang="ts">
// VideoCaptor 主页面：串联数据流
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { validateParams, executeConversion } from "./api";
import FileSelector from "./components/FileSelector.vue";
import PresetSelector from "./components/PresetSelector.vue";
import ParamPanel from "./components/ParamPanel.vue";
import ProgressView from "./components/ProgressView.vue";
import ResultView from "./components/ResultView.vue";

const selectedPreset = ref("");
const validateError = ref("");
const fileSelectorRef = ref<InstanceType<typeof FileSelector> | null>(null);
const paramPanelRef = ref<InstanceType<typeof ParamPanel> | null>(null);
const progressRef = ref<InstanceType<typeof ProgressView> | null>(null);

let unlisten: (() => void) | null = null;

// 监听后端进度事件
onMounted(async () => {
  unlisten = await listen("conversion-progress", (event) => {
    const payload = event.payload as Record<string, unknown>;
    progressRef.value?.updateProgress({
      step_name: payload.step_name as string,
      step_index: payload.step_index as number,
      total_steps: payload.total_steps as number,
      progress: payload.progress as number,
      message: payload.message as string,
    });
  });
});

onUnmounted(() => {
  unlisten?.();
});

function onPresetChange(preset: string) {
  selectedPreset.value = preset;
  validateError.value = "";
}

async function onGenerate() {
  validateError.value = "";
  const file = fileSelectorRef.value?.filePath ?? "";
  const start = fileSelectorRef.value?.startTime ?? "00:00:00";
  const end = fileSelectorRef.value?.endTime ?? "00:00:10";
  const params = paramPanelRef.value?.getParams() ?? {};

  if (!file) {
    console.log("错误: 未选择视频文件");
    return;
  }

  if (!selectedPreset.value) {
    validateError.value = "请先选择预设";
    paramPanelRef.value?.setValidateError(validateError.value);
    return;
  }

  const result = await validateParams(selectedPreset.value, params, file);
  if (!result.ok) {
    validateError.value = result.error ?? "参数校验失败";
    paramPanelRef.value?.setValidateError(validateError.value);
    return;
  }

  const presetPath = `presets/${selectedPreset.value}.lua`;
  const outputPath = file.replace(/\.[^.]+$/, "") + ".gif";

  try {
    progressRef.value?.resetProgress();
    const output = await executeConversion(presetPath, params, file, start, end, outputPath);
    console.log("转换完成", output);
    progressRef.value?.markComplete();
  } catch (err) {
    validateError.value = `转换失败: ${err}`;
    paramPanelRef.value?.setValidateError(validateError.value);
  }
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
