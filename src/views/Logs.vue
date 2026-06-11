<template>
  <div class="logs-page">
    <LogViewer />
  </div>
</template>

<script setup lang="ts">
import LogViewer from "@/components/common/LogViewer.vue";
</script>

<style scoped>
.logs-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
