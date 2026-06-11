<template>
  <span v-if="isAnomaly" class="anomaly-badge" :class="anomalyClass">
    ⚡ <span class="anomaly-text">{{ anomalyText }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  changeSpeed: number;
  changePercent: number;
  anomalyThreshold?: number;
}>();

const isAnomaly = computed(() => {
  const threshold = props.anomalyThreshold ?? 2.0; // 默认涨速超过2%视为异动
  return Math.abs(props.changeSpeed) > threshold;
});

const anomalyClass = computed(() => {
  if (props.changeSpeed > 0) return "anomaly-rise";
  return "anomaly-fall";
});

const anomalyText = computed(() => {
  if (props.changeSpeed > 0) return "异动拉升";
  return "异动下跌";
});
</script>

<style scoped>
.anomaly-badge {
  display: inline-flex;
  align-items: center;
  padding: 0 4px;
  border-radius: 2px;
  font-size: 10px;
  line-height: 16px;
  margin-left: 4px;
}
.anomaly-rise {
  background: rgba(245, 34, 45, 0.15);
  color: #f5222d;
}
.anomaly-fall {
  background: rgba(82, 196, 26, 0.15);
  color: #52c41a;
}
.anomaly-text {
  font-weight: 600;
}
</style>
