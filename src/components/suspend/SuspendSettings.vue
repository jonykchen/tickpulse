<template>
  <div class="suspend-settings">
    <NCard title="悬浮窗配置" size="small">
      <NSpace vertical>
        <!-- 透明度 -->
        <div class="setting-item">
          <span class="label">透明度</span>
          <NSlider
            v-model:value="localConfig.opacity"
            :min="0.1"
            :max="1"
            :step="0.1"
            :tooltip="true"
            @update:value="debounceSave"
          />
        </div>

        <!-- 显示数量 -->
        <div class="setting-item">
          <span class="label">显示数量</span>
          <NInputNumber
            v-model:value="localConfig.colorNum"
            :min="1"
            :max="20"
            @update:value="debounceSave"
          />
        </div>

        <!-- 背景色 -->
        <div class="setting-item">
          <span class="label">背景色</span>
          <NColorPicker
            v-model:value="localConfig.bgColor"
            :modes="['hex']"
            :show-alpha="true"
            @update:value="debounceSave"
          />
        </div>

        <!-- 显示类型 -->
        <div class="setting-item">
          <span class="label">显示类型</span>
          <NSelect
            v-model:value="localConfig.showEnum"
            :options="showEnumOptions"
            @update:value="debounceSave"
          />
        </div>

        <!-- 刷新频率 -->
        <div class="setting-item">
          <span class="label">刷新频率(秒)</span>
          <NInputNumber
            v-model:value="localConfig.refreshRate"
            :min="3"
            :max="300"
            @update:value="debounceSave"
          />
        </div>

        <!-- 排序方式 -->
        <div class="setting-item">
          <span class="label">排序方式</span>
          <NSelect
            v-model:value="localConfig.sortType"
            :options="sortTypeOptions"
            @update:value="debounceSave"
          />
        </div>

        <!-- 显示微信按钮 -->
        <div class="setting-item">
          <span class="label">显示微信按钮</span>
          <NSwitch
            v-model:value="localConfig.showWechat"
            @update:value="debounceSave"
          />
        </div>

        <!-- 分组配置 -->
        <NDivider>分组配置</NDivider>

        <div class="setting-item">
          <span class="label">显示分组</span>
          <NSwitch
            v-model:value="localConfig.group.show"
            @update:value="debounceSave"
          />
        </div>

        <template v-if="localConfig.group.show">
          <div class="setting-item">
            <span class="label">分组类型</span>
            <NSelect
              v-model:value="localConfig.group.type"
              :options="groupTypeOptions"
              @update:value="debounceSave"
            />
          </div>

          <div class="setting-item">
            <span class="label">分组索引</span>
            <NInputNumber
              v-model:value="localConfig.group.index"
              :min="0"
              :max="100"
              @update:value="debounceSave"
            />
          </div>
        </template>

        <!-- 重置按钮 -->
        <div class="setting-actions">
          <NButton size="small" @click="resetConfig">恢复默认</NButton>
        </div>
      </NSpace>
    </NCard>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import {
  NCard,
  NSpace,
  NSlider,
  NInputNumber,
  NColorPicker,
  NSelect,
  NSwitch,
  NDivider,
  NButton,
} from "naive-ui";
import { useConfigStore } from "@/stores/config";
import { DEFAULT_SUSPEND_CONFIG, type SuspendConfig } from "@/types/config";

const configStore = useConfigStore();

// Local config state
const localConfig = ref<SuspendConfig>({ ...DEFAULT_SUSPEND_CONFIG });

// Debounce timer
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Select options
const showEnumOptions = [
  { label: "全部股票", value: "all" },
  { label: "自选股", value: "self" },
  { label: "持仓股", value: "position" },
];

const sortTypeOptions = [
  { label: "涨跌幅", value: "change" },
  { label: "成交额", value: "amount" },
  { label: "换手率", value: "turnover" },
];

const groupTypeOptions = [
  { label: "行业板块", value: "industry" },
  { label: "概念板块", value: "concept" },
  { label: "自定分组", value: "self" },
];

// Debounce save function
function debounceSave(): void {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    saveConfig();
  }, 300);
}

// Save config to store
function saveConfig(): void {
  configStore.updateSetting("suspend", { ...localConfig.value });
}

// Reset to defaults
function resetConfig(): void {
  localConfig.value = { ...DEFAULT_SUSPEND_CONFIG };
  saveConfig();
}

// Load config on mount
onMounted(() => {
  localConfig.value = configStore.mergeSuspendConfig(configStore.suspendConfig);
});

// Watch for external config changes
watch(
  () => configStore.suspendConfig,
  (newConfig) => {
    localConfig.value = configStore.mergeSuspendConfig(newConfig);
  },
  { deep: true }
);
</script>

<style scoped>
.suspend-settings {
  padding: var(--spacing-sm);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
}

.setting-item .label {
  min-width: 100px;
  color: var(--color-text-secondary);
}

.setting-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: var(--spacing-md);
}

:deep(.n-slider) {
  flex: 1;
  max-width: 200px;
}

:deep(.n-input-number) {
  width: 100px;
}

:deep(.n-select) {
  width: 120px;
}

:deep(.n-color-picker) {
  width: 100px;
}
</style>
