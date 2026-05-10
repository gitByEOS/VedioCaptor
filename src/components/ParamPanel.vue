<script setup lang="ts">
// 动态参数面板：根据 Tauri 返回的 ControlDef 渲染控件
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ControlDef } from "../vite-env";

const props = defineProps<{
  preset: string;
}>();

const controls = ref<ControlDef[]>([]);
const paramValues = ref<Record<string, string | number>>({});
const loading = ref(false);
const errorMsg = ref("");

async function loadControls(presetName: string) {
  if (!presetName) {
    controls.value = [];
    paramValues.value = {};
    return;
  }
  loading.value = true;
  errorMsg.value = "";
  try {
    const path = `presets/${presetName}.lua`;
    const defs = await invoke<ControlDef[]>("get_preset_controls", { presetPath: path });
    controls.value = defs;
    paramValues.value = buildDefaultValues(defs);
  } catch (err: unknown) {
    errorMsg.value = `获取控件失败: ${String(err)}`;
    controls.value = [];
  } finally {
    loading.value = false;
  }
}

// 从控件定义中提取默认值
function buildDefaultValues(defs: ControlDef[]): Record<string, string | number> {
  const values: Record<string, string | number> = {};
  for (const ctrl of defs) {
    values[ctrl.label] = ctrl.default;
  }
  return values;
}

// 对外暴露当前参数值
function getParams(): Record<string, string | number> {
  return { ...paramValues.value };
}

defineExpose({ getParams });

watch(() => props.preset, loadControls);
</script>

<template>
  <section class="panel">
    <h3>参数面板</h3>
    <div v-if="loading" class="status">加载控件中...</div>
    <div v-else-if="errorMsg" class="error">{{ errorMsg }}</div>
    <div v-else-if="controls.length === 0" class="empty">请选择预设以显示参数</div>
    <div v-else class="param-list">
      <div v-for="ctrl in controls" :key="ctrl.label" class="param-item">
        <label>{{ ctrl.label }}</label>

        <!-- Slider 控件 -->
        <template v-if="ctrl.type === 'slider'">
          <input
            type="range"
            :min="ctrl.min"
            :max="ctrl.max"
            :step="1"
            v-model.number="paramValues[ctrl.label]"
          />
          <span class="value-display">{{ paramValues[ctrl.label] }}</span>
        </template>

        <!-- Select 控件 -->
        <template v-else-if="ctrl.type === 'select'">
          <select v-model="paramValues[ctrl.label]">
            <option v-for="opt in ctrl.values" :key="opt" :value="opt">
              {{ opt }}
            </option>
          </select>
        </template>

        <!-- Number 控件 -->
        <template v-else-if="ctrl.type === 'number'">
          <input
            type="number"
            :min="ctrl.min"
            :max="ctrl.max"
            v-model.number="paramValues[ctrl.label]"
          />
        </template>
      </div>
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
.status, .empty {
  color: #999;
  font-size: 13px;
}
.error {
  color: #e53e3e;
  font-size: 13px;
}
.param-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.param-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.param-item label {
  font-size: 13px;
  color: #555;
  min-width: 80px;
}
.param-item input[type="range"] {
  flex: 1;
}
.value-display {
  font-size: 13px;
  color: #333;
  min-width: 36px;
  text-align: right;
}
.param-item select {
  padding: 4px 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 13px;
}
.param-item input[type="number"] {
  width: 80px;
  padding: 4px 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 13px;
}
</style>
