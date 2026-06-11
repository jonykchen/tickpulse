<template>
  <div
    class="skeleton"
    :style="{
      width: width ?? '100%',
      height: height ?? '20px',
      borderRadius: radius ?? 'var(--radius-sm)'
    }"
  ></div>
</template>

<script setup lang="ts">
defineProps<{
  width?: string;
  height?: string;
  radius?: string;
}>();
</script>

<style scoped>
/* 使用 global.css 中定义的 .skeleton 类 */
</style>
