<script setup lang="ts">
import { computed, ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    总时长秒?: number;
    当前播放秒?: number;
  }>(),
  { 总时长秒: 60, 当前播放秒: 0 }
);

const emit = defineEmits<{
  (e: "range-change", start: number, end: number): void;
}>();

const 普通精度系数 = 1;
const 微调精度系数 = 10;
const 精度系数 = computed(() => 微调模式.value ? 微调精度系数 : 普通精度系数);
const 开始值 = ref(0);
const 结束值 = ref(props.总时长秒);
const 微调模式 = ref(false);
const 微调滑块Min = ref(0);
const 微调滑块Max = ref(0);

watch(
  () => props.总时长秒,
  (newVal) => {
    开始值.value = 0;
    结束值.value = newVal;
    微调模式.value = false;
    emit("range-change", 开始值.value / 普通精度系数, 结束值.value / 普通精度系数);
  }
);

function 格式化时间(秒数: number, showMs: boolean = false): string {
  const h = Math.floor(秒数 / 3600);
  const m = Math.floor((秒数 % 3600) / 60);
  const s = Math.floor(秒数 % 60);
  const ms = Math.round((秒数 - Math.floor(秒数)) * 10);
  const 基础 = [h, m, s].map((v) => String(v).padStart(2, "0")).join(":");
  return showMs ? `${基础}.${ms}` : 基础;
}

const 开始秒 = computed(() => 开始值.value / 精度系数.value);
const 结束秒 = computed(() => 结束值.value / 精度系数.value);
const 开始时间显示 = computed(() => 格式化时间(开始秒.value, 微调模式.value));
const 结束时间显示 = computed(() => 格式化时间(结束秒.value, 微调模式.value));
const 总时长值 = computed(() => props.总时长秒 * 精度系数.value);

const 当前Min = computed(() => 微调模式.value ? 微调滑块Min.value : 0);
const 当前Max = computed(() => 微调模式.value ? 微调滑块Max.value : 总时长值.value);
const 当前Span = computed(() => 当前Max.value - 当前Min.value);

const 开始百分比 = computed(() => {
  if (微调模式.value) {
    return ((开始值.value - 当前Min.value) / 当前Span.value) * 100;
  }
  return (开始值.value / 总时长值.value) * 100;
});

const 结束百分比 = computed(() => {
  if (微调模式.value) {
    return ((结束值.value - 当前Min.value) / 当前Span.value) * 100;
  }
  return (结束值.value / 总时长值.value) * 100;
});

const 播放百分比 = computed(() => {
  if (微调模式.value) {
    const pct = ((props.当前播放秒 * 精度系数.value - 当前Min.value) / 当前Span.value) * 100;
    return Math.max(0, Math.min(100, pct));
  }
  return (props.当前播放秒 / props.总时长秒) * 100;
});
const 正在播放 = computed(() => props.当前播放秒 >= 开始秒.value && props.当前播放秒 <= 结束秒.value);

function 更新开始(value: number) {
  开始值.value = Math.max(当前Min.value, Math.min(value, 结束值.value - 精度系数.value));
}

function 更新结束(value: number) {
  结束值.value = Math.min(当前Max.value, Math.max(value, 开始值.value + 精度系数.value));
}

function onSliderChange() {
  emit("range-change", 开始秒.value, 结束秒.value);
}

function 切换微调() {
  if (!微调模式.value) {
    // 进入微调：提升精度到0.1s，转换内部值
    开始值.value = 开始值.value * 微调精度系数;
    结束值.value = 结束值.value * 微调精度系数;
    // 锁定范围（选中范围 ± 20% 余量）
    const 选中时长 = 结束值.value - 开始值.value;
    const 余量 = Math.max(选中时长 * 0.2, 微调精度系数);
    微调滑块Min.value = Math.max(0, 开始值.value - 余量);
    微调滑块Max.value = Math.min(props.总时长秒 * 微调精度系数, 结束值.value + 余量);
    微调模式.value = true;
  } else {
    // 退出微调：降回秒级精度
    开始值.value = Math.round(开始值.value / 微调精度系数);
    结束值.value = Math.round(结束值.value / 微调精度系数);
    微调模式.value = false;
  }
}

function 重置() {
  if (微调模式.value) {
    微调模式.value = false;
  }
  开始值.value = 0;
  结束值.value = props.总时长秒;
  emit("range-change", 0, props.总时长秒);
}

function getRange(): { start: string; end: string } {
  return { start: 开始时间显示.value, end: 结束时间显示.value };
}

function setRange(开始: number, 结束: number) {
  const coef = 精度系数.value;
  开始值.value = Math.max(0, Math.min(开始 * coef, 结束值.value - coef));
  结束值.value = Math.min(props.总时长秒 * coef, Math.max(结束 * coef, 开始值.value + coef));
}

defineExpose({ getRange, setRange });
</script>

<template>
  <div class="time-range">
    <button class="reset-btn" @click="重置">重置</button>
    <span class="time-item start">
      <span class="label">开始</span>
      <span class="value">{{ 开始时间显示 }}</span>
    </span>
    <div class="track-wrapper">
      <div class="track-bg" />
      <div
        class="track-active"
        :style="{ left: 开始百分比 + '%', width: 结束百分比 - 开始百分比 + '%' }"
      />
      <div
        v-if="正在播放"
        class="play-progress"
        :style="{ left: 播放百分比 + '%' }"
      />
      <div class="triangle start" :style="{ left: 开始百分比 + '%' }" />
      <div class="triangle end" :style="{ left: 结束百分比 + '%' }" />
      <input
        type="range"
        class="handle start"
        :min="当前Min"
        :max="当前Max"
        :value="开始值"
        @input="更新开始(Number(($event.target as HTMLInputElement).value))"
        @change="onSliderChange"
      />
      <input
        type="range"
        class="handle end"
        :min="当前Min"
        :max="当前Max"
        :value="结束值"
        @input="更新结束(Number(($event.target as HTMLInputElement).value))"
        @change="onSliderChange"
      />
    </div>
    <span class="time-item end">
      <span class="label">结束</span>
      <span class="value">{{ 结束时间显示 }}</span>
      <button class="fine-toggle" @click="切换微调">{{ 微调模式 ? "返回" : "微调" }}</button>
    </span>
  </div>
</template>

<style scoped>
.time-range {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 0;
}

.time-item {
  display: flex;
  gap: 6px;
  align-items: baseline;
  white-space: nowrap;
}

.time-item.start {
  order: 0;
}

.time-item.end {
  order: 2;
}

.label {
  font-size: 12px;
  color: #888;
}

.value {
  font-size: 13px;
  font-weight: 500;
  color: #222;
  font-variant-numeric: tabular-nums;
}

.track-wrapper {
  order: 1;
  flex: 1;
  position: relative;
  height: 32px;
  min-width: 100px;
}

.triangle {
  position: absolute;
  bottom: 4px;
  width: 0;
  height: 0;
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-bottom: 8px solid #222;
  transform: translateX(-6px);
}

.track-bg {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  left: 0;
  right: 0;
  height: 4px;
  background: #ddd;
  border-radius: 2px;
}

.track-active {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  height: 4px;
  background: #222;
  border-radius: 2px;
}

.play-progress {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 2px;
  height: 10px;
  background: #ff6b35;
  border-radius: 1px;
}

.handle {
  position: absolute;
  width: 100%;
  height: 32px;
  top: 0;
  left: 0;
  margin: 0;
  appearance: none;
  background: transparent;
  pointer-events: none;
}

.handle::-webkit-slider-runnable-track {
  height: 32px;
  background: transparent;
}

.handle::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 32px;
  background: transparent;
  pointer-events: auto;
  cursor: grab;
}

.handle::-moz-range-thumb {
  width: 16px;
  height: 32px;
  background: transparent;
  border: none;
  pointer-events: auto;
  cursor: grab;
}

.handle::-moz-range-track {
  height: 32px;
  background: transparent;
}

.fine-toggle {
  padding: 2px 8px;
  font-size: 12px;
  border: 1px solid #ccc;
  border-radius: 4px;
  background: #fff;
  color: #555;
  cursor: pointer;
  margin-left: 4px;
}

.fine-toggle:hover {
  background: #f0f0f0;
}

.reset-btn {
  padding: 2px 8px;
  font-size: 12px;
  border: 1px solid #ccc;
  border-radius: 4px;
  background: #fff;
  color: #555;
  cursor: pointer;
}

.reset-btn:hover {
  background: #f0f0f0;
}
</style>
