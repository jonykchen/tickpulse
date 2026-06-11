import { defineStore } from "pinia";
import { ref } from "vue";
import { getSettings, updateSetting } from "@/lib/tauri";
import { DEFAULT_CONFIG, type AppConfig } from "@/types/config";

const STORAGE_KEY = "stock-monitor-config";

export const useConfigStore = defineStore("config", () => {
  // State
  const theme = ref<"dark" | "light">(DEFAULT_CONFIG.theme);
  const defaultGroupId = ref<number>(DEFAULT_CONFIG.defaultGroupId);
  const autoStart = ref<boolean>(DEFAULT_CONFIG.autoStart);
  const minimizeToTray = ref<boolean>(DEFAULT_CONFIG.minimizeToTray);
  const alertEnabled = ref<boolean>(DEFAULT_CONFIG.alertEnabled);
  const alertSound = ref<boolean>(DEFAULT_CONFIG.alertSound);

  // Persist to localStorage
  function persistToStorage(): void {
    const config: AppConfig = {
      theme: theme.value,
      defaultGroupId: defaultGroupId.value,
      autoStart: autoStart.value,
      minimizeToTray: minimizeToTray.value,
      alertEnabled: alertEnabled.value,
      alertSound: alertSound.value,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  }

  // Load from localStorage
  function loadFromStorage(): AppConfig | null {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored) as AppConfig;
      } catch {
        return null;
      }
    }
    return null;
  }

  // Actions
  async function loadSettings(): Promise<void> {
    // First try to load from localStorage for fast UI render
    const localConfig = loadFromStorage();
    if (localConfig) {
      theme.value = localConfig.theme;
      defaultGroupId.value = localConfig.defaultGroupId;
      autoStart.value = localConfig.autoStart;
      minimizeToTray.value = localConfig.minimizeToTray;
      alertEnabled.value = localConfig.alertEnabled;
      alertSound.value = localConfig.alertSound;
    }

    // Then sync with backend settings
    try {
      const settings = await getSettings();
      if (settings.theme) {
        theme.value = settings.theme as "dark" | "light";
      }
      if (settings.defaultGroupId) {
        defaultGroupId.value = parseInt(settings.defaultGroupId, 10);
      }
      if (settings.autoStart !== undefined) {
        autoStart.value = settings.autoStart === "true";
      }
      if (settings.minimizeToTray !== undefined) {
        minimizeToTray.value = settings.minimizeToTray === "true";
      }
      if (settings.alertEnabled !== undefined) {
        alertEnabled.value = settings.alertEnabled === "true";
      }
      if (settings.alertSound !== undefined) {
        alertSound.value = settings.alertSound === "true";
      }
      // Persist synced config to localStorage
      persistToStorage();
    } catch (error) {
      console.error("Failed to load settings from backend:", error);
    }
  }

  async function updateSettingByKey(
    key: keyof AppConfig,
    value: string | number | boolean
  ): Promise<void> {
    const strValue = String(value);

    // Update local state immediately for responsive UI
    switch (key) {
      case "theme":
        theme.value = value as "dark" | "light";
        break;
      case "defaultGroupId":
        defaultGroupId.value = value as number;
        break;
      case "autoStart":
        autoStart.value = value as boolean;
        break;
      case "minimizeToTray":
        minimizeToTray.value = value as boolean;
        break;
      case "alertEnabled":
        alertEnabled.value = value as boolean;
        break;
      case "alertSound":
        alertSound.value = value as boolean;
        break;
    }

    // Persist to localStorage
    persistToStorage();

    // Sync with backend
    try {
      await updateSetting(key, strValue);
    } catch (error) {
      console.error(`Failed to update setting ${key}:`, error);
    }
  }

  function toggleTheme(): void {
    const newTheme = theme.value === "dark" ? "light" : "dark";
    updateSettingByKey("theme", newTheme);
  }

  return {
    // State
    theme,
    defaultGroupId,
    autoStart,
    minimizeToTray,
    alertEnabled,
    alertSound,
    // Actions
    loadSettings,
    updateSetting: updateSettingByKey,
    toggleTheme,
  };
});
