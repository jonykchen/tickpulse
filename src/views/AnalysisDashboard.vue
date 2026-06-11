<template>
  <div class="analysis-dashboard">
    <NTabs type="line" animated>
      <NTabPane name="peg" tab="PEG看板">
        <PegBoard />
      </NTabPane>
      <NTabPane name="report" tab="分析报告">
        <AnalysisReport />
      </NTabPane>
      <NTabPane name="industry" tab="行业对比">
        <IndustryPegComparison />
      </NTabPane>
      <NTabPane name="history" tab="历史记录">
        <AnalysisHistory />
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane } from "naive-ui";
import PegBoard from "./PegBoard.vue";
import AnalysisReport from "./AnalysisReport.vue";
import IndustryPegComparison from "./IndustryPegComparison.vue";
import AnalysisHistory from "./AnalysisHistory.vue";
</script>

<style scoped>
.analysis-dashboard {
  height: 100%;
  padding: var(--spacing-md);
}
</style>
