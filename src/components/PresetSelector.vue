<script setup lang="ts">
// 预设选择组件：下拉选择 Lua 预设
import { ref, watch } from "vue";

// 假数据：Lua 预设列表
const presets = ref(["表情小文件", "清晰演示", "自定义步骤"]);
const selected = ref("");

const emit = defineEmits<{
  change: [preset: string];
}>();

function onChange() {
  emit("change", selected.value);
}

watch(selected, onChange);
</script>

<template>
  <section class="panel">
    <h3>预设选择</h3>
    <select v-model="selected">
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
</style>
