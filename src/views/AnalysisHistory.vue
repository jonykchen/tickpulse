<template>
  <div class="analysis-history">
    <NEmpty v-if="historyList.length === 0" description="暂无分析记录" />
    <div v-else class="history-list">
      <div v-for="item in historyList" :key="item.id" class="history-item" @click="viewDetail(item)">
        <div class="history-header">
          <span class="history-name">{{ item.stockName }}</span>
          <span class="history-date">{{ formatDate(item.createdAt) }}</span>
        </div>
        <div class="history-summary">
          {{ OVERALL_RATING_LABELS[item.overallRating as OverallRating] ?? item.overallRating }}
          · {{ item.overallScore.toFixed(0) }}分
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NEmpty } from "naive-ui";
import { useAnalysisStore } from "@/stores/analysis";
import { OverallRating, OVERALL_RATING_LABELS } from "@/types/analysis";
import type { AnalysisResult } from "@/types/analysis";

const analysisStore = useAnalysisStore();
const historyList = computed(() => analysisStore.latestResults);

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString("zh-CN");
}

function viewDetail(item: AnalysisResult) {
  // TODO: 跳转到分析报告详情
}
</script>

<style scoped>
.analysis-history {
  padding: var(--spacing-md);
}
.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.history-item {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: background 0.2s;
}
.history-item:hover {
  background: var(--color-surface-hover);
}
.history-header {
  display: flex;
  justify-content: space-between;
}
.history-name {
  font-weight: 600;
  color: var(--color-text-primary);
}
.history-date {
  font-size: 12px;
  color: var(--color-text-tertiary);
}
.history-summary {
  margin-top: 4px;
  font-size: 12px;
  color: var(--color-text-secondary);
}
</style>
