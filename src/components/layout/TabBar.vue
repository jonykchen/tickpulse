<template>
  <div class="tab-bar titlebar-drag">
    <div class="tab-bar__nav titlebar-no-drag">
      <router-link
        v-for="tab in tabs"
        :key="tab.path"
        :to="tab.path"
        class="tab-bar__item"
        active-class="tab-bar__item--active"
      >
        {{ tab.label }}
      </router-link>
    </div>
    <div class="tab-bar__status titlebar-no-drag">
      <MarketPhaseBar />
    </div>
  </div>
</template>

<script setup lang="ts">
import MarketPhaseBar from "./MarketPhaseBar.vue";

const tabs = [
  { path: "/", label: "自选" },
  { path: "/market", label: "行情" },
  { path: "/position", label: "持仓" },
  { path: "/anomaly", label: "异动" },
  { path: "/settings", label: "设置" },
];
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 var(--spacing-lg);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-bg-tertiary);
}
.tab-bar__nav {
  display: flex;
  gap: var(--spacing-lg);
}
.tab-bar__item {
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: var(--font-size-md);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  transition: color var(--transition-fast);
}
.tab-bar__item:hover {
  color: var(--color-text-primary);
}
.tab-bar__item--active {
  color: var(--color-primary);
  font-weight: 600;
}
.tab-bar__status {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}
</style>
