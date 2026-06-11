<template>
  <table class="position-table">
    <thead>
      <tr>
        <th>名称</th>
        <th>成本价</th>
        <th>数量</th>
        <th>现价</th>
        <th>盈亏</th>
        <th>盈亏率</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="pos in positions" :key="pos.id">
        <td>{{ pos.name || pos.secid }}</td>
        <td class="num">{{ pos.costPrice }}</td>
        <td class="num">{{ pos.quantity }}</td>
        <td class="num">{{ getCurrentPrice(pos.secid) }}</td>
        <td class="num" :class="getProfitClass(pos)">
          {{ calcProfit(pos) }}
        </td>
        <td class="num" :class="getProfitClass(pos)">
          {{ calcProfitRate(pos) }}%
        </td>
      </tr>
    </tbody>
  </table>
</template>

<script setup lang="ts">
const props = defineProps<{
  positions: any[];
  quotes: any[];
}>();

function getCurrentPrice(secid: string): string {
  const q = props.quotes.find((q) => q.secid === secid);
  return q ? q.price.toFixed(2) : "--";
}

function calcProfit(pos: any): string {
  const q = props.quotes.find((q) => q.secid === pos.secid);
  if (!q) return "--";
  const profit = (q.price - parseFloat(pos.costPrice)) * parseInt(pos.quantity);
  const prefix = profit > 0 ? "+" : "";
  return `${prefix}${profit.toFixed(2)}`;
}

function calcProfitRate(pos: any): string {
  const q = props.quotes.find((q) => q.secid === pos.secid);
  if (!q) return "--";
  const rate = ((q.price - parseFloat(pos.costPrice)) / parseFloat(pos.costPrice)) * 100;
  const prefix = rate > 0 ? "+" : "";
  return `${prefix}${rate.toFixed(2)}`;
}

function getProfitClass(pos: any): string {
  const q = props.quotes.find((q) => q.secid === pos.secid);
  if (!q) return "";
  const profit = q.price - parseFloat(pos.costPrice);
  if (profit > 0) return "text-up";
  if (profit < 0) return "text-down";
  return "text-flat";
}
</script>

<style scoped>
.position-table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-size-sm);
}
th, td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  border-bottom: 1px solid var(--color-bg-secondary);
}
th {
  color: var(--color-text-tertiary);
  font-weight: 500;
}
th:first-child, td:first-child {
  text-align: left;
}
</style>