<script setup lang="ts">
import { ref, computed } from "vue";
import { timeToSeconds, isValidTimeRange, secondsToTime } from "../utils";

import { open } from "@tauri-apps/plugin-dialog";

const filePath = ref("");
const startTime = ref("00:00:00");
const endTime = ref("00:00:10");
const duration = ref("");
const errorMsg = ref("");

const durationSeconds = computed(() => duration.value ? timeToSeconds(duration.value) : 0);

async function onSelectFile() {
  errorMsg.value = "";
  try {
    const selected = await open({
      filters: [{ name: "视频", extensions: ["mp4", "mkv", "avi", "webm", "mov"] }],
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      filePath.value = selected;
      endTime.value = "00:00:10";
    } else if (selected && typeof selected === "object" && "path" in selected) {
      filePath.value = selected.path;
      endTime.value = "00:00:10";
    }
  } catch (err) {
    console.error("文件选择失败:", err);
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

defineExpose({ filePath, startTime, endTime, validateTime });
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
.file-row button {
  padding: 8px 16px;
  background: #333;
  color: #fff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}
.file-row button:hover {
  background: #555;
}
.duration-info {
  font-size: 12px;
  color: #666;
  margin-bottom: 8px;
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
.time-error {
  color: #e53e3e;
  font-size: 12px;
  margin-top: 8px;
}
</style>
