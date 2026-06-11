<template>
  <div class="market-phase-bar">
    <span class="phase-tag" :class="phaseClass">{{ phaseName }}</span>
    <span v-if="summary" class="market-summary">
      <span class="text-up">{{ summary.upCount }}</span>
      <span class="separator">/</span>
      <span class="text-down">{{ summary.downCount }}</span>
      <span class="separator">/</span>
      <span class="text-flat">{{ summary.flatCount }}</span>
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useMarketStore } from "@/stores/market";

const marketStore = useMarketStore();

const phaseName = computed(() => marketStore.phase || "未连接");
const summary = computed(() => marketStore.marketSummary);

const phaseClass = computed(() => {
  const name = phaseName.value;
  if (name === "休市") return "phase-holiday";
  if (name === "交易中" || name === "集合竞价") return "phase-trading";
  if (name === "午间休市") return "phase-lunch";
  if (name === "已收盘") return "phase-closed";
  return "phase-default";
});
</script>

<style scoped>
.market-phase-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  font-size: var(--font-size-sm);
}
.phase-tag {
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-weight: 600;
}
.phase-trading {
  background: rgba(231, 76, 60, 0.15);
  color: var(--color-up);
}
.phase-lunch {
  background: rgba(243, 156, 18, 0.15);
  color: var(--color-warning);
}
.phase-closed,
.phase-holiday {
  background: rgba(149, 165, 166, 0.15);
  color: var(--color-text-tertiary);
}
.market-summary {
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
}
.separator {
  color: var(--color-text-tertiary);
  margin: 0 1px;
}
</style>
