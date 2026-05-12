<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { listen } from "@tauri-apps/api/event";
import { appCacheDir, join } from "@tauri-apps/api/path";
import { mkdir } from "@tauri-apps/plugin-fs";
import { validateParams, executeConversion, type ConversionResult } from "./api";
import { isValidTimeRange } from "./utils";
import FileSelector from "./components/FileSelector.vue";
import TimeRangeSlider from "./components/TimeRangeSlider.vue";
import PresetParamPanel from "./components/PresetParamPanel.vue";
import ProgressView from "./components/ProgressView.vue";
import ResultView from "./components/ResultView.vue";

type AppStatus = "idle" | "validating" | "converting" | "done" | "error";

const selectedPreset = ref("表情制作");
const videoDuration = ref(60);
const currentPlayTime = ref(0);
const status = ref<AppStatus>("idle");
const errorInfo = ref("");
const fileSelectorRef = ref<InstanceType<typeof FileSelector> | null>(null);
const timeSliderRef = ref<InstanceType<typeof TimeRangeSlider> | null>(null);
const presetParamRef = ref<InstanceType<typeof PresetParamPanel> | null>(null);
const progressRef = ref<InstanceType<typeof ProgressView> | null>(null);
const resultRef = ref<ConversionResult | null>(null);
const logMessages = ref<string[]>([]);
const previewPath = ref("");
let taskStartTime = 0;

let unlisten: (() => void) | null = null;

const isConverting = computed(() => status.value === "converting" || status.value === "validating");

function addLog(msg: string) {
  const elapsed = Math.floor((Date.now() - taskStartTime) / 1000);
  const h = Math.floor(elapsed / 3600);
  const m = Math.floor((elapsed % 3600) / 60);
  const s = elapsed % 60;
  const timestamp = [h, m, s].map(v => String(v).padStart(2, "0")).join(":");
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

// 新 UI 出现时自动滚动到底部
watch(() => [resultRef.value, logMessages.value.length], async () => {
  await nextTick();
  window.scrollTo({ top: document.body.scrollHeight, behavior: "smooth" });
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

function onVideoDuration(duration: number) {
  videoDuration.value = duration;
}

function onTimeUpdate(time: number) {
  currentPlayTime.value = time;
}

function onRangeChange(start: number, end: number) {
  fileSelectorRef.value?.playRange(start, end);
}

function collectForm() {
  const file = fileSelectorRef.value?.filePath ?? "";
  const timeRange = timeSliderRef.value?.getRange() ?? { start: "00:00:00", end: "00:00:10" };
  const params = presetParamRef.value?.getParams() ?? {};
  return { file, start: timeRange.start, end: timeRange.end, params };
}

function showValidation(msg: string) {
  errorInfo.value = msg;
  addLog(`错误: ${msg}`);
  presetParamRef.value?.setValidateError(msg);
  setStatus("error");
}

async function onGenerate() {
  errorInfo.value = "";
  resultRef.value = null;
  logMessages.value = [];
  taskStartTime = Date.now();

  // 初始化预览路径
  const cacheDirPath = await appCacheDir();
  previewPath.value = await join(cacheDirPath, "preview.gif");
  addLog(`预览路径: ${previewPath.value}`);

  // 确保缓存目录存在
  try {
    await mkdir(cacheDirPath, { recursive: true });
  } catch {
    // 目录已存在，忽略
  }

  const { file, start, end, params } = collectForm();

  if (!file) {
    showValidation("请先选择视频文件");
    return;
  }

  if (!isValidTimeRange(start, end)) {
    showValidation("结束时间必须大于起始时间");
    return;
  }

  // 打印任务详情
  addLog(`文件: ${file}`);
  addLog(`时间: ${start} → ${end}`);
  addLog(`预设: ${selectedPreset.value}`);
  addLog(`参数: ${JSON.stringify(params)}`);

  setStatus("validating");

  const validateResult = await validateParams(selectedPreset.value, params, file);
  if (!validateResult.ok) {
    showValidation(validateResult.error ?? "参数校验失败");
    return;
  }

  setStatus("converting");
  progressRef.value?.resetProgress();

  const presetName = selectedPreset.value;

  try {
    const conversionResult = await executeConversion(
      presetName, params, file, start, end, previewPath.value,
    );
    resultRef.value = { output_path: previewPath.value, message: conversionResult.message };
    progressRef.value?.markComplete();
    setStatus("done");
  } catch (err) {
    errorInfo.value = `转换失败: ${err}`;
    addLog(`转换失败: ${err}`);
    progressRef.value?.markComplete();
    setStatus("error");
  }
}

function onExported(savePath: string) {
  addLog(`已导出: ${savePath}`);
}
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>VideoCaptor</h1>
    </header>

    <main class="main">
      <FileSelector ref="fileSelectorRef" @duration="onVideoDuration" @time-update="onTimeUpdate" />
      <TimeRangeSlider ref="timeSliderRef" :总时长秒="videoDuration" :当前播放秒="currentPlayTime" @range-change="onRangeChange" />
      <PresetParamPanel ref="presetParamRef" @change="onPresetChange" />

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

      <ProgressView
      v-if="logMessages.length > 0 || isConverting"
      ref="progressRef"
      :log-messages="logMessages"
    />
      <ResultView
        v-if="resultRef"
        :gif-path="previewPath"
        :message="resultRef.message"
        @exported="onExported"
      />
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
.generate-btn:hover:not(.disabled) {
  background: #444;
}
.generate-btn.disabled {
  background: #999;
  cursor: not-allowed;
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