<script setup lang="ts">
// 文件选择组件：路径输入 + 起止时间
import { ref } from "vue";

const filePath = ref("");
const startTime = ref("00:00:00");
const endTime = ref("00:00:10");

function onSelectFile() {
  console.log("占位：触发 Tauri 文件选择器");
}

defineExpose({ filePath, startTime, endTime });
</script>

<template>
  <section class="panel">
    <h3>文件选择</h3>
    <div class="file-row">
      <input
        v-model="filePath"
        type="text"
        placeholder="视频文件路径"
        class="file-input"
      />
      <button type="button" @click="onSelectFile">选择文件</button>
    </div>
    <div class="time-row">
      <label>
        起始时间
        <input v-model="startTime" type="text" placeholder="00:00:00" />
      </label>
      <label>
        结束时间
        <input v-model="endTime" type="text" placeholder="00:00:10" />
      </label>
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
.file-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.file-input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 6px;
  font-size: 14px;
}
.time-row {
  display: flex;
  gap: 16px;
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
</style>
