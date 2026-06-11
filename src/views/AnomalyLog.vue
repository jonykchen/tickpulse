<template>
  <div class="anomaly-view">
    <div class="anomaly-header">
      <h2>异动历史</h2>
      <button class="btn-clear" @click="clearAll">清空</button>
    </div>
    <div class="anomaly-list">
      <div
        v-for="event in recentAnomalies"
        :key="event.id"
        class="anomaly-item"
        :class="`anomaly-${event.type}`"
      >
        <span class="anomaly-type">{{ typeLabel(event.type) }}</span>
        <span class="anomaly-name">{{ event.stockName }}</span>
        <span class="anomaly-value num">{{ event.value.toFixed(2) }}</span>
        <span class="anomaly-time">{{ formatTime(event.timestamp) }}</span>
        <span v-if="event.detail" class="anomaly-detail">{{ event.detail }}</span>
      </div>
      <div v-if="recentAnomalies.length === 0" class="empty-state">
        暂无异动记录
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useAnomalyStore } from "@/stores/anomaly";
import { AnomalyType } from "@/types/anomaly";

const anomalyStore = useAnomalyStore();
const recentAnomalies = computed(() => anomalyStore.recentAnomalies);

function typeLabel(type: AnomalyType): string {
  const map: Record<AnomalyType, string> = {
    [AnomalyType.Surge]: "急涨",
    [AnomalyType.Dive]: "急跌",
    [AnomalyType.VolumeSpike]: "放量",
    [AnomalyType.SealBoard]: "封板",
    [AnomalyType.BreakBoard]: "炸板",
    [AnomalyType.TempSuspend]: "临停",
  };
  return map[type] || type;
}

function formatTime(timestamp: number): string {
  const d = new Date(timestamp * 1000);
  return `${d.getHours().toString().padStart(2, "0")}:${d.getMinutes().toString().padStart(2, "0")}:${d.getSeconds().toString().padStart(2, "0")}`;
}

function clearAll() {
  anomalyStore.clearAnomalies();
}
</script>

<style scoped>
.anomaly-view {
  padding: var(--spacing-lg);
}
.anomaly-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}
.btn-clear {
  padding: var(--spacing-xs) var(--spacing-md);
  background: transparent;
  border: 1px solid var(--color-bg-tertiary);
  border-radius: var(--radius-sm);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: var(--font-size-sm);
}
.anomaly-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}
.anomaly-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-surface);
  border-radius: var(--radius-sm);
  font-size: var(--font-size-sm);
}
.anomaly-type {
  padding: 1px 6px;
  border-radius: 2px;
  font-weight: 600;
  font-size: 10px;
}
.anomaly-surge .anomaly-type { background: rgba(255, 102, 0, 0.2); color: var(--color-anomaly-surge); }
.anomaly-dive .anomaly-type { background: rgba(0, 170, 68, 0.2); color: var(--color-anomaly-dive); }
.anomaly-volume_spike .anomaly-type { background: rgba(255, 170, 0, 0.2); color: var(--color-anomaly-volume); }
.anomaly-seal_board .anomaly-type { background: rgba(255, 34, 34, 0.2); color: var(--color-seal-strong); }
.anomaly-break_board .anomaly-type { background: rgba(255, 136, 136, 0.2); color: var(--color-seal-weak); }
.anomaly-name { color: var(--color-text-primary); }
.anomaly-time { color: var(--color-text-tertiary); font-size: var(--font-size-xs); }
.anomaly-detail { color: var(--color-text-tertiary); font-size: var(--font-size-xs); }
.empty-state { color: var(--color-text-tertiary); text-align: center; padding: var(--spacing-xl); }
</style>