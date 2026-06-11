/**
 * 暗色/亮色主题切换
 * 基于 config store 的 theme 配置，自动在 document.documentElement 上切换 class
 */
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

/** 配置 store 接口 — 与 stores/config.ts 解耦，通过依赖注入或直接调用 */
interface ConfigStore {
  config: { value: { theme: "dark" | "light" } | null };
  init: () => Promise<void>;
}

/** 配置 store 单例引用，延迟绑定 */
let _configStore: ConfigStore | null = null;

/** 绑定 config store（在 main.ts 或 App.vue 中调用一次） */
export function bindConfigStore(store: ConfigStore): void {
  _configStore = store;
}

/** 主题 class 名称常量 */
const DARK_CLASS = "dark";
const LIGHT_CLASS = "light";

export function useTheme() {
  /** 是否为暗色主题 */
  const isDark = computed(() => {
    if (!_configStore?.config?.value) return true; // 默认暗色
    return _configStore.config.value.theme === "dark";
  });

  /** 应用主题 class 到 document.documentElement */
  function applyTheme(theme: "dark" | "light"): void {
    const root = document.documentElement;
    root.classList.remove(DARK_CLASS, LIGHT_CLASS);
    root.classList.add(theme);
  }

  /** 切换主题 */
  async function toggleTheme(): Promise<void> {
    if (!_configStore) {
      console.warn("[useTheme] config store 未绑定，请先调用 bindConfigStore()");
      return;
    }

    const currentTheme = _configStore.config.value?.theme ?? "dark";
    const newTheme: "dark" | "light" =
      currentTheme === "dark" ? "light" : "dark";

    applyTheme(newTheme);

    try {
      await invoke("update_setting", { key: "theme", value: newTheme });
      // 更新本地 store 状态
      if (_configStore.config.value) {
        _configStore.config.value.theme = newTheme;
      }
    } catch (e) {
      console.error("[useTheme] 主题持久化失败:", e);
      // 回滚
      applyTheme(currentTheme);
    }
  }

  /** 初始化主题（应用启动时调用） */
  async function initTheme(): Promise<void> {
    if (_configStore) {
      await _configStore.init();
    }
    const theme = _configStore?.config.value?.theme ?? "dark";
    applyTheme(theme);
  }

  return {
    isDark,
    toggleTheme,
    initTheme,
  };
}
