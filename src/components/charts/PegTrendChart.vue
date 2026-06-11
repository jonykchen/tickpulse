<template>
  <div ref="chartRef" class="chart-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
  data: { date: string; peg: number }[];
}>();

const chartRef = ref<HTMLElement | null>(null);
let chart: echarts.ECharts | null = null;

function renderChart() {
  if (!chartRef.value || !chart) return;

  chart.setOption({
    tooltip: { trigger: "axis" },
    grid: { left: "3%", right: "3%", bottom: "3%", containLabel: true },
    xAxis: {
      type: "category",
      data: props.data.map((d) => d.date),
      axisLabel: { color: "#8c8c8c", fontSize: 10 },
    },
    yAxis: {
      type: "value",
      axisLabel: { color: "#8c8c8c" },
      splitLine: { lineStyle: { color: "rgba(255,255,255,0.05)" } },
    },
    visualMap: {
      show: false,
      pieces: [
        { lte: 0.5, color: "#52c41a" },
        { gt: 0.5, lte: 0.8, color: "#73d13d" },
        { gt: 0.8, lte: 1.2, color: "#faad14" },
        { gt: 1.2, lte: 1.5, color: "#fa8c16" },
        { gt: 1.5, color: "#f5222d" },
      ],
    },
    series: [
      {
        type: "line",
        data: props.data.map((d) => d.peg),
        smooth: true,
      },
    ],
  });
}

onMounted(() => {
  if (chartRef.value) {
    chart = echarts.init(chartRef.value);
    renderChart();
  }
});

watch(() => props.data, renderChart, { deep: true });

onUnmounted(() => {
  chart?.dispose();
});
</script>

<style scoped>
.chart-container {
  width: 100%;
  height: 200px;
}
</style>
