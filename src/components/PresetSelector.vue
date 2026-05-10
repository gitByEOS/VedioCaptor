<script setup lang="ts">
// 预设选择组件：从 Tauri 获取真实预设列表
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const presets = ref<string[]>([]);
const selected = ref("");
const loading = ref(true);
const errorMsg = ref("");

const emit = defineEmits<{
  change: [preset: string];
}>();

async function loadPresets() {
  try {
    const list = await invoke<string[]>("list_presets", {});
    presets.value = list;
    errorMsg.value = "";
  } catch (err: unknown) {
    errorMsg.value = `加载预设失败: ${String(err)}`;
    presets.value = [];
  } finally {
    loading.value = false;
  }
}

function onChange() {
  emit("change", selected.value);
}

onMounted(loadPresets);
</script>

<template>
  <section class="panel">
    <h3>预设选择</h3>
    <div v-if="loading" class="status">加载中...</div>
    <div v-else-if="errorMsg" class="error">{{ errorMsg }}</div>
    <select v-else v-model="selected" @change="onChange">
      <option value="" disabled>请选择 Lua 预设</option>
      <option v-for="preset in presets" :key="preset" :value="preset">
        {{ preset }}
      </option>
    </select>
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
select {
  width: 100%;
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 6px;
  font-size: 14px;
  background: #fff;
}
.status {
  color: #999;
  font-size: 13px;
}
.error {
  color: #e53e3e;
  font-size: 13px;
}
</style>
