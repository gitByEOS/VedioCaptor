<script setup lang="ts">
import { ref } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

const filePath = ref("");
const videoSrc = ref("");
const isDragging = ref(false);
const duration = ref(0);
const videoRef = ref<HTMLVideoElement | null>(null);
const currentTime = ref(0);
const rangeStart = ref(0);
const rangeEnd = ref(0);

const emit = defineEmits<{
  (e: "duration", value: number): void;
  (e: "time-update", value: number): void;
}>();

async function handleFileSelected(path: string) {
  filePath.value = path;
  const assetUrl = convertFileSrc(path);
  videoSrc.value = assetUrl;
}

function onVideoLoaded(e: Event) {
  const video = e.target as HTMLVideoElement;
  duration.value = video.duration;
  emit("duration", Math.floor(video.duration));
}

function onTimeUpdate(e: Event) {
  const video = e.target as HTMLVideoElement;
  currentTime.value = video.currentTime;
  emit("time-update", video.currentTime);

  // 播放超出结束时间时跳回开始时间循环
  if (rangeEnd.value > 0 && video.currentTime >= rangeEnd.value) {
    video.currentTime = rangeStart.value;
  }
}

function playRange(start: number, end: number) {
  rangeStart.value = start;
  rangeEnd.value = end;
  if (videoRef.value) {
    videoRef.value.currentTime = start;
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

function onDragOver(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}

function onDragLeave(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
}

async function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;

  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    const validExtensions = ["mp4", "mkv", "avi", "webm", "mov"];
    const ext = file.name.split(".").pop()?.toLowerCase();
    if (ext && validExtensions.includes(ext)) {
      const path = (file as any).path || file.name;
      await handleFileSelected(path);
    }
  }
}

defineExpose({ filePath, duration, playRange, currentTime });
</script>

<template>
  <section class="panel">
    <div
      class="preview-area"
      :class="{ 'dragging': isDragging, 'has-preview': videoSrc }"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
      @click="onSelectFile"
    >
      <video
        v-if="videoSrc"
        ref="videoRef"
        :src="videoSrc"
        class="preview-video"
        muted
        autoplay
        loop
        @loadedmetadata="onVideoLoaded"
        @timeupdate="onTimeUpdate"
      />
      <div v-else class="preview-placeholder">
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

.preview-area.dragging {
  border-color: #333;
  background: #d0d0d0;
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
</style>
