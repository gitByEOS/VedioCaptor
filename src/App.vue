<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { validateParams, executeConversion, type ConversionResult } from "./api";
import { isValidTimeRange } from "./utils";
import FileSelector from "./components/FileSelector.vue";
import PresetSelector from "./components/PresetSelector.vue";
import ParamPanel from "./components/ParamPanel.vue";
import ProgressView from "./components/ProgressView.vue";
import ResultView from "./components/ResultView.vue";

type AppStatus = "idle" | "validating" | "converting" | "done" | "error";

const selectedPreset = ref("");
const status = ref<AppStatus>("idle");
const errorInfo = ref("");
const fileSelectorRef = ref<InstanceType<typeof FileSelector> | null>(null);
const paramPanelRef = ref<InstanceType<typeof ParamPanel> | null>(null);
const progressRef = ref<InstanceType<typeof ProgressView> | null>(null);
const resultRef = ref<ConversionResult | null>(null);
const logMessages = ref<string[]>([]);

let unlisten: (() => void) | null = null;

const isConverting = computed(() => status.value === "converting" || status.value === "validating");

function addLog(msg: string) {
  const timestamp = new Date().toLocaleTimeString();
  logMessages.value.push(`[${timestamp}] ${msg}`);
}

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
    if (payload.message) {
      addLog(payload.message as string);
    }
  });
});

onUnmounted(() => {
  unlisten?.();
});

function setStatus(s: AppStatus) {
  status.value = s;
  if (s === "validating") {
    addLog("参数校验中...");
  } else if (s === "converting") {
    addLog("开始转换...");
  } else if (s === "done") {
    addLog("转换完成");
  }
}

function onPresetChange(preset: string) {
  selectedPreset.value = preset;
  errorInfo.value = "";
  if (status.value === "error") {
    setStatus("idle");
  }
}

function collectForm() {
  const file = fileSelectorRef.value?.filePath ?? "";
  const start = fileSelectorRef.value?.startTime ?? "00:00:00";
  const end = fileSelectorRef.value?.endTime ?? "00:00:10";
  const params = paramPanelRef.value?.getParams() ?? {};
  return { file, start, end, params };
}

function showValidation(msg: string) {
  errorInfo.value = msg;
  addLog(`错误: ${msg}`);
  paramPanelRef.value?.setValidateError(msg);
  setStatus("error");
}

async function onGenerate() {
  errorInfo.value = "";
  resultRef.value = null;

  const { file, start, end, params } = collectForm();

  if (!file) {
    showValidation("请先选择视频文件");
    return;
  }

  if (!selectedPreset.value) {
    showValidation("请先选择预设");
    return;
  }

  if (!isValidTimeRange(start, end)) {
    showValidation("结束时间必须大于起始时间");
    fileSelectorRef.value?.validateTime();
    return;
  }

  setStatus("validating");

  const validateResult = await validateParams(selectedPreset.value, params, file);
  if (!validateResult.ok) {
    showValidation(validateResult.error ?? "参数校验失败");
    return;
  }

  setStatus("converting");
  progressRef.value?.resetProgress();

  const presetName = selectedPreset.value;
  const outputPath = file.replace(/\.[^.]+$/, "") + ".gif";

  try {
    const conversionResult = await executeConversion(
      presetName, params, file, start, end, outputPath,
    );
    resultRef.value = conversionResult;
    progressRef.value?.markComplete();
    setStatus("done");
  } catch (err) {
    errorInfo.value = `转换失败: ${err}`;
    addLog(`转换失败: ${err}`);
    progressRef.value?.markComplete();
    setStatus("error");
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
        <button
          type="button"
          class="generate-btn"
          :class="{ disabled: isConverting }"
          :disabled="isConverting"
          @click="onGenerate"
        >
          {{ isConverting ? "处理中..." : "生成 GIF" }}
        </button>
      </div>

      <ProgressView ref="progressRef" />
      <ResultView
        v-if="resultRef"
        :gif-path="resultRef.output_path"
        :message="resultRef.message"
      />
    </main>

    <div v-if="logMessages.length > 0" class="log-box">
      <div v-for="(log, index) in logMessages" :key="index" class="log-item">
        {{ log }}
      </div>
    </div>
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
.generate-btn:hover:not(.disabled) {
  background: #444;
}
.generate-btn.disabled {
  background: #999;
  cursor: not-allowed;
}
.log-box {
  margin-top: 24px;
  padding: 12px;
  background: #1a1a1a;
  border-radius: 8px;
  font-family: "SF Mono", Monaco, monospace;
  font-size: 12px;
  max-height: 200px;
  overflow-y: auto;
}
.log-item {
  color: #888;
  line-height: 1.6;
}
.log-item:last-child {
  color: #ccc;
}
</style>

<style>
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