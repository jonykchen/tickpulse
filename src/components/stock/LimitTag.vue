<template>
  <span v-if="label" class="limit-tag" :class="limitClass">
    {{ label }}
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  changePercent: number;
  sealStrength?: number;
}>();

const label = computed(() => {
  if (props.changePercent >= 9.9) return "涨停";
  if (props.changePercent >= 4.9 && props.changePercent < 5.1) return "涨停";
  if (props.changePercent <= -9.9) return "跌停";
  if (props.changePercent <= -4.9 && props.changePercent > -5.1) return "跌停";
  return "";
});

const limitClass = computed(() => {
  if (props.changePercent > 0) return "limit-up";
  return "limit-down";
});
</script>

<style scoped>
.limit-tag {
  display: inline-block;
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 10px;
  font-weight: 700;
}
.limit-up {
  background: rgba(255, 68, 68, 0.2);
  color: var(--color-limit-up);
}
.limit-down {
  background: rgba(0, 204, 102, 0.2);
  color: var(--color-limit-down);
}
</style>
