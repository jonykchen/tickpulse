<template>
  <div class="north-flow">
    <div class="flow-card">
      <div class="flow-label">沪股通</div>
      <div class="flow-value num" :class="flowClass(shNetInflow)">
        {{ formatFlowValue(shNetInflow) }}
      </div>
    </div>
    <div class="flow-card">
      <div class="flow-label">深股通</div>
      <div class="flow-value num" :class="flowClass(szNetInflow)">
        {{ formatFlowValue(szNetInflow) }}
      </div>
    </div>
    <div class="flow-card flow-total">
      <div class="flow-label">北向合计</div>
      <div class="flow-value num" :class="flowClass(totalNetInflow)">
        {{ formatFlowValue(totalNetInflow) }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  shNetInflow: number;
  szNetInflow: number;
}>();

const totalNetInflow = computed(() => props.shNetInflow + props.szNetInflow);

function flowClass(val: number): string {
  if (val > 0) return "text-inflow";
  if (val < 0) return "text-outflow";
  return "text-flat";
}

function formatFlowValue(val: number): string {
  const absVal = Math.abs(val);
  let formatted: string;
  if (absVal >= 1e8) {
    formatted = (absVal / 1e8).toFixed(2) + "亿";
  } else if (absVal >= 1e4) {
    formatted = (absVal / 1e4).toFixed(2) + "万";
  } else {
    formatted = absVal.toFixed(0);
  }
  const prefix = val > 0 ? "+" : val < 0 ? "-" : "";
  return prefix + formatted;
}
</script>

<style scoped>
.north-flow {
  display: flex;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
}

.flow-card {
  flex: 1;
  padding: var(--spacing-lg);
  border-radius: var(--radius-md);
  background: var(--color-surface);
  text-align: center;
  transition: background var(--transition-fast);
}

.flow-card:hover {
  background: var(--color-surface-hover);
}

.flow-total {
  border: 1px solid var(--color-bg-tertiary);
}

.flow-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.flow-value {
  font-size: var(--font-size-xl);
  font-weight: 700;
}

.text-inflow {
  color: var(--color-up);
}

.text-outflow {
  color: var(--color-down);
}
</style>
