<template>
  <div class="position-table-wrapper">
    <table class="position-table">
      <thead>
        <tr>
          <th>名称</th>
          <th>成本价</th>
          <th>数量</th>
          <th>现价</th>
          <th>市值</th>
          <th>浮动盈亏</th>
          <th>盈亏率</th>
          <th>当日盈亏</th>
          <th>仓位占比</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="pos in positionsWithQuote" :key="pos.id">
          <td>
            <div class="stock-name">{{ pos.name || pos.secid }}</div>
          </td>
          <td class="num">{{ pos.costPrice }}</td>
          <td class="num">{{ pos.quantity }}</td>
          <td class="num" :class="priceClass(pos)">{{ formatPrice(pos.currentPrice) }}</td>
          <td class="num">{{ formatAmount(pos.marketValue) }}</td>
          <td class="num" :class="pnlClass(pos.floatPnl)">{{ formatPnl(pos.floatPnl) }}</td>
          <td class="num" :class="pnlClass(pos.floatPnl)">{{ formatPnlRate(pos.pnlRate) }}</td>
          <td class="num" :class="pnlClass(pos.todayPnl)">{{ formatPnl(pos.todayPnl) }}</td>
          <td class="num">{{ pos.ratio.toFixed(1) }}%</td>
          <td>
            <NButton size="tiny" quaternary @click="$emit('editPosition', pos)">编辑</NButton>
          </td>
        </tr>
      </tbody>
      <tfoot v-if="summary">
        <tr class="summary-row">
          <td><strong>合计</strong></td>
          <td></td>
          <td></td>
          <td></td>
          <td class="num">{{ summary.totalMarketValue }}</td>
          <td class="num" :class="pnlClass(parseFloat(summary.totalFloatPnl))">{{ summary.totalFloatPnl }}</td>
          <td class="num" :class="pnlClass(parseFloat(summary.totalFloatPnl))">{{ summary.totalPnlRate }}%</td>
          <td class="num" :class="pnlClass(parseFloat(summary.totalTodayPnl))">{{ summary.totalTodayPnl }}</td>
          <td class="num">{{ summary.todayPnlRate }}%</td>
          <td></td>
        </tr>
        <tr v-if="excessReturn !== null" class="excess-return-row">
          <td colspan="10" class="excess-return-cell">
            <span class="excess-return-label">超额收益：</span>
            <span :class="excessReturn >= 0 ? 'text-up' : 'text-down'" class="excess-return-value">
              {{ excessReturn >= 0 ? '+' : '' }}{{ excessReturn.toFixed(2) }}%
              {{ excessReturn >= 0 ? '跑赢' : '跑输' }}沪深300
            </span>
          </td>
        </tr>
      </tfoot>
    </table>
    <div v-if="positions.length === 0" class="empty-state">
      <NEmpty description="暂无持仓" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NButton, NEmpty } from "naive-ui";
import type { StockQuote } from "@/types/stock";
import type { PortfolioSummary } from "@/types/portfolio";
import { formatPrice, formatAmount, formatPnlAmount } from "@/lib/format";

const props = defineProps<{
  positions: any[];
  quotes: StockQuote[];
  summary: PortfolioSummary | null;
}>();

defineEmits<{
  editPosition: [position: any];
}>();

interface PositionRow {
  id: number;
  secid: string;
  name: string;
  costPrice: string;
  quantity: string;
  currentPrice: number;
  marketValue: number;
  floatPnl: number;
  pnlRate: number;
  todayPnl: number;
  ratio: number;
}

const totalMarketValue = computed(() => {
  return positionsWithQuote.value.reduce((sum, p) => sum + p.marketValue, 0);
});

const excessReturn = computed(() => {
  const summary = props.summary;
  if (!summary?.benchmarkChange) return null;

  // 超额收益 = 今日收益率 - 基准涨跌幅
  const todayReturn = parseFloat(summary.todayPnlRate) || 0;
  const benchmarkReturn = parseFloat(summary.benchmarkChange) || 0;
  return todayReturn - benchmarkReturn;
});

const positionsWithQuote = computed<PositionRow[]>(() => {
  return props.positions.map((pos) => {
    const quote = props.quotes.find((q) => q.secid === pos.secid);
    const price = quote?.price ?? 0;
    const cost = parseFloat(pos.costPrice) || 0;
    const qty = parseInt(pos.quantity) || 0;
    const prevClose = quote?.preClose ?? 0;
    const mv = price * qty;
    const floatPnl = (price - cost) * qty;
    const pnlRate = cost > 0 ? ((price - cost) / cost) * 100 : 0;
    const todayPnl = (price - prevClose) * qty;

    return {
      id: pos.id,
      secid: pos.secid,
      name: pos.name || pos.secid,
      costPrice: pos.costPrice,
      quantity: pos.quantity,
      currentPrice: price,
      marketValue: mv,
      floatPnl,
      pnlRate,
      todayPnl,
      ratio: totalMarketValue.value > 0 ? (mv / totalMarketValue.value) * 100 : 0,
    };
  });
});

function priceClass(pos: PositionRow) {
  if (pos.currentPrice > 0 && pos.floatPnl > 0) return "text-up";
  if (pos.currentPrice > 0 && pos.floatPnl < 0) return "text-down";
  return "";
}

function pnlClass(val: number) {
  if (val > 0) return "text-up";
  if (val < 0) return "text-down";
  return "";
}

function formatPnl(val: number): string {
  if (val === 0) return "--";
  const prefix = val > 0 ? "+" : "";
  return `${prefix}${formatAmount(val)}`;
}

function formatPnlRate(rate: number): string {
  if (rate === 0) return "--";
  const prefix = rate > 0 ? "+" : "";
  return `${prefix}${rate.toFixed(2)}%`;
}
</script>

<style scoped>
.position-table-wrapper {
  width: 100%;
  overflow-x: auto;
}
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
  white-space: nowrap;
  position: sticky;
  top: 0;
  background: var(--color-bg);
  z-index: 1;
}
th:first-child, td:first-child {
  text-align: left;
}
.stock-name {
  color: var(--color-text-primary);
}
.summary-row {
  background: var(--color-bg-secondary);
  font-weight: 600;
}
.summary-row td {
  border-top: 2px solid var(--color-border);
}
.excess-return-row {
  background: var(--color-bg-secondary);
}
.excess-return-cell {
  text-align: left !important;
  padding: var(--spacing-sm) var(--spacing-md);
}
.excess-return-label {
  color: var(--color-text-tertiary);
  margin-right: var(--spacing-xs);
}
.excess-return-value {
  font-weight: 500;
}
.empty-state {
  padding: var(--spacing-xl) 0;
}
</style>
