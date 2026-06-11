<template>
  <svg class="mini-sparkline" :width="width" :height="height" :viewBox="`0 0 ${width} ${height}`">
    <polyline
      :points="points"
      fill="none"
      :stroke="lineColor"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
  </svg>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{
  prices: number[];
  width?: number;
  height?: number;
}>(), {
  width: 80,
  height: 30,
});

const lineColor = computed(() => {
  if (props.prices.length < 2) return "var(--color-flat)";
  return props.prices[props.prices.length - 1]! >= props.prices[0]!
    ? "var(--color-up)"
    : "var(--color-down)";
});

const points = computed(() => {
  const { prices, width, height } = props;
  if (prices.length < 2) return "";

  const min = Math.min(...prices);
  const max = Math.max(...prices);
  const range = max - min || 1;
  const padding = 2;

  return prices
    .map((p, i) => {
      const x = (i / (prices.length - 1)) * width;
      const y = height - padding - ((p - min) / range) * (height - padding * 2);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});
</script>

<style scoped>
.mini-sparkline {
  display: inline-block;
  vertical-align: middle;
}
</style>
