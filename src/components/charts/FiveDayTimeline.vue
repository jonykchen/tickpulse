<template>
  <div ref="chartRef" class="five-day-timeline"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { createChart, ColorType } from "lightweight-charts";

const props = defineProps<{
  secid: string;
}>();

const chartRef = ref<HTMLElement | null>(null);
let chart: any = null;
let lineSeries: any = null;

onMounted(() => {
  if (!chartRef.value) return;
  chart = createChart(chartRef.value, {
    layout: {
      background: { type: ColorType.Solid, color: "transparent" },
      textColor: "#8c8c8c",
    },
    grid: {
      vertLines: { color: "rgba(255,255,255,0.05)" },
      horzLines: { color: "rgba(255,255,255,0.05)" },
    },
    width: chartRef.value.clientWidth,
    height: 300,
    timeScale: {
      timeVisible: true,
      secondsVisible: false,
    },
  });
  lineSeries = chart.addLineSeries({
    color: "#1890ff",
    lineWidth: 1,
  });
});

onUnmounted(() => {
  chart?.remove();
});
</script>

<style scoped>
.five-day-timeline {
  width: 100%;
  height: 300px;
}
</style>
