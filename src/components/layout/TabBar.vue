<template>
  <div class="tab-bar">
    <nav class="tab-bar__nav" ref="navRef">
      <router-link
        v-for="tab in tabs"
        :key="tab.path"
        :to="tab.path"
        class="tab-item"
        :ref="(el) => setTabRef(el, tab.path)"
        @mouseenter="updateIndicator(tab.path)"
      >
        {{ tab.label }}
      </router-link>
      <span
        class="tab-indicator"
        :style="indicatorStyle"
      ></span>
    </nav>
    <div class="tab-bar__status">
      <MarketPhaseBar />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { useRoute } from "vue-router";
import MarketPhaseBar from "./MarketPhaseBar.vue";

const tabs = [
  { path: "/", label: "自选" },
  { path: "/market", label: "行情" },
  { path: "/position", label: "持仓" },
  { path: "/anomaly", label: "异动" },
  { path: "/logs", label: "日志" },
  { path: "/settings", label: "设置" },
];

const route = useRoute();
const navRef = ref<HTMLElement | null>(null);
const tabRefs = ref<Map<string, HTMLElement>>(new Map());
const indicatorStyle = ref({ left: "0px", width: "0px" });
const hoveredPath = ref<string | null>(null);

function setTabRef(el: unknown, path: string) {
  if (el instanceof HTMLElement) {
    tabRefs.value.set(path, el);
  }
}

function updateIndicator(path: string) {
  hoveredPath.value = path;
}

function calculateIndicatorStyle() {
  const activePath = hoveredPath.value || route.path;
  const el = tabRefs.value.get(activePath);
  if (el && navRef.value) {
    const navRect = navRef.value.getBoundingClientRect();
    const elRect = el.getBoundingClientRect();
    indicatorStyle.value = {
      left: `${elRect.left - navRect.left}px`,
      width: `${elRect.width}px`,
    };
  }
}

watch([route, hoveredPath], () => {
  nextTick(calculateIndicatorStyle);
});

onMounted(() => {
  nextTick(calculateIndicatorStyle);
});
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 var(--spacing-lg);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
}
.tab-bar__nav {
  display: flex;
  gap: var(--spacing-lg);
  position: relative;
}
.tab-item {
  color: var(--color-text-secondary);
  text-decoration: none;
  font-size: var(--font-size-md);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  transition: all var(--duration-fast) var(--ease-out);
  position: relative;
}
.tab-item:hover {
  color: var(--color-text-primary);
  transform: scale(1.05);
}
.tab-item.router-link-active {
  color: var(--color-primary);
  font-weight: 600;
}
.tab-indicator {
  position: absolute;
  bottom: -1px;
  height: 2px;
  background: var(--color-primary);
  border-radius: 1px;
  transition: all var(--duration-normal) var(--ease-spring);
  pointer-events: none;
}
.tab-bar__status {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}
</style>
