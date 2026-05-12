<script setup lang="ts">
// 结果展示组件：GIF 预览 + 导出按钮
import { ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { dirname } from "@tauri-apps/api/path";
import { save } from "@tauri-apps/plugin-dialog";
import { readFile, writeFile } from "@tauri-apps/plugin-fs";

const props = defineProps<{
  gifPath: string;
  message?: string;
}>();

const emit = defineEmits<{
  exported: [path: string];
}>();

const exporting = ref(false);
const imageLoaded = ref(false);
const imageError = ref(false);
const cacheBustUrl = ref("");

// gifPath 变化时重置加载状态，并加时间戳破缓存
watch(() => props.gifPath, (newPath) => {
  imageLoaded.value = false;
  imageError.value = false;
  if (newPath) {
    cacheBustUrl.value = `${getPreviewUrl(newPath)}?t=${Date.now()}`;
  }
}, { immediate: true });

function getPreviewUrl(path: string): string {
  return convertFileSrc(path);
}

async function onOpenFolder() {
  if (!props.gifPath) return;
  const dir = await dirname(props.gifPath);
  openPath(dir);
}

async function onExport() {
  if (!props.gifPath || exporting.value) return;
  exporting.value = true;

  try {
    const savePath = await save({
      filters: [{ name: "GIF", extensions: ["gif"] }],
      defaultPath: "output.gif",
    });
    if (!savePath) {
      exporting.value = false;
      return;
    }

    // 读取临时文件，写入用户选择的位置
    const data = await readFile(props.gifPath);
    await writeFile(savePath, data);

    emit("exported", savePath);
  } catch (err) {
    console.error("导出失败:", err);
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <section v-if="gifPath" class="panel">
    <h3>预览</h3>
    <div class="preview">
      <div v-if="!imageLoaded && !imageError" class="preview-loading">加载中...</div>
      <img
        v-show="imageLoaded"
        :src="cacheBustUrl"
        alt="GIF 预览"
        @load="imageLoaded = true"
        @error="imageError = true"
      />
      <div v-if="imageError" class="preview-error">预览加载失败</div>
    </div>
    <p v-if="message" class="result-message">{{ message }}</p>
    <div class="actions">
      <button type="button" class="export-btn" :disabled="exporting" @click="onExport">
        {{ exporting ? "导出中..." : "导出" }}
      </button>
      <button type="button" class="folder-btn" @click="onOpenFolder">打开缓存目录</button>
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
.preview {
  margin-bottom: 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  overflow: hidden;
  background: #eee;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 120px;
}
.preview img {
  max-width: 100%;
  display: block;
}
.preview-loading {
  padding: 20px;
  color: #999;
  font-size: 13px;
  text-align: center;
}
.preview-error {
  padding: 20px;
  color: #c0392b;
  font-size: 13px;
  text-align: center;
}
.result-message {
  margin: 8px 0;
  padding: 8px 12px;
  background: #e8f5e9;
  border-radius: 6px;
  font-size: 13px;
  color: #2e7d32;
}
.actions {
  display: flex;
  gap: 8px;
}
.export-btn {
  padding: 8px 20px;
  background: #222;
  color: #fff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
}
.export-btn:hover:not(:disabled) {
  background: #444;
}
.export-btn:disabled {
  background: #999;
  cursor: not-allowed;
}
.folder-btn {
  padding: 8px 16px;
  background: #666;
  color: #fff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}
.folder-btn:hover {
  background: #555;
}
</style>
