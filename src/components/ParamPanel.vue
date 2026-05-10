<script setup lang="ts">
// 动态参数面板：根据预设渲染参数控件（假数据）
import { ref, watch } from "vue";

const props = defineProps<{
  preset: string;
}>();

// 假参数配置
const paramConfig = ref<{ label: string; type: string; options?: string[] }[]>([]);
const paramValues = ref<Record<string, string | number>>({});

// 根据预设切换假参数
watch(
  () => props.preset,
  (preset) => {
    if (preset === "表情小文件") {
      paramConfig.value = [
        { label: "缩放比例", type: "slider" },
        { label: "输出格式", type: "select", options: ["gif", "webp", "mp4"] },
      ];
    } else if (preset === "清晰演示") {
      paramConfig.value = [
        { label: "帧率", type: "slider" },
        { label: "画质", type: "select", options: ["高", "中", "低"] },
      ];
    } else if (preset === "自定义步骤") {
      paramConfig.value = [
        { label: "亮度", type: "slider" },
        { label: "对比度", type: "slider" },
      ];
    } else {
      paramConfig.value = [];
    }
    paramValues.value = {};
  },
  { immediate: true }
);
</script>

<template>
  <section class="panel">
    <h3>参数面板</h3>
    <div v-if="paramConfig.length === 0" class="empty">请选择预设以显示参数</div>
    <div v-else class="param-list">
      <div v-for="param in paramConfig" :key="param.label" class="param-item">
        <label>{{ param.label }}</label>
        <input
          v-if="param.type === 'slider'"
          type="range"
          min="0"
          max="100"
          v-model.number="paramValues[param.label]"
        />
        <select
          v-else-if="param.type === 'select'"
          v-model="paramValues[param.label]"
        >
          <option v-for="opt in param.options" :key="opt" :value="opt">
            {{ opt }}
          </option>
        </select>
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
.empty {
  color: #999;
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
.param-item select {
  padding: 4px 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 13px;
}
</style>
