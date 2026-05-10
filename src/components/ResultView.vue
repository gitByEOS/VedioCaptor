<script setup lang="ts">
// 结果展示组件：GIF 预览 + 打开文件夹 + 自定义消息
import { openPath } from "@tauri-apps/plugin-opener";
import { dirname } from "@tauri-apps/api/path";

const props = defineProps<{
  gifPath: string;
  message?: string;
}>();

async function onOpenFolder() {
  if (!props.gifPath) return;
  const dir = await dirname(props.gifPath);
  openPath(dir);
}

function toFileUrl(path: string): string {
  return `file://${path}`;
}
</script>

<template>
  <section v-if="gifPath" class="panel">
    <h3>结果</h3>
    <div class="preview">
      <img :src="toFileUrl(gifPath)" alt="生成的 GIF 预览" />
    </div>
    <p v-if="message" class="result-message">{{ message }}</p>
    <p class="path-text" :title="gifPath">{{ gifPath }}</p>
    <button type="button" class="folder-btn" @click="onOpenFolder">打开文件夹</button>
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
.result-message {
  margin: 8px 0;
  padding: 8px 12px;
  background: #e8f5e9;
  border-radius: 6px;
  font-size: 13px;
  color: #2e7d32;
}
.path-text {
  margin: 4px 0 12px;
  font-size: 12px;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.folder-btn {
  padding: 8px 16px;
  background: #333;
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
