<script setup lang="ts">
import { ref } from "vue";

import { open } from "@tauri-apps/plugin-dialog";

const filePath = ref("");
const previewUrl = ref("");
const isDragging = ref(false);

// 截取视频第一帧
async function captureFirstFrame(videoPath: string) {
  const video = document.createElement("video");
  video.crossOrigin = "anonymous";
  video.preload = "metadata";

  // Tauri 环境下需要转换为可访问的 URL
  const { convertFileSrc } = await import("@tauri-apps/api/core");
  const assetUrl = convertFileSrc(videoPath);
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
    video.onerror = () => {
      console.error("视频加载失败:", videoPath);
      resolve("");
    };
  });
}

// 处理文件选择
async function handleFileSelected(path: string) {
  filePath.value = path;
  previewUrl.value = await captureFirstFrame(path);
}

async function onSelectFile() {
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

defineExpose({ filePath });
</script>

<template>
  <section class="panel">
    <!-- 预览区域 -->
    <div
      class="preview-area"
      :class="{ 'dragging': isDragging, 'has-preview': previewUrl }"
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
  </section>
</template>

<style scoped>
.panel {
  padding: 0;
  border: none;
  background: transparent;
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
.preview-area.has-preview {
  border-style: solid;
  border-color: #333;
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
</style>
