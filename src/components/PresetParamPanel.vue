<script setup lang="ts">
// 预设 + 参数组合面板
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ControlDef } from "../vite-env";

interface PresetInfo {
  id: string;
  name: string;
}

// 预设顺序（前端控制）
const PRESET_ORDER = ["emoji_make", "clear_demo", "custom_steps"];

const presets = ref<PresetInfo[]>([]);
const selected = ref("");
const controls = ref<ControlDef[]>([]);
const paramValues = ref<Record<string, string | number>>({});
const loadingPresets = ref(true);
const loadingControls = ref(false);
const errorMsg = ref("");
const validateErrorMsg = ref("");

const emit = defineEmits<{
  change: [preset: string];
}>();

defineExpose({ getParams, setValidateError });

function getParams(): Record<string, string | number> {
  return { ...paramValues.value };
}

function setValidateError(msg: string) {
  validateErrorMsg.value = msg;
}

async function loadPresets() {
  try {
    const raw = await invoke<PresetInfo[]>("list_presets", {});
    console.log("[DEBUG] 后端返回:", raw.map(p => p.id));
    console.log("[DEBUG] 前端定义:", PRESET_ORDER);
    errorMsg.value = "";
    // 按前端固定顺序排序
    const ordered: PresetInfo[] = [];
    for (const id of PRESET_ORDER) {
      const found = raw.find(p => p.id === id);
      console.log(`[DEBUG] find "${id}" -> ${found ? found.name : "undefined"}`);
      if (found) {
        ordered.push(found);
      }
    }
    // 追加不在预设列表中的预设
    const known = new Set(PRESET_ORDER);
    for (const p of raw) {
      if (!known.has(p.id)) {
        ordered.push(p);
      }
    }
    presets.value = ordered;
    console.log("[DEBUG] 排序后:", presets.value.map(p => p.id));
    if (presets.value.length > 0) {
      selected.value = presets.value[0].id;
      await loadControls(selected.value);
      emit("change", selected.value);
    }
  } catch (err: unknown) {
    errorMsg.value = `加载预设失败: ${String(err)}`;
    presets.value = [];
  } finally {
    loadingPresets.value = false;
  }
}

async function loadControls(presetName: string) {
  if (!presetName) {
    controls.value = [];
    paramValues.value = {};
    return;
  }
  loadingControls.value = true;
  try {
    const defs = await invoke<ControlDef[]>("get_preset_controls", { presetName });
    controls.value = defs;
    paramValues.value = buildDefaultValues(defs);
    errorMsg.value = "";
  } catch (err: unknown) {
    errorMsg.value = `获取控件失败: ${String(err)}`;
    controls.value = [];
  } finally {
    loadingControls.value = false;
  }
}

function buildDefaultValues(defs: ControlDef[]): Record<string, string | number> {
  const values: Record<string, string | number> = {};
  for (const ctrl of defs) {
    values[ctrl.key] = ctrl.default;
  }
  return values;
}

function onPresetChange() {
  validateErrorMsg.value = "";
  emit("change", selected.value);
}

onMounted(loadPresets);
watch(selected, loadControls);
</script>

<template>
  <section class="panel">
    <div class="preset-row">
      <h3>预设</h3>
      <div v-if="loadingPresets" class="status">加载中...</div>
      <select v-else v-model="selected" @change="onPresetChange">
        <option value="" disabled>选择预设</option>
        <option v-for="p in presets" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
    </div>

    <div v-if="errorMsg" class="error">{{ errorMsg }}</div>

    <div v-if="loadingControls" class="status">加载控件...</div>
    <div v-else-if="controls.length > 0" class="param-section">
      <h4>参数列表</h4>
      <div class="param-list">
        <div v-for="ctrl in controls" :key="ctrl.key" class="param-item">
          <label>{{ ctrl.label }}</label>
          <template v-if="ctrl.type === 'slider'">
            <input type="range" :min="ctrl.min" :max="ctrl.max" :step="1" v-model.number="paramValues[ctrl.key]" />
            <span class="value">{{ paramValues[ctrl.key] }}</span>
          </template>
          <template v-else-if="ctrl.type === 'select'">
            <select v-model="paramValues[ctrl.key]">
              <option v-for="opt in ctrl.values" :key="opt" :value="opt">{{ opt }}</option>
            </select>
          </template>
          <template v-else-if="ctrl.type === 'number'">
            <input type="number" :min="ctrl.min" :max="ctrl.max" v-model.number="paramValues[ctrl.key]" />
          </template>
        </div>
      </div>
    </div>

    <div v-if="validateErrorMsg" class="validate-error">{{ validateErrorMsg }}</div>
  </section>
</template>

<style scoped>
.panel {
  padding: 16px;
  border: 1px solid #ccc;
  border-radius: 8px;
  background: #fafafa;
}
.preset-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}
h3 {
  margin: 0;
  font-size: 14px;
  color: #333;
}
.preset-row h3 {
  min-width: 48px;
}
.preset-row select {
  flex: 1;
  padding: 6px 10px;
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
  margin-bottom: 8px;
}
.param-section {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #e0e0e0;
}
h4 {
  margin: 0 0 10px;
  font-size: 13px;
  color: #666;
}
.param-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
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
.value {
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
.validate-error {
  color: #e53e3e;
  font-size: 13px;
  font-weight: 600;
  margin-top: 8px;
  padding: 8px 12px;
  background: #fff5f5;
  border: 1px solid #fed7d7;
  border-radius: 4px;
}
</style>