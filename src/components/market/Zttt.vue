<template>
  <div class="zttt">
    <div v-for="group in limitUpGroups" :key="group.days" class="ladder-group">
      <div class="ladder-header">
        <span class="ladder-days">{{ group.days }}连板</span>
        <span class="ladder-count">({{ group.stocks.length }})</span>
      </div>
      <table>
        <thead>
          <tr>
            <th class="col-name">名称</th>
            <th class="col-code">代码</th>
            <th class="col-seal">封板强度</th>
            <th class="col-break">炸板次数</th>
            <th class="col-amount">成交额</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="stock in group.stocks" :key="stock.secid">
            <td class="col-name">
              <span class="stock-name text-up">{{ stock.name }}</span>
            </td>
            <td class="col-code num">{{ stock.code }}</td>
            <td class="col-seal num">
              <span :class="sealClass(stock.sealStrength)">
                {{ formatSealStrength(stock.sealStrength) }}
              </span>
            </td>
            <td class="col-break num">
              <span :class="stock.sealBreakCount > 0 ? 'text-warning' : 'text-flat'">
                {{ stock.sealBreakCount }}
              </span>
            </td>
            <td class="col-amount num">{{ formatAmount(stock.amount) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-if="limitUpGroups.length === 0" class="empty-state">
      今日无涨停股
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { StockQuote } from "@/types/stock";
import { formatAmount } from "@/lib/format";

const props = defineProps<{
  stocks: StockQuote[];
}>();

interface LimitUpGroup {
  days: number;
  stocks: StockQuote[];
}

const limitUpGroups = computed(() => {
  const limitUpStocks = props.stocks.filter((s) => s.isLimitUp);
  if (limitUpStocks.length === 0) return [];

  // Group by consecutive limit-up days (approximated by sealBreakCount as proxy)
  // Since StockQuote doesn't have a direct "consecutive days" field,
  // we use a simple grouping: 1-day (sealBreakCount > 0 or no extra info)
  // and multi-day (sealBreakCount === 0 suggests strong consecutive limit-up)
  // For a proper implementation, this would come from backend data.
  // Here we group by estimated consecutive days using a simple heuristic.
  const groups: LimitUpGroup[] = [];
  const dayMap = new Map<number, StockQuote[]>();

  for (const s of limitUpStocks) {
    // Use sealBreakCount as a rough proxy - stocks with 0 breaks are more likely
    // to be consecutive limit-up. Real implementation would use a dedicated field.
    const days = estimateConsecutiveDays(s);
    if (!dayMap.has(days)) {
      dayMap.set(days, []);
    }
    dayMap.get(days)!.push(s);
  }

  // Sort groups by days descending
  const sortedKeys = Array.from(dayMap.keys()).sort((a, b) => b - a);
  for (const days of sortedKeys) {
    const groupStocks = dayMap.get(days)!;
    // Within group, sort by sealStrength descending
    groupStocks.sort((a, b) => (b.sealStrength ?? 0) - (a.sealStrength ?? 0));
    groups.push({ days, stocks: groupStocks });
  }

  return groups;
});

/** Estimate consecutive limit-up days based on available data.
 *  This is a placeholder heuristic - real data should come from backend. */
function estimateConsecutiveDays(stock: StockQuote): number {
  // Simple heuristic: if sealBreakCount is 0 and sealStrength > 0.8, assume 2+ days
  // Otherwise assume 1 day. Real implementation would use a dedicated field.
  if (stock.sealStrength && stock.sealStrength > 0.9 && stock.sealBreakCount === 0) {
    return 3;
  }
  if (stock.sealStrength && stock.sealStrength > 0.7 && stock.sealBreakCount === 0) {
    return 2;
  }
  return 1;
}

function formatSealStrength(val: number | null): string {
  if (val === null || val === undefined) return "--";
  return (val * 100).toFixed(0) + "%";
}

function sealClass(val: number | null): string {
  if (val === null || val === undefined) return "text-flat";
  if (val >= 0.8) return "seal-strong";
  if (val >= 0.5) return "seal-medium";
  return "seal-weak";
}
</script>

<style scoped>
.zttt {
  padding: var(--spacing-md);
}

.ladder-group {
  margin-bottom: var(--spacing-lg);
}

.ladder-group:last-child {
  margin-bottom: 0;
}

.ladder-header {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-xs);
  margin-bottom: var(--spacing-sm);
  padding-bottom: var(--spacing-xs);
  border-bottom: 1px solid var(--color-bg-secondary);
}

.ladder-days {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--color-limit-up);
}

.ladder-count {
  font-size: var(--font-size-xs);
  color: var(--color-text-tertiary);
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

.stock-name {
  font-size: var(--font-size-md);
}

.seal-strong {
  color: var(--color-seal-strong);
  font-weight: 700;
}

.seal-medium {
  color: var(--color-warning);
}

.seal-weak {
  color: var(--color-seal-weak);
}

.text-warning {
  color: var(--color-warning);
}

.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}
</style>
