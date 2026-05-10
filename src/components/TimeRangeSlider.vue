<script setup lang="ts">
// 双滑块时间范围选择器：左滑块控制起始时间，右滑块控制结束时间
import { computed, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    总时长秒?: number;
  }>(),
  { 总时长秒: 3600 }
);

const 开始秒 = ref(0);
const 结束秒 = ref(props.总时长秒);

watch(
  () => props.总时长秒,
  (newVal) => {
    结束秒.value = newVal;
  }
);

// 格式化秒数为 HH:MM:SS
function 格式化时间(秒数: number): string {
  const h = Math.floor(秒数 / 3600);
  const m = Math.floor((秒数 % 3600) / 60);
  const s = 秒数 % 60;
  return [h, m, s].map((v) => String(v).padStart(2, "0")).join(":");
}

const 开始时间显示 = computed(() => 格式化时间(开始秒.value));
const 结束时间显示 = computed(() => 格式化时间(结束秒.value));
const 总时长显示 = computed(() => 格式化时间(props.总时长秒));

// 滑块百分比位置
const 开始百分比 = computed(() => (开始秒.value / props.总时长秒) * 100);
const 结束百分比 = computed(() => (结束秒.value / props.总时长秒) * 100);

// 限制范围：左滑块不能超过右滑块，右滑块不能小于左滑块
function 更新开始(value: number) {
  开始秒.value = Math.min(value, 结束秒.value - 1);
}

function 更新结束(value: number) {
  结束秒.value = Math.max(value, 开始秒.value + 1);
}

// 对外暴露当前范围
function getRange(): { 开始秒: number; 结束秒: number } {
  return { 开始秒: 开始秒.value, 结束秒: 结束秒.value };
}

function setRange(开始: number, 结束: number) {
  开始秒.value = Math.max(0, Math.min(开始, 结束 - 1));
  结束秒.value = Math.min(props.总时长秒, Math.max(结束, 开始秒.value + 1));
}

defineExpose({ getRange, setRange, 开始秒, 结束秒 });
</script>

<template>
  <section class="slider-panel">
    <h3>时间范围</h3>
    <div class="time-display">
      <span class="time-start">{{ 开始时间显示 }}</span>
      <span class="time-separator">—</span>
      <span class="time-end">{{ 结束时间显示 }}</span>
      <span class="time-total">/ {{ 总时长显示 }}</span>
    </div>
    <div class="slider-container">
      <!-- 中间选中区域高亮 -->
      <div
        class="range-highlight"
        :style="{
          left: 开始百分比 + '%',
          width: 结束百分比 - 开始百分比 + '%',
        }"
      />
      <!-- 左滑块：起始时间 -->
      <input
        type="range"
        class="slider slider-left"
        :min="0"
        :max="总时长秒"
        :value="开始秒"
        @input="更新开始(Number(($event.target as HTMLInputElement).value))"
      />
      <!-- 右滑块：结束时间 -->
      <input
        type="range"
        class="slider slider-right"
        :min="0"
        :max="总时长秒"
        :value="结束秒"
        @input="更新结束(Number(($event.target as HTMLInputElement).value))"
      />
    </div>
  </section>
</template>

<style scoped>
.slider-panel {
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

.time-display {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 14px;
  font-family: monospace;
}

.time-start,
.time-end {
  color: #333;
  font-weight: 600;
}

.time-separator {
  color: #999;
}

.time-total {
  color: #999;
  margin-left: 4px;
}

.slider-container {
  position: relative;
  height: 24px;
}

.range-highlight {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  height: 6px;
  background: #4a90d9;
  border-radius: 3px;
  pointer-events: none;
}

.slider {
  position: absolute;
  width: 100%;
  height: 24px;
  top: 0;
  left: 0;
  margin: 0;
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  pointer-events: none;
}

.slider::-webkit-slider-runnable-track {
  height: 6px;
  background: #e0e0e0;
  border-radius: 3px;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #333;
  cursor: pointer;
  margin-top: -5px;
  pointer-events: auto;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.slider::-moz-range-track {
  height: 6px;
  background: #e0e0e0;
  border-radius: 3px;
}

.slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #333;
  cursor: pointer;
  border: none;
  pointer-events: auto;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

.slider-left::-webkit-slider-thumb {
  background: #2d7d46;
}

.slider-right::-webkit-slider-thumb {
  background: #d94a4a;
}

.slider-left::-moz-range-thumb {
  background: #2d7d46;
}

.slider-right::-moz-range-thumb {
  background: #d94a4a;
}

.slider:hover::-webkit-slider-thumb {
  transform: scale(1.1);
}

.slider:hover::-moz-range-thumb {
  transform: scale(1.1);
}
</style>