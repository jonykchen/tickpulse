<template>
  <div ref="chartContainer" class="timeline-chart"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type LineData,
  type Time,
  ColorType,
} from "lightweight-charts";
import { TIMELINE_COLORS, GRID_CONFIG, TIME_SCALE_CONFIG } from "@/lib/chart-options";

const props = defineProps<{
  priceData: LineData<Time>[];
  avgData?: LineData<Time>[];
  preClose?: number;
}>();

const chartContainer = ref<HTMLDivElement>();
let chart: IChartApi | null = null;
let priceSeries: ISeriesApi<"Area"> | null = null;
let avgSeries: ISeriesApi<"Line"> | null = null;

function initChart() {
  if (!chartContainer.value) return;

  chart = createChart(chartContainer.value, {
    layout: {
      background: { type: ColorType.Solid, color: "transparent" },
      textColor: "#a0a0b0",
    },
    grid: GRID_CONFIG,
    timeScale: { ...TIME_SCALE_CONFIG, timeVisible: true },
    rightPriceScale: {
      borderColor: "rgba(255, 255, 255, 0.1)",
    },
  });

  priceSeries = chart.addAreaSeries({
    lineColor: TIMELINE_COLORS.lineColor,
    topColor: TIMELINE_COLORS.areaTopColor,
    bottomColor: TIMELINE_COLORS.areaBottomColor,
    lineWidth: 1,
  });

  if (props.priceData.length > 0) {
    priceSeries.setData(props.priceData);
  }

  // 均价线
  if (props.avgData && props.avgData.length > 0) {
    avgSeries = chart.addLineSeries({
      color: TIMELINE_COLORS.avgLineColor,
      lineWidth: 1,
      priceLineVisible: false,
      lastValueVisible: false,
    });
    avgSeries.setData(props.avgData);
  }

  // 昨收参考线
  if (props.preClose && props.preClose > 0) {
    priceSeries.createPriceLine({
      price: props.preClose,
      color: "rgba(149, 165, 166, 0.5)",
      lineWidth: 1,
      lineStyle: 2, // Dashed
    });
  }

  chart.timeScale().fitContent();
}

watch(
  () => props.priceData,
  (newData) => {
    if (priceSeries && newData.length > 0) {
      priceSeries.setData(newData);
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
.timeline-chart {
  width: 100%;
  height: 100%;
  min-height: 250px;
}
</style>
