<template>
  <div class="hot-stocks">
    <table>
      <thead>
        <tr>
          <th class="col-rank">排名</th>
          <th class="col-name">名称</th>
          <th class="col-price">最新价</th>
          <th class="col-change">涨跌幅</th>
          <th class="col-amount">成交额</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(stock, idx) in topStocks" :key="stock.secid">
          <td class="col-rank num">
            <span class="rank-badge" :class="rankClass(idx)">{{ idx + 1 }}</span>
          </td>
          <td class="col-name">
            <div class="stock-name">{{ stock.name }}</div>
            <div class="stock-code num">{{ stock.code }}</div>
          </td>
          <td class="col-price num" :class="getChangeColorClass(stock.changePercent)">
            {{ formatPrice(stock.price) }}
          </td>
          <td class="col-change num" :class="getChangeColorClass(stock.changePercent)">
            {{ formatChangePercent(stock.changePercent) }}
          </td>
          <td class="col-amount num">{{ formatAmount(stock.amount) }}</td>
        </tr>
        <tr v-if="topStocks.length === 0">
          <td colspan="5" class="empty-state">暂无数据</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { StockQuote } from "@/types/stock";
import { formatPrice, formatChangePercent, formatAmount, getChangeColorClass } from "@/lib/format";

const props = defineProps<{
  stocks: StockQuote[];
}>();

const topStocks = computed(() => {
  return [...props.stocks]
    .filter((s) => s.amount > 0)
    .sort((a, b) => b.amount - a.amount)
    .slice(0, 10);
});

function rankClass(idx: number): string {
  if (idx < 3) return "rank-top";
  return "rank-normal";
}
</script>

<style scoped>
.hot-stocks {
  padding: var(--spacing-md);
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--font-size-sm);
}

th {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  color: var(--color-text-tertiary);
  font-weight: 500;
  white-space: nowrap;
}

th:first-child,
td:first-child {
  text-align: center;
}

th:nth-child(2),
td:nth-child(2) {
  text-align: left;
}

td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  border-bottom: 1px solid var(--color-bg-secondary);
}

.rank-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: var(--radius-sm);
  font-size: var(--font-size-xs);
  font-weight: 700;
}

.rank-top {
  background: var(--color-up);
  color: #fff;
}

.rank-normal {
  background: var(--color-bg-secondary);
  color: var(--color-text-tertiary);
}

.stock-name {
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
}

.stock-code {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
}

.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}
</style>
