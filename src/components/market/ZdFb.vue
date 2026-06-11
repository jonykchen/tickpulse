<template>
  <div class="zd-fb">
    <v-chart :option="chartOption" autoresize class="chart" />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import VChart from "vue-echarts";
import { use } from "echarts/core";
import { BarChart } from "echarts/charts";
import { GridComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { StockQuote } from "@/types/stock";

use([BarChart, GridComponent, TooltipComponent, CanvasRenderer]);

const props = defineProps<{
  stocks: StockQuote[];
}>();

/** 涨跌幅区间定义 */
const RANGES = [
  { label: "<-10", min: -Infinity, max: -10 },
  { label: "-10~-8", min: -10, max: -8 },
  { label: "-8~-6", min: -8, max: -6 },
  { label: "-6~-4", min: -6, max: -4 },
  { label: "-4~-2", min: -4, max: -2 },
  { label: "-2~0", min: -2, max: 0 },
  { label: "0~2", min: 0, max: 2 },
  { label: "2~4", min: 2, max: 4 },
  { label: "4~6", min: 4, max: 6 },
  { label: "6~8", min: 6, max: 8 },
  { label: "8~10", min: 8, max: 10 },
  { label: ">10", min: 10, max: Infinity },
];

const distribution = computed(() => {
  const counts = RANGES.map(() => 0);
  for (const s of props.stocks) {
    const pct = s.changePercent;
    for (let i = 0; i < RANGES.length; i++) {
      const r = RANGES[i]!;
      if (pct >= r.min && pct < r.max) {
        counts[i]++;
        break;
      }
    }
  }
  return counts;
});

const chartOption = computed(() => {
  const labels = RANGES.map((r) => r.label);
  const data = distribution.value;
  const colors = RANGES.map((r) => {
    if (r.max <= 0) return "var(--color-down)";
    if (r.min >= 0) return "var(--color-up)";
    return "var(--color-flat)";
  });

  return {
    tooltip: {
      trigger: "axis" as const,
      axisPointer: { type: "shadow" as const },
      formatter: (params: any) => {
        const p = Array.isArray(params) ? params[0] : params;
        return `${p.name}<br/>数量: ${p.value}`;
      },
    },
    grid: {
      left: 40,
      right: 16,
      top: 16,
      bottom: 28,
    },
    xAxis: {
      type: "category" as const,
      data: labels,
      axisLabel: {
        fontSize: 10,
        color: "var(--color-text-tertiary)",
        rotate: 30,
      },
      axisLine: { lineStyle: { color: "var(--color-bg-secondary)" } },
      axisTick: { show: false },
    },
    yAxis: {
      type: "value" as const,
      axisLabel: { fontSize: 10, color: "var(--color-text-tertiary)" },
      splitLine: { lineStyle: { color: "var(--color-bg-secondary)" } },
    },
    series: [
      {
        type: "bar" as const,
        data: data.map((val, idx) => ({
          value: val,
          itemStyle: { color: colors[idx] },
        })),
        barWidth: "60%",
      },
    ],
  };
});
</script>

<style scoped>
.zd-fb {
  padding: var(--spacing-md);
  width: 100%;
  height: 100%;
  min-height: 200px;
}

.chart {
  width: 100%;
  height: 100%;
  min-height: 180px;
}
</style>
