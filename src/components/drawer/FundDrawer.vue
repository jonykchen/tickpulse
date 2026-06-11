<template>
  <NDrawer
    :show="pageStore.fundDrawer.visible"
    :width="500"
    placement="right"
    @update:show="(v: boolean) => { if (!v) pageStore.closeFund() }"
  >
    <NDrawerContent :title="`${pageStore.fundDrawer.name} 基金估值`">
      <div class="fund-drawer">
        <NEmpty description="基金数据加载中..." />
      </div>
    </NDrawerContent>
  </NDrawer>
</template>

<script setup lang="ts">
import { NDrawer, NDrawerContent, NEmpty } from "naive-ui";
import { usePageDataStore } from "@/stores/page";

const pageStore = usePageDataStore();
</script>