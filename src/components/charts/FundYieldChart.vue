<template>
  <div ref="chartRef" class="chart-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
  data: { date: string; value: number }[];
  title?: string;
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
      axisLabel: { color: "#8c8c8c", formatter: "{value}%" },
      splitLine: { lineStyle: { color: "rgba(255,255,255,0.05)" } },
    },
    series: [
      {
        type: "line",
        data: props.data.map((d) => d.value),
        smooth: true,
        lineStyle: { color: "#1890ff" },
        areaStyle: { color: "rgba(24,144,255,0.1)" },
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
