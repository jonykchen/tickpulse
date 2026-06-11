<template>
  <span v-if="note === 'early'" class="volume-ratio-note" title="开盘30分钟内量比偏高，参考价值有限">
    ⚠
  </span>
</template>

<script setup lang="ts">
import type { VolumeRatioNote } from "@/types/stock";

defineProps<{
  note: VolumeRatioNote | null;
}>();
</script>

<style scoped>
.volume-ratio-note {
  display: inline-block;
  font-size: 11px;
  color: #faad14;
  margin-left: 2px;
  cursor: help;
}
</style>
