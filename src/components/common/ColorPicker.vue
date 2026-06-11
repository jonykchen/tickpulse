<template>
  <NColorPicker
    :value="modelValue"
    :modes="['hex']"
    :show-alpha="false"
    :swatches="presetColors"
    @update:value="$emit('update:modelValue', $event)"
  />
</template>

<script setup lang="ts">
import { NColorPicker } from "naive-ui";

defineProps<{
  modelValue: string;
}>();

defineEmits<{
  "update:modelValue": [value: string];
}>();

const presetColors = [
  "#f5222d", "#fa541c", "#fa8c16", "#faad14",
  "#52c41a", "#13c2c2", "#1890ff", "#2f54eb",
  "#722ed1", "#eb2f96", "#ffffff", "#000000",
];
</script>
