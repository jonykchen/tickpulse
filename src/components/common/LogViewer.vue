<template>
  <div class="log-viewer">
    <div class="log-toolbar">
      <select v-model="levelFilter" class="log-filter">
        <option value="all">全部</option>
        <option value="DEBUG">DEBUG</option>
        <option value="INFO">INFO</option>
        <option value="WARN">WARN</option>
        <option value="ERROR">ERROR</option>
      </select>
      <input
        v-model="searchKeyword"
        placeholder="搜索关键词..."
        class="log-search"
      />
      <span class="log-count">{{ filteredLogs.length }} 条</span>
      <button @click="refreshLogs" class="log-btn">刷新</button>
      <button @click="handleClearLogs" class="log-btn log-btn-danger">清空</button>
      <label class="log-autoscroll">
        <input type="checkbox" v-model="autoScroll" />
        自动滚动
      </label>
    </div>
    <div class="log-content" ref="logContainer">
      <div
        v-for="(log, index) in filteredLogs"
        :key="index"
        :class="['log-entry', `log-${log.level.toLowerCase()}`]"
      >
        <span class="log-time">{{ formatTime(log.timestamp) }}</span>
        <span class="log-level">{{ log.level }}</span>
        <span class="log-target">{{ log.target.split('::').pop() }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
      <div v-if="filteredLogs.length === 0" class="log-empty">
        暂无日志记录
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from "vue";
import { getRecentLogs, clearLogs } from "@/lib/tauri";
import type { LogEntry, LogLevel } from "@/types/log";

const logs = ref<LogEntry[]>([]);
const levelFilter = ref<LogLevel>("all");
const searchKeyword = ref("");
const logContainer = ref<HTMLElement | null>(null);
const autoScroll = ref(true);

const filteredLogs = computed(() => {
  let result = logs.value;

  if (levelFilter.value !== "all") {
    result = result.filter((log) => log.level === levelFilter.value);
  }

  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    result = result.filter(
      (log) =>
        log.message.toLowerCase().includes(keyword) ||
        log.target.toLowerCase().includes(keyword)
    );
  }

  return result;
});

async function refreshLogs() {
  try {
    logs.value = await getRecentLogs(500);
    if (autoScroll.value) {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
      });
    }
  } catch (e) {
    console.error("获取日志失败:", e);
  }
}

async function handleClearLogs() {
  if (confirm("确定要清空日志缓冲区吗？")) {
    try {
      await clearLogs();
      logs.value = [];
    } catch (e) {
      console.error("清空日志失败:", e);
    }
  }
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  });
}

let refreshInterval: ReturnType<typeof setInterval>;

onMounted(() => {
  refreshLogs();
  refreshInterval = setInterval(refreshLogs, 2000);
});

onUnmounted(() => {
  clearInterval(refreshInterval);
});

// 监听自动滚动
watch(autoScroll, (val) => {
  if (val) {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight;
      }
    });
  }
});
</script>

<style scoped>
.log-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg);
}

.log-toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  flex-wrap: wrap;
}

.log-filter,
.log-search {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-sm);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  outline: none;
  transition: border-color var(--duration-fast);
}

.log-filter:focus,
.log-search:focus {
  border-color: var(--color-primary);
}

.log-search {
  flex: 1;
  min-width: 150px;
}

.log-count {
  color: var(--color-text-tertiary);
  font-size: var(--font-size-sm);
}

.log-btn {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-xs) var(--spacing-md);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all var(--duration-fast);
}

.log-btn:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.log-btn-danger:hover {
  background: var(--color-danger);
  border-color: var(--color-danger);
}

.log-autoscroll {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  cursor: pointer;
}

.log-content {
  flex: 1;
  overflow-y: auto;
  font-family: var(--font-mono);
  font-size: var(--font-size-xs);
  padding: var(--spacing-sm);
}

.log-entry {
  display: flex;
  gap: var(--spacing-sm);
  padding: 3px 0;
  border-bottom: 1px solid var(--color-border);
  line-height: 1.4;
}

.log-time {
  color: var(--color-text-tertiary);
  min-width: 65px;
  flex-shrink: 0;
}

.log-level {
  min-width: 50px;
  font-weight: 600;
  flex-shrink: 0;
}

.log-target {
  color: var(--color-text-secondary);
  min-width: 80px;
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.log-message {
  flex: 1;
  word-break: break-word;
  color: var(--color-text-primary);
}

.log-debug .log-level {
  color: var(--color-text-tertiary);
}

.log-info .log-level {
  color: var(--color-primary);
}

.log-warn {
  background: rgba(243, 156, 18, 0.1);
}

.log-warn .log-level {
  color: var(--color-warning);
}

.log-error {
  background: var(--color-up-bg);
}

.log-error .log-level {
  color: var(--color-danger);
}

.log-empty {
  text-align: center;
  padding: var(--spacing-xl);
  color: var(--color-text-tertiary);
}
</style>
