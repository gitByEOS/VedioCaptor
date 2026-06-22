<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { prepareVideoPreview, type VideoInfo } from "../api";
import CropOverlay from "./CropOverlay.vue";

interface CropRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

const MAX_PREVIEW_SECONDS = 12;

const filePath = ref("");
const videoSrc = ref("");
const duration = ref(0);
const isVideoReady = ref(false);
const isPreparingPreview = ref(false);
const videoError = ref("");
const sourceVideoInfo = ref<VideoInfo>({ duration: 0, width: 0, height: 0 });
const videoRef = ref<HTMLVideoElement | null>(null);
const currentTime = ref(0);
const rangeStart = ref(0);
const rangeEnd = ref(0);
const previewStart = ref(0);
const previewEnd = ref(0);
const cropRegion = ref<CropRegion | null>(null);
let previewRequestId = 0;

const emit = defineEmits<{
  (e: "duration", value: number): void;
  (e: "time-update", value: number): void;
  (e: "video-info", value: VideoInfo): void;
}>();

async function playVideo() {
  const video = videoRef.value;
  if (!video) return;

  try {
    await video.play();
  } catch (err) {
    console.warn("视频自动播放失败:", err);
  }
}

function resetPreviewState() {
  cropRegion.value = null;
  duration.value = 0;
  currentTime.value = 0;
  rangeStart.value = 0;
  rangeEnd.value = 0;
  previewStart.value = 0;
  previewEnd.value = 0;
  isVideoReady.value = false;
  videoError.value = "";
  sourceVideoInfo.value = { duration: 0, width: 0, height: 0 };
}

async function prepareRangePreview(start: number, end: number) {
  if (!filePath.value) return;

  const requestId = ++previewRequestId;
  const safeStart = Math.max(0, start);
  const safeEnd = Math.min(Math.max(safeStart + 1, end), safeStart + MAX_PREVIEW_SECONDS);
  isVideoReady.value = false;
  isPreparingPreview.value = true;
  videoError.value = "";
  try {
    const preview = await prepareVideoPreview(filePath.value, safeStart, safeEnd);
    if (requestId !== previewRequestId) return;

    sourceVideoInfo.value = preview.video_info;
    duration.value = preview.video_info.duration;
    previewStart.value = safeStart;
    previewEnd.value = safeEnd;
    rangeStart.value = safeStart;
    rangeEnd.value = safeEnd;
    currentTime.value = safeStart;
    emit("duration", Math.floor(preview.video_info.duration));
    emit("video-info", preview.video_info);
    videoSrc.value = `${convertFileSrc(preview.preview_path)}?t=${Date.now()}`;
  } catch (err) {
    if (requestId !== previewRequestId) return;
    videoError.value = "生成预览失败";
    console.error("生成预览失败:", err);
  } finally {
    if (requestId === previewRequestId) {
      isPreparingPreview.value = false;
    }
  }

  await nextTick();
  videoRef.value?.load();
  await playVideo();
}

async function handleFileSelected(path: string) {
  filePath.value = path;
  resetPreviewState();
  emit("video-info", { duration: 0, width: 0, height: 0 });
  await prepareRangePreview(0, 10);
}

function onVideoLoaded() {
  isVideoReady.value = true;
  videoError.value = "";
  void playVideo();
}

function onVideoError(e: Event) {
  const video = e.target as HTMLVideoElement;
  isVideoReady.value = false;
  const code = video.error?.code;
  const reason = code === 2
    ? "本地文件读取失败"
    : code === 3
      ? "视频解码失败"
      : code === 4
        ? "视频格式不支持"
        : "视频加载失败";
  videoError.value = reason;
  console.error("视频加载失败:", video.error, video.currentSrc);
}

function onTimeUpdate(e: Event) {
  const video = e.target as HTMLVideoElement;
  const sourceTime = previewStart.value + video.currentTime;
  currentTime.value = sourceTime;
  emit("time-update", sourceTime);

  const previewDuration = Math.max(0, previewEnd.value - previewStart.value);
  if (previewDuration > 0 && video.currentTime >= previewDuration) {
    video.currentTime = 0;
  }
}

function playRange(start: number, end: number) {
  rangeStart.value = start;
  rangeEnd.value = end;
  if (videoRef.value) {
    videoRef.value.currentTime = Math.max(0, start - previewStart.value);
    videoRef.value.play();
  }
}

async function onSelectFile() {
  try {
    const selected = await open({
      filters: [{ name: "视频", extensions: ["mp4", "mkv", "avi", "webm", "mov"] }],
      multiple: false,
    });
    if (typeof selected === "string") {
      await handleFileSelected(selected);
    }
  } catch (err) {
    console.error("文件选择失败:", err);
  }
}

function onCropChange(c: CropRegion | null) {
  const video = videoRef.value;
  const source = sourceVideoInfo.value;
  if (!c || !video || !video.videoWidth || !source.width || !source.height) {
    cropRegion.value = c;
    return;
  }

  const scaleX = source.width / video.videoWidth;
  const scaleY = source.height / video.videoHeight;
  cropRegion.value = {
    x: Math.round(c.x * scaleX),
    y: Math.round(c.y * scaleY),
    width: Math.round(c.width * scaleX),
    height: Math.round(c.height * scaleY),
  };
}

function onChangeVideo() {
  onSelectFile();
}

// 监听 Rust 层 file-dropped 事件

onMounted(async () => {
  await listen<string>("file-dropped", (event) => {
    handleFileSelected(event.payload);
  });
});

defineExpose({ filePath, duration, playRange, prepareRangePreview, currentTime, cropRegion });
</script>

<template>
  <section class="panel">
    <div class="preview-area" :class="{ 'has-preview': videoSrc }" @click="!videoSrc && onSelectFile()">
      <video
        v-if="videoSrc"
        :key="videoSrc"
        ref="videoRef"
        :src="videoSrc"
        class="preview-video"
        muted
        autoplay
        playsinline
        loop
        preload="auto"
        @loadedmetadata="onVideoLoaded"
        @timeupdate="onTimeUpdate"
        @error="onVideoError"
      />
      <CropOverlay
        v-if="isVideoReady"
        :video-ref="videoRef"
        @crop-change="onCropChange"
      />
      <button v-if="videoSrc" class="change-video-btn" @click.stop="onChangeVideo">更换视频</button>
      <div v-if="isPreparingPreview" class="preview-loading">生成预览中...</div>
      <div v-if="videoError" class="video-error">{{ videoError }}</div>
      <div v-if="!videoSrc" class="preview-placeholder">
        <span>选择文件或拖拽视频到此处</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.panel {
  padding: 0;
  border: none;
  background: transparent;
}

.preview-area {
  position: relative;
  width: 100%;
  padding-bottom: 56.25%;
  background: #e8e8e8;
  border: 2px dashed #ccc;
  border-radius: 8px;
  cursor: pointer;
  overflow: hidden;
}

.preview-area:hover {
  border-color: #999;
}

.preview-area.has-preview {
  border-style: solid;
  border-color: #333;
}

.preview-video {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #000;
}

.preview-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  font-size: 14px;
}

.preview-loading {
  position: absolute;
  inset: 0;
  z-index: 5;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #666;
  background: #f8f8f8;
  font-size: 13px;
}

.change-video-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
}

.change-video-btn:hover {
  background: rgba(0, 0, 0, 0.8);
}

.video-error {
  position: absolute;
  inset: 0;
  z-index: 5;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #c0392b;
  background: #f8f8f8;
  font-size: 13px;
}
</style>
