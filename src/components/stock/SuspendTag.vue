<template>
  <span v-if="isTempSuspended" class="suspend-tag temp" :title="tooltip">
    临停
    <span v-if="resumeCountdown" class="countdown">{{ resumeCountdown }}</span>
  </span>
  <span v-else-if="isSuspended" class="suspend-tag suspended">停</span>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  isSuspended: boolean;
  isTempSuspended?: boolean;
  tempSuspendReason?: string | null;
  tempSuspendResumeTime?: number | null;
}>();

const tooltip = computed(() => {
  if (props.tempSuspendReason) {
    return `临停原因: ${props.tempSuspendReason}`;
  }
  return "临时停牌";
});

const resumeCountdown = computed(() => {
  if (!props.tempSuspendResumeTime) return null;
  const remaining = props.tempSuspendResumeTime - Math.floor(Date.now() / 1000);
  if (remaining <= 0) return null;
  const mins = Math.floor(remaining / 60);
  const secs = remaining % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
});
</script>

<style scoped>
.suspend-tag {
  display: inline-block;
  padding: 0 4px;
  border-radius: 2px;
  font-size: 10px;
  font-weight: 600;
  line-height: 16px;
  margin-left: 4px;
}
.suspended {
  background: var(--color-bg-secondary);
  color: var(--color-text-tertiary);
}
.temp {
  background: #fa8c16;
  color: #fff;
}
.countdown {
  margin-left: 2px;
  font-size: 9px;
  opacity: 0.9;
}
</style>
