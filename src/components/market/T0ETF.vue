<template>
  <div class="t0-etf">
    <table>
      <thead>
        <tr>
          <th class="col-name">名称</th>
          <th class="col-price">最新价</th>
          <th class="col-change">涨跌幅</th>
          <th class="col-amount">成交额</th>
          <th class="col-turnover">换手率</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="etf in t0Etfs" :key="etf.secid">
          <td class="col-name">
            <div class="etf-name">{{ etf.name }}</div>
            <div class="etf-code num">{{ etf.code }}</div>
          </td>
          <td
            class="col-price num"
            :class="getChangeColorClass(etf.changePercent)"
          >
            {{ formatPrice(etf.price) }}
          </td>
          <td
            class="col-change num"
            :class="getChangeColorClass(etf.changePercent)"
          >
            {{ formatChangePercent(etf.changePercent) }}
          </td>
          <td class="col-amount num">{{ formatAmount(etf.amount) }}</td>
          <td class="col-turnover num">{{ etf.turnoverRate.toFixed(2) }}%</td>
        </tr>
        <tr v-if="t0Etfs.length === 0">
          <td colspan="5" class="empty-state">暂无T+0 ETF数据</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { StockQuote } from "@/types/stock";
import {
  formatPrice,
  formatChangePercent,
  formatAmount,
  getChangeColorClass,
} from "@/lib/format";

const props = defineProps<{
  stocks: StockQuote[];
}>();

/**
 * T+0 ETF code prefixes:
 * - 5xxxxx (SH): bond ETFs, gold ETFs, cross-border ETFs
 * - 1xxxxx (SZ): some bond/money market ETFs
 * - Common T+0 types: bond ETF (债券), gold (黄金), cross-border (跨境),
 *   commodity (商品), money market (货币)
 *
 * We filter by code prefix as a heuristic. A more precise approach would
 * use a dedicated ETF type field from backend data.
 */
const T0_ETF_PREFIXES = ["511", "518", "513", "515", "1599", "1590"];

const t0Etfs = computed(() => {
  return props.stocks.filter((s) => {
    const code = s.code;
    return T0_ETF_PREFIXES.some((prefix) => code.startsWith(prefix));
  });
});
</script>

<style scoped>
.t0-etf {
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
  text-align: left;
}

td {
  padding: var(--spacing-sm) var(--spacing-md);
  text-align: right;
  border-bottom: 1px solid var(--color-bg-secondary);
}

.etf-name {
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
}

.etf-code {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-xs);
}

.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}
</style>
