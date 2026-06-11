<template>
  <div ref="chartContainer" class="volume-chart"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type HistogramData,
  type Time,
  ColorType,
} from "lightweight-charts";
import { VOLUME_COLORS, GRID_CONFIG, TIME_SCALE_CONFIG } from "@/lib/chart-options";

const props = defineProps<{
  data: HistogramData<Time>[];
}>();

const chartContainer = ref<HTMLDivElement>();
let chart: IChartApi | null = null;
let volumeSeries: ISeriesApi<"Histogram"> | null = null;

function initChart() {
  if (!chartContainer.value) return;

  chart = createChart(chartContainer.value, {
    layout: {
      background: { type: ColorType.Solid, color: "transparent" },
      textColor: "#a0a0b0",
    },
    grid: GRID_CONFIG,
    timeScale: TIME_SCALE_CONFIG,
    rightPriceScale: {
      borderColor: "rgba(255, 255, 255, 0.1)",
    },
  });

  volumeSeries = chart.addHistogramSeries({
    priceFormat: { type: "volume" },
    priceLineVisible: false,
    lastValueVisible: false,
  });

  if (props.data.length > 0) {
    volumeSeries.setData(props.data);
  }

  chart.timeScale().fitContent();
}

watch(
  () => props.data,
  (newData) => {
    if (volumeSeries && newData.length > 0) {
      volumeSeries.setData(newData);
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
.volume-chart {
  width: 100%;
  height: 100%;
  min-height: 100px;
}
</style>
