<template>
  <NDrawer
    :show="pageStore.etfDrawer.visible"
    :width="500"
    placement="right"
    @update:show="(v: boolean) => { if (!v) pageStore.closeEtf() }"
  >
    <NDrawerContent :title="`${pageStore.etfDrawer.name} ETF持仓`">
      <div class="etf-drawer">
        <NEmpty description="ETF持仓数据加载中..." />
      </div>
    </NDrawerContent>
  </NDrawer>
</template>

<script setup lang="ts">
import { NDrawer, NDrawerContent, NEmpty } from "naive-ui";
import { usePageDataStore } from "@/stores/page";

const pageStore = usePageDataStore();
</script>
