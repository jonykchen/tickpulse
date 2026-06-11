<template>
  <div ref="chartContainer" class="kline-chart"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type CandlestickData,
  type Time,
  ColorType,
} from "lightweight-charts";
import { CANDLE_COLORS, GRID_CONFIG, CROSSHAIR_CONFIG, TIME_SCALE_CONFIG, MA_COLORS } from "@/lib/chart-options";

const props = defineProps<{
  data: CandlestickData<Time>[];
  maData?: Record<string, CandlestickData<Time>[]>;
}>();

const chartContainer = ref<HTMLDivElement>();
let chart: IChartApi | null = null;
let candleSeries: ISeriesApi<"Candlestick"> | null = null;
let maSeries: Map<string, ISeriesApi<"Line">> = new Map();

function initChart() {
  if (!chartContainer.value) return;

  chart = createChart(chartContainer.value, {
    layout: {
      background: { type: ColorType.Solid, color: "transparent" },
      textColor: "#a0a0b0",
    },
    grid: GRID_CONFIG,
    crosshair: CROSSHAIR_CONFIG,
    timeScale: TIME_SCALE_CONFIG,
    rightPriceScale: {
      borderColor: "rgba(255, 255, 255, 0.1)",
    },
  });

  candleSeries = chart.addCandlestickSeries({
    ...CANDLE_COLORS,
  });

  if (props.data.length > 0) {
    candleSeries.setData(props.data);
  }

  // MA lines
  if (props.maData) {
    for (const [key, data] of Object.entries(props.maData)) {
      const color = MA_COLORS[key as keyof typeof MA_COLORS] || "#ffffff";
      const series = chart.addLineSeries({
        color,
        lineWidth: 1,
        priceLineVisible: false,
        lastValueVisible: false,
      });
      series.setData(data);
      maSeries.set(key, series);
    }
  }

  chart.timeScale().fitContent();
}

watch(
  () => props.data,
  (newData) => {
    if (candleSeries && newData.length > 0) {
      candleSeries.setData(newData);
    }
  }
);

onMounted(() => {
  initChart();
  window.addEventListener("resize", handleResize);
});

onUnmounted(() => {
  window.removeEventListener("resize", handleResize);
  if (chart) {
    chart.remove();
    chart = null;
  }
});

function handleResize() {
  if (chart && chartContainer.value) {
    chart.applyOptions({
      width: chartContainer.value.clientWidth,
      height: chartContainer.value.clientHeight,
    });
  }
}
</script>

<style scoped>
.kline-chart {
  width: 100%;
  height: 100%;
  min-height: 300px;
}
</style>
