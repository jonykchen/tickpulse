<template>
  <div class="app-layout">
    <TabBar />
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import TabBar from "@/components/layout/TabBar.vue";
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.main-content {
  flex: 1;
  overflow: auto;
}
</style>
