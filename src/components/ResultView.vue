<script setup lang="ts">
// 结果展示组件：GIF 预览 + 打开文件夹
function onOpenFolder() {
  console.log("占位：触发 Tauri 打开输出文件夹");
}
</script>

<template>
  <section class="panel">
    <h3>结果</h3>
    <div class="preview">
      <img
        src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='320' height='180'%3E%3Crect fill='%23ddd' width='320' height='180'/%3E%3Ctext x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle' fill='%23999' font-size='14'%3EGIF 预览占位%3C/text%3E%3C/svg%3E"
        alt="生成的 GIF 预览"
      />
    </div>
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
