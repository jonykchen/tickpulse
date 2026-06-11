<template>
  <NDrawer
    :show="pageStore.klineDrawer.visible"
    :width="600"
    placement="right"
    @update:show="(v: boolean) => { if (!v) pageStore.closeKline() }"
  >
    <NDrawerContent :title="`${pageStore.klineDrawer.name} K线`">
      <div class="kline-drawer">
        <!-- 周期切换 -->
        <NRadioGroup
          v-model:value="pageStore.klineDrawer.period"
          size="small"
          class="period-switch"
        >
          <NRadioButton value="1m">1分</NRadioButton>
          <NRadioButton value="5m">5分</NRadioButton>
          <NRadioButton value="15m">15分</NRadioButton>
          <NRadioButton value="30m">30分</NRadioButton>
          <NRadioButton value="60m">60分</NRadioButton>
          <NRadioButton value="daily">日K</NRadioButton>
          <NRadioButton value="week">周K</NRadioButton>
          <NRadioButton value="month">月K</NRadioButton>
        </NRadioGroup>

        <!-- 复权切换 -->
        <NRadioGroup
          v-model:value="pageStore.klineDrawer.adjust"
          size="small"
          class="adjust-switch"
        >
          <NRadioButton value="forward">前复权</NRadioButton>
          <NRadioButton value="backward">后复权</NRadioButton>
          <NRadioButton value="none">不复权</NRadioButton>
        </NRadioGroup>

        <!-- K线图 -->
        <div v-if="secid" class="chart-area">
          <KlineChart
            :secid="secid"
            :period="pageStore.klineDrawer.period"
            :adjust="pageStore.klineDrawer.adjust"
          />
        </div>
      </div>
    </NDrawerContent>
  </NDrawer>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { NDrawer, NDrawerContent, NRadioGroup, NRadioButton } from "naive-ui";
import { usePageDataStore } from "@/stores/page";
import KlineChart from "@/components/charts/KlineChart.vue";

const pageStore = usePageDataStore();
const secid = computed(() => pageStore.klineDrawer.secid);
</script>

<style scoped>
.kline-drawer {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
}
.period-switch,
.adjust-switch {
  display: flex;
}
.chart-area {
  flex: 1;
  min-height: 400px;
}
</style>
