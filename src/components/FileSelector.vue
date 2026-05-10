<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { timeToSeconds, isValidTimeRange, secondsToTime } from "../utils";

import { open } from "@tauri-apps/plugin-dialog";

const filePath = ref("");
const startTime = ref("00:00:00");
const endTime = ref("00:00:10");
const duration = ref("");
const errorMsg = ref("");
const previewUrl = ref("");
const isDragging = ref(false);

const durationSeconds = computed(() => duration.value ? timeToSeconds(duration.value) : 0);

// 截取视频第一帧
async function captureFirstFrame(videoPath: string) {
  const video = document.createElement("video");
  video.crossOrigin = "anonymous";
  video.preload = "metadata";

  // Tauri 环境下需要转换为可访问的 URL
  const assetUrl = await import("@tauri-apps/api/core").then(m => m.convertFileSrc(videoPath));
  video.src = assetUrl;

  return new Promise<string>((resolve) => {
    video.onloadeddata = () => {
      video.currentTime = 0.1;
    };
    video.onseeked = () => {
      const canvas = document.createElement("canvas");
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      const ctx = canvas.getContext("2d");
      if (ctx) {
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
        resolve(canvas.toDataURL("image/jpeg", 0.8));
      } else {
        resolve("");
      }
    };
    video.onerror = () => resolve("");
  });
}

// 处理文件选择
async function handleFileSelected(path: string) {
  filePath.value = path;
  endTime.value = "00:00:10";
  previewUrl.value = await captureFirstFrame(path);
}

async function onSelectFile() {
  errorMsg.value = "";
  try {
    const selected = await open({
      filters: [{ name: "视频", extensions: ["mp4", "mkv", "avi", "webm", "mov"] }],
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      await handleFileSelected(selected);
    } else if (selected && typeof selected === "object" && "path" in selected) {
      await handleFileSelected(selected.path);
    }
  } catch (err) {
    console.error("文件选择失败:", err);
  }
}

// 拖拽处理
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
      // Tauri 环境需要获取文件路径
      const path = (file as any).path || file.name;
      await handleFileSelected(path);
    }
  }
}

function validateTime(): boolean {
  if (!isValidTimeRange(startTime.value, endTime.value)) {
    errorMsg.value = "结束时间必须大于起始时间";
    return false;
  }
  errorMsg.value = "";
  return true;
}

function onStartTimeChange() {
  if (durationSeconds.value && timeToSeconds(startTime.value) >= durationSeconds.value) {
    errorMsg.value = "起始时间不能超过视频总时长";
  } else {
    validateTime();
  }
}

function onEndTimeChange() {
  if (durationSeconds.value && timeToSeconds(endTime.value) > durationSeconds.value) {
    endTime.value = secondsToTime(durationSeconds.value);
  }
  validateTime();
}

// 监听 duration 变化，外部组件设置时长
watch(duration, () => {});

defineExpose({ filePath, startTime, endTime, duration, validateTime });
</script>

<template>
  <section class="panel">
    <h3>文件选择</h3>

    <!-- 预览区域 -->
    <div
      class="preview-area"
      :class="{ 'dragging': isDragging }"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
      @click="onSelectFile"
    >
      <img v-if="previewUrl" :src="previewUrl" class="preview-image" />
      <div v-else class="preview-placeholder">
        <span>选择文件或拖拽视频到此处</span>
      </div>
    </div>

    <!-- 文件路径显示 -->
    <div class="file-info">
      <span class="file-path" :title="filePath">{{ filePath || '未选择文件' }}</span>
      <button type="button" class="select-btn" @click="onSelectFile">选择文件</button>
    </div>

    <div v-if="duration" class="duration-info">
      视频时长: {{ duration }}
    </div>

    <div class="time-row">
      <label>
        起始时间
        <input
          v-model="startTime"
          type="text"
          placeholder="HH:MM:SS"
          @input="onStartTimeChange"
        />
      </label>
      <label>
        结束时间
        <input
          v-model="endTime"
          type="text"
          placeholder="HH:MM:SS"
          @input="onEndTimeChange"
        />
      </label>
    </div>

    <div v-if="errorMsg" class="time-error">{{ errorMsg }}</div>
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

/* 预览区域 */
.preview-area {
  position: relative;
  width: 100%;
  padding-bottom: 56.25%; /* 16:9 */
  background: #e8e8e8;
  border: 2px dashed #ccc;
  border-radius: 8px;
  cursor: pointer;
  overflow: hidden;
  transition: border-color 0.2s, background-color 0.2s;
}
.preview-area:hover {
  border-color: #999;
}
.preview-area.dragging {
  border-color: #333;
  background: #d0d0d0;
}
.preview-image {
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

/* 文件信息 */
.file-info {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  align-items: center;
}
.file-path {
  flex: 1;
  padding: 8px;
  background: #fff;
  border: 1px solid #ccc;
  border-radius: 6px;
  font-size: 13px;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.select-btn {
  padding: 8px 16px;
  background: #333;
  color: #fff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}
.select-btn:hover {
  background: #555;
}

.duration-info {
  font-size: 12px;
  color: #666;
  margin-top: 8px;
}
.time-row {
  display: flex;
  gap: 16px;
  margin-top: 12px;
}
.time-row label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: #666;
}
.time-row input {
  padding: 6px 8px;
  border: 1px solid #ccc;
  border-radius: 6px;
  font-size: 13px;
  width: 120px;
}
.time-error {
  color: #e53e3e;
  font-size: 12px;
  margin-top: 8px;
}
</style>
