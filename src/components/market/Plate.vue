<template>
  <div class="plate">
    <NTabs type="line" size="small" animated>
      <NTabPane name="industry" tab="行业">
        <table>
          <thead>
            <tr>
              <th class="col-name">名称</th>
              <th class="col-change">涨跌幅</th>
              <th class="col-cap">总市值</th>
              <th class="col-inflow">主力净流入</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in industryPlates" :key="item.name">
              <td class="col-name">{{ item.name }}</td>
              <td
                class="col-change num"
                :class="getChangeColorClass(item.changePercent)"
              >
                {{ formatChangePercent(item.changePercent) }}
              </td>
              <td class="col-cap num">{{ formatMarketCap(item.totalMarketCap) }}</td>
              <td
                class="col-inflow num"
                :class="getChangeColorClass(item.mainNetInflow)"
              >
                {{ formatPnlAmount(item.mainNetInflow) }}
              </td>
            </tr>
            <tr v-if="industryPlates.length === 0">
              <td colspan="4" class="empty-state">暂无数据</td>
            </tr>
          </tbody>
        </table>
      </NTabPane>
      <NTabPane name="concept" tab="概念">
        <table>
          <thead>
            <tr>
              <th class="col-name">名称</th>
              <th class="col-change">涨跌幅</th>
              <th class="col-cap">总市值</th>
              <th class="col-inflow">主力净流入</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in conceptPlates" :key="item.name">
              <td class="col-name">{{ item.name }}</td>
              <td
                class="col-change num"
                :class="getChangeColorClass(item.changePercent)"
              >
                {{ formatChangePercent(item.changePercent) }}
              </td>
              <td class="col-cap num">{{ formatMarketCap(item.totalMarketCap) }}</td>
              <td
                class="col-inflow num"
                :class="getChangeColorClass(item.mainNetInflow)"
              >
                {{ formatPnlAmount(item.mainNetInflow) }}
              </td>
            </tr>
            <tr v-if="conceptPlates.length === 0">
              <td colspan="4" class="empty-state">暂无数据</td>
            </tr>
          </tbody>
        </table>
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane } from "naive-ui";
import {
  formatChangePercent,
  formatMarketCap,
  formatPnlAmount,
  getChangeColorClass,
} from "@/lib/format";

/** 板块数据项 */
export interface PlateItem {
  name: string;
  changePercent: number;
  totalMarketCap: number;
  mainNetInflow: number;
}

defineProps<{
  industryPlates: PlateItem[];
  conceptPlates: PlateItem[];
}>();
</script>

<style scoped>
.plate {
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

.col-name {
  color: var(--color-text-primary);
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-state {
  text-align: center;
  padding: var(--spacing-xl) 0;
  color: var(--color-text-tertiary);
}
</style>
