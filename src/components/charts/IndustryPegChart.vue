<template>
  <div ref="chartRef" class="chart-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
  data: { name: string; peg: number; industryPeg: number }[];
}>();

const chartRef = ref<HTMLElement | null>(null);
let chart: echarts.ECharts | null = null;

function renderChart() {
  if (!chartRef.value || !chart) return;

  chart.setOption({
    tooltip: { trigger: "axis" },
    legend: {
      data: ["个股 PEG", "行业 PEG"],
      textStyle: { color: "#8c8c8c" },
    },
    grid: { left: "3%", right: "3%", bottom: "3%", containLabel: true },
    xAxis: {
      type: "category",
      data: props.data.map((d) => d.name),
      axisLabel: { color: "#8c8c8c", fontSize: 10, rotate: 30 },
    },
    yAxis: {
      type: "value",
      axisLabel: { color: "#8c8c8c" },
      splitLine: { lineStyle: { color: "rgba(255,255,255,0.05)" } },
    },
    series: [
      {
        name: "个股 PEG",
        type: "bar",
        data: props.data.map((d) => d.peg),
        itemStyle: { color: "#1890ff" },
      },
      {
        name: "行业 PEG",
        type: "line",
        data: props.data.map((d) => d.industryPeg),
        lineStyle: { color: "#faad14", type: "dashed" },
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
  height: 300px;
}
</style>
