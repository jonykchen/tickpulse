<template>
  <div class="settings-view">
    <h2>设置</h2>

    <section class="setting-section">
      <h3>通用</h3>
      <div class="setting-item">
        <span>主题</span>
        <select v-model="theme" @change="onThemeChange">
          <option value="dark">深色</option>
          <option value="light">浅色</option>
        </select>
      </div>
      <div class="setting-item">
        <span>开机自启</span>
        <input type="checkbox" v-model="autoStart" @change="onAutoStartChange" />
      </div>
      <div class="setting-item">
        <span>最小化到托盘</span>
        <input type="checkbox" v-model="minimizeToTray" @change="onMinimizeToTrayChange" />
      </div>
    </section>

    <section class="setting-section">
      <h3>预警</h3>
      <div class="setting-item">
        <span>启用预警</span>
        <input type="checkbox" v-model="alertEnabled" @change="onAlertEnabledChange" />
      </div>
      <div class="setting-item">
        <span>预警声音</span>
        <input type="checkbox" v-model="alertSound" @change="onAlertSoundChange" />
      </div>
    </section>

    <section class="setting-section">
      <h3>诊断</h3>
      <div class="health-info">
        <div class="health-item">
          <span>内存占用</span>
          <span class="num">{{ health.memory_usage_mb.toFixed(1) }} MB</span>
        </div>
        <div class="health-item">
          <span>数据库大小</span>
          <span class="num">{{ health.db_size_mb.toFixed(2) }} MB</span>
        </div>
        <div class="health-item">
          <span>API延迟</span>
          <span class="num">{{ health.api_latency_ms }} ms</span>
        </div>
        <div class="health-item">
          <span>缓存命中率</span>
          <span class="num">{{ (health.cache_hit_rate * 100).toFixed(1) }}%</span>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useConfigStore } from "@/stores/config";
import { getHealthMetrics } from "@/lib/tauri";

const configStore = useConfigStore();

const theme = ref(configStore.theme);
const autoStart = ref(configStore.autoStart);
const minimizeToTray = ref(configStore.minimizeToTray);
const alertEnabled = ref(configStore.alertEnabled);
const alertSound = ref(configStore.alertSound);
const health = ref({ memory_usage_mb: 0, db_size_mb: 0, api_latency_ms: 0, cache_hit_rate: 0, consecutive_errors: 0, last_success_at: null, active_sources: [] as string[], cpu_usage_percent: 0 });

function onThemeChange() { configStore.updateSetting("theme", theme.value); }
function onAutoStartChange() { configStore.updateSetting("autoStart", String(autoStart.value)); }
function onMinimizeToTrayChange() { configStore.updateSetting("minimizeToTray", String(minimizeToTray.value)); }
function onAlertEnabledChange() { configStore.updateSetting("alertEnabled", String(alertEnabled.value)); }
function onAlertSoundChange() { configStore.updateSetting("alertSound", String(alertSound.value)); }

onMounted(async () => {
  try {
    health.value = await getHealthMetrics();
  } catch {}
});
</script>

<style scoped>
.settings-view {
  padding: var(--spacing-lg);
  max-width: 600px;
}
.setting-section {
  margin-bottom: var(--spacing-xl);
}
.setting-section h3 {
  font-size: var(--font-size-lg);
  margin-bottom: var(--spacing-md);
  color: var(--color-text-secondary);
}
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) 0;
}
select {
  background: var(--color-surface);
  border: 1px solid var(--color-bg-tertiary);
  border-radius: var(--radius-sm);
  color: var(--color-text-primary);
  padding: var(--spacing-xs) var(--spacing-md);
}
.health-info {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}
.health-item {
  display: flex;
  justify-content: space-between;
  padding: var(--spacing-sm) 0;
  font-size: var(--font-size-sm);
}
</style>