<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";

interface CropRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

const props = defineProps<{
  videoRef: HTMLVideoElement | null;
}>();

const emit = defineEmits<{
  (e: "crop-change", crop: CropRegion | null): void;
}>();

const crop = ref<CropRegion | null>(null);

// 拖拽模式
type DragMode = "idle" | "drawing" | "move" | "resize";
const HANDLE_CURSORS: Record<ResizeHandle, string> = {
  nw: "nwse-resize",
  ne: "nesw-resize",
  sw: "nesw-resize",
  se: "nwse-resize"
};

const dragMode = ref<DragMode>("idle");
const resizeHandle = ref<ResizeHandle>("nw");
const dragStartDom = ref({ x: 0, y: 0 }); // 鼠标起始 DOM 坐标
const dragStartCrop = ref<CropRegion>({ x: 0, y: 0, width: 0, height: 0 }); // 起始 crop 像素值
const dynamicCursor = ref("crosshair"); // 动态鼠标样式

// 将 DOM 坐标转换为视频实际像素坐标
function getVideoTransform(video: HTMLVideoElement) {
  const displayW = video.offsetWidth;
  const displayH = video.offsetHeight;
  const actualW = video.videoWidth;
  const actualH = video.videoHeight;
  const scale = Math.min(displayW / actualW, displayH / actualH);
  const renderedW = actualW * scale;
  const renderedH = actualH * scale;
  return { offsetX: (displayW - renderedW) / 2, offsetY: (displayH - renderedH) / 2, scale };
}

function domToVideoPixel(domX: number, domY: number, video: HTMLVideoElement) {
  const t = getVideoTransform(video);
  return {
    x: Math.max(0, Math.min(Math.round((domX - t.offsetX) / t.scale), video.videoWidth)),
    y: Math.max(0, Math.min(Math.round((domY - t.offsetY) / t.scale), video.videoHeight)),
  };
}

// 视频像素 → overlay DOM 坐标
function videoPixelToDomRect(px: number, py: number, pw: number, ph: number) {
  const video = props.videoRef;
  if (!video || !video.videoWidth) return null;
  const t = getVideoTransform(video);
  return {
    x: t.offsetX + px * t.scale,
    y: t.offsetY + py * t.scale,
    width: pw * t.scale,
    height: ph * t.scale,
  };
}

// 判断鼠标是否在手柄上（返回 handle 名或 null）
function hitTestHandle(domX: number, domY: number, domRect: NonNullable<ReturnType<typeof videoPixelToDomRect>>): ResizeHandle | null {
  const radius = 8; // 点击容差
  const handles: { name: ResizeHandle; hx: number; hy: number }[] = [
    { name: "nw", hx: domRect.x, hy: domRect.y },
    { name: "ne", hx: domRect.x + domRect.width, hy: domRect.y },
    { name: "sw", hx: domRect.x, hy: domRect.y + domRect.height },
    { name: "se", hx: domRect.x + domRect.width, hy: domRect.y + domRect.height },
  ];
  for (const h of handles) {
    if (Math.abs(domX - h.hx) <= radius && Math.abs(domY - h.hy) <= radius) return h.name;
  }
  return null;
}

// 判断鼠标是否在裁剪区域内
function hitTestBody(domX: number, domY: number, domRect: NonNullable<ReturnType<typeof videoPixelToDomRect>>) {
  const margin = 2;
  return domX >= domRect.x - margin && domX <= domRect.x + domRect.width + margin &&
         domY >= domRect.y - margin && domY <= domRect.y + domRect.height + margin;
}

function onMousedown(e: MouseEvent) {
  const video = props.videoRef;
  if (!video || !video.videoWidth || e.button !== 0) return;

  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const domX = e.clientX - rect.left;
  const domY = e.clientY - rect.top;

  const t = getVideoTransform(video);
  // 点击在 letterbox 区域外，忽略
  if (domX < t.offsetX || domX > t.offsetX + video.videoWidth * t.scale ||
      domY < t.offsetY || domY > t.offsetY + video.videoHeight * t.scale) return;

  // 检查是否点击在裁剪区域或手柄上
  if (crop.value) {
    const domRect = videoPixelToDomRect(crop.value.x, crop.value.y, crop.value.width, crop.value.height);
    if (!domRect) return;

    const handle = hitTestHandle(domX, domY, domRect);
    if (handle) {
      dragMode.value = "resize";
      resizeHandle.value = handle;
      dragStartDom.value = { x: domX, y: domY };
      dragStartCrop.value = { ...crop.value };
      e.stopPropagation();
      return;
    }

    if (hitTestBody(domX, domY, domRect)) {
      dragMode.value = "move";
      dragStartDom.value = { x: domX, y: domY };
      dragStartCrop.value = { ...crop.value };
      e.stopPropagation();
      return;
    }

    // 点击在遮罩区域，忽略
    return;
  }

  // 无裁剪区域，开始绘制
  dragMode.value = "drawing";
  dragStartDom.value = { x: domX, y: domY };
  const p = domToVideoPixel(domX, domY, video);
  dragStartCrop.value = { x: p.x, y: p.y, width: 0, height: 0 };
}

function onMousemove(e: MouseEvent) {
  const video = props.videoRef;
  if (!video || !video.videoWidth) return;

  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const domX = e.clientX - rect.left;
  const domY = e.clientY - rect.top;

  // 动态计算鼠标样式
  if (crop.value && dragMode.value === "idle") {
    const domRect = videoPixelToDomRect(crop.value.x, crop.value.y, crop.value.width, crop.value.height);
    if (domRect) {
      const handle = hitTestHandle(domX, domY, domRect);
      if (handle) {
        dynamicCursor.value = HANDLE_CURSORS[handle];
      } else if (hitTestBody(domX, domY, domRect)) {
        dynamicCursor.value = "move";
      } else {
        // 遮罩区域显示默认指针，不响应裁剪操作
        dynamicCursor.value = "default";
      }
    }
  } else if (!crop.value && dragMode.value === "idle") {
    // 无裁剪时显示十字星用于绘制新裁剪
    dynamicCursor.value = "crosshair";
  } else if (dragMode.value !== "idle") {
    // 拖拽时保持当前样式
    if (dragMode.value === "move") dynamicCursor.value = "move";
    else if (dragMode.value === "resize") {
      dynamicCursor.value = HANDLE_CURSORS[resizeHandle.value];
    }
  }

  if (dragMode.value === "drawing") {
    const p = domToVideoPixel(domX, domY, video);
    const sx = dragStartCrop.value.x;
    const sy = dragStartCrop.value.y;
    const x = Math.min(sx, p.x);
    const y = Math.min(sy, p.y);
    const w = Math.abs(p.x - sx);
    const h = Math.abs(p.y - sy);
    crop.value = { x, y, width: w, height: h };
    emit("crop-change", crop.value);
  } else if (dragMode.value === "move") {
    const dx = domX - dragStartDom.value.x;
    const dy = domY - dragStartDom.value.y;
    const t = getVideoTransform(video);
    const pxDx = dx / t.scale;
    const pyDy = dy / t.scale;
    const sc = dragStartCrop.value;
    const newX = clampInt(sc.x + pxDx, 0, video.videoWidth - sc.width);
    const newY = clampInt(sc.y + pyDy, 0, video.videoHeight - sc.height);
    crop.value = { x: newX, y: newY, width: sc.width, height: sc.height };
    emit("crop-change", crop.value);
  } else if (dragMode.value === "resize") {
    const t = getVideoTransform(video);
    const pxDx = (domX - dragStartDom.value.x) / t.scale;
    const pyDy = (domY - dragStartDom.value.y) / t.scale;
    const sc = dragStartCrop.value;
    const min = 16;
    let { x, y, width, height } = sc;

    switch (resizeHandle.value) {
      case "se":
        // 右下：向右增宽度，向下增高度
        width = Math.max(min, sc.width + pxDx);
        height = Math.max(min, sc.height + pyDy);
        break;
      case "sw":
        // 左下：向左减x增宽度，向下增高度
        const swWidth = Math.max(min, sc.width - pxDx);
        x = sc.x + sc.width - swWidth;
        width = swWidth;
        height = Math.max(min, sc.height + pyDy);
        break;
      case "ne":
        // 右上：向右增宽度，向上减y增高度
        const neHeight = Math.max(min, sc.height - pyDy);
        y = sc.y + sc.height - neHeight;
        width = Math.max(min, sc.width + pxDx);
        height = neHeight;
        break;
      case "nw":
        // 左上：向左减x增宽度，向上减y增高度
        const nwWidth = Math.max(min, sc.width - pxDx);
        const nwHeight = Math.max(min, sc.height - pyDy);
        x = sc.x + sc.width - nwWidth;
        y = sc.y + sc.height - nwHeight;
        width = nwWidth;
        height = nwHeight;
        break;
    }

    // 边界约束
    x = clampInt(x, 0, video.videoWidth - width);
    y = clampInt(y, 0, video.videoHeight - height);
    width = clampInt(width, min, video.videoWidth - x);
    height = clampInt(height, min, video.videoHeight - y);

    crop.value = { x, y, width, height };
    emit("crop-change", crop.value);
  }
}

function onMouseup() {
  if (dragMode.value === "drawing" && crop.value) {
    if (crop.value.width < 16 || crop.value.height < 16) {
      crop.value = null;
      emit("crop-change", null);
    }
  }
  dragMode.value = "idle";
}

function clampInt(v: number, min: number, max: number) {
  return Math.max(min, Math.min(max, Math.round(v)));
}

function clearCrop() {
  crop.value = null;
  dragMode.value = "idle";
  emit("crop-change", null);
}

// 已选裁剪区域在 overlay 中的 DOM 坐标
const cropDomRect = computed(() => {
  if (!crop.value || crop.value.width === 0 || crop.value.height === 0) return null;
  return videoPixelToDomRect(crop.value.x, crop.value.y, crop.value.width, crop.value.height);
});

// 尺寸提示文本
const sizeHint = computed(() => {
  if (!crop.value) return "";
  return `${crop.value.width} × ${crop.value.height}`;
});

// 全局 mouseup
onUnmounted(() => {
  dragMode.value = "idle";
});
</script>

<template>
  <div
    class="crop-overlay"
    :style="{ cursor: dynamicCursor }"
    @mousedown="onMousedown"
    @mousemove="onMousemove"
    @mouseup="onMouseup"
    @mouseleave="onMouseup"
  >
    <!-- 遮罩覆盖整个容器（包括letterbox） -->
    <template v-if="cropDomRect">
      <div class="mask mask-top" :style="{ top: '0', height: cropDomRect.y + 'px' }" />
      <div class="mask mask-bottom" :style="{ top: (cropDomRect.y + cropDomRect.height) + 'px', bottom: '0' }" />
      <div class="mask mask-left" :style="{ left: '0', width: cropDomRect.x + 'px', top: cropDomRect.y + 'px', height: cropDomRect.height + 'px' }" />
      <div class="mask mask-right" :style="{ left: (cropDomRect.x + cropDomRect.width) + 'px', right: '0', top: cropDomRect.y + 'px', height: cropDomRect.height + 'px' }" />
    </template>

    <!-- 已选裁剪矩形边框 -->
    <div v-if="cropDomRect" class="crop-border" :style="{
      left: cropDomRect.x + 'px',
      top: cropDomRect.y + 'px',
      width: cropDomRect.width + 'px',
      height: cropDomRect.height + 'px',
    }">
      <!-- 四角手柄（隐藏右上角） -->
      <div class="crop-handle handle-nw" style="left: -5px; top: -5px; cursor: nwse-resize;" />
      <div class="crop-handle handle-sw" style="left: -5px; bottom: -5px; cursor: nesw-resize;" />
      <div class="crop-handle handle-se" style="right: -5px; bottom: -5px; cursor: nwse-resize;" />
      <!-- 尺寸提示 -->
      <div class="size-hint">{{ sizeHint }}</div>
      <!-- 清除按钮 -->
      <div class="crop-clear-btn" style="right: -10px; top: -10px;" @mousedown.stop @click.stop="clearCrop"></div>
    </div>

    <!-- 未选择时的提示 -->
    <div v-if="!crop && dragMode === 'idle'" class="crop-hint">
      拖动选择裁剪区域
    </div>
  </div>
</template>

<style scoped>
.crop-overlay {
  position: absolute;
  inset: 0;
}

.mask {
  position: absolute;
  background: rgba(0, 0, 0, 0.45);
  pointer-events: none;
}

.mask-top, .mask-bottom {
  left: 0;
  right: 0;
}

.mask-top {
  top: 0;
}

.mask-bottom {
  bottom: 0;
}

.crop-border {
  position: absolute;
  border: 2px solid #4fc3f7;
  box-sizing: border-box;
  pointer-events: none;
}

.crop-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background: #4fc3f7;
  border: 1.5px solid #fff;
  border-radius: 2px;
  pointer-events: auto;
}

.crop-clear-btn {
  position: absolute;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.7);
  border: 1px solid #fff;
  pointer-events: auto;
  cursor: pointer;
}

.crop-clear-btn::before,
.crop-clear-btn::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 10px;
  height: 1.5px;
  background: #fff;
  transform-origin: center;
}

.crop-clear-btn::before {
  transform: translate(-50%, -50%) rotate(45deg);
}

.crop-clear-btn::after {
  transform: translate(-50%, -50%) rotate(-45deg);
}

.size-hint {
  position: absolute;
  bottom: -24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.75);
  color: #fff;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  white-space: nowrap;
  pointer-events: none;
}

.crop-hint {
  position: absolute;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.65);
  color: #fff;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
  pointer-events: none;
  white-space: nowrap;
}
</style>
