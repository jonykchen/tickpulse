<template>
  <div class="kline-panel">
    <div class="kline-main" ref="mainChartRef"></div>
    <div class="kline-indicator" ref="indicatorChartRef"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { createChart, ColorType } from "lightweight-charts";

const props = defineProps<{
  secid: string;
  period?: string;
  adjust?: string;
}>();

const mainChartRef = ref<HTMLElement | null>(null);
const indicatorChartRef = ref<HTMLElement | null>(null);
let mainChart: any = null;
let indicatorChart: any = null;

onMounted(() => {
  if (mainChartRef.value) {
    mainChart = createChart(mainChartRef.value, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#8c8c8c",
      },
      grid: {
        vertLines: { color: "rgba(255,255,255,0.05)" },
        horzLines: { color: "rgba(255,255,255,0.05)" },
      },
      width: mainChartRef.value.clientWidth,
      height: 300,
    });
    const candleSeries = mainChart.addCandlestickSeries({
      upColor: "#f5222d",
      downColor: "#52c41a",
      borderUpColor: "#f5222d",
      borderDownColor: "#52c41a",
      wickUpColor: "#f5222d",
      wickDownColor: "#52c41a",
    });
  }

  if (indicatorChartRef.value) {
    indicatorChart = createChart(indicatorChartRef.value, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#8c8c8c",
      },
      grid: {
        vertLines: { color: "rgba(255,255,255,0.05)" },
        horzLines: { color: "rgba(255,255,255,0.05)" },
      },
      width: indicatorChartRef.value.clientWidth,
      height: 100,
    });
  }
});

onUnmounted(() => {
  mainChart?.remove();
  indicatorChart?.remove();
});
</script>

<style scoped>
.kline-panel {
  display: flex;
  flex-direction: column;
}
.kline-main {
  flex: 3;
}
.kline-indicator {
  flex: 1;
}
</style>
