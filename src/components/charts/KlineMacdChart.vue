<template>
  <div ref="chartRef" class="kline-macd-chart"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { createChart, ColorType } from "lightweight-charts";

const props = defineProps<{
  secid: string;
  period?: string;
}>();

const chartRef = ref<HTMLElement | null>(null);
let chart: any = null;

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
    height: 200,
  });
  // MACD histogram
  const histogramSeries = chart.addHistogramSeries({
    priceFormat: { type: "price", precision: 2 },
  });
});

onUnmounted(() => {
  chart?.remove();
});
</script>

<style scoped>
.kline-macd-chart {
  width: 100%;
  height: 200px;
}
</style>
