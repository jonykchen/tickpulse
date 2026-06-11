<template>
  <div ref="chartRef" class="chart-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
  data: { date: string; shInflow: number; szInflow: number; total: number }[];
}>();

const chartRef = ref<HTMLElement | null>(null);
let chart: echarts.ECharts | null = null;

function renderChart() {
  if (!chartRef.value || !chart) return;

  chart.setOption({
    tooltip: { trigger: "axis" },
    legend: {
      data: ["沪股通", "深股通", "合计"],
      textStyle: { color: "#8c8c8c" },
    },
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
    series: [
      {
        name: "沪股通",
        type: "bar",
        data: props.data.map((d) => d.shInflow),
        itemStyle: { color: "#f5222d" },
      },
      {
        name: "深股通",
        type: "bar",
        data: props.data.map((d) => d.szInflow),
        itemStyle: { color: "#1890ff" },
      },
      {
        name: "合计",
        type: "line",
        data: props.data.map((d) => d.total),
        lineStyle: { color: "#faad14" },
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
