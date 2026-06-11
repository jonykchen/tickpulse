import { defineStore } from "pinia";
import { ref } from "vue";
import { getSettings, updateSetting } from "@/lib/tauri";
import { DEFAULT_CONFIG, DEFAULT_SUSPEND_CONFIG, type AppConfig, type SuspendConfig } from "@/types/config";

const STORAGE_KEY = "stock-monitor-config";

export const useConfigStore = defineStore("config", () => {
  // State
  const theme = ref<"dark" | "light">(DEFAULT_CONFIG.theme);
  const defaultGroupId = ref<number>(DEFAULT_CONFIG.defaultGroupId);
  const autoStart = ref<boolean>(DEFAULT_CONFIG.autoStart);
  const minimizeToTray = ref<boolean>(DEFAULT_CONFIG.minimizeToTray);
  const alertEnabled = ref<boolean>(DEFAULT_CONFIG.alertEnabled);
  const alertSound = ref<boolean>(DEFAULT_CONFIG.alertSound);
  const klineAdjust = ref<"forward" | "backward" | "none">(DEFAULT_CONFIG.klineAdjust);
  const peDisplayType = ref<"ttm" | "dynamic" | "static">(DEFAULT_CONFIG.peDisplayType);
  const homeTableFields = ref<string[]>([...DEFAULT_CONFIG.homeTableFields]);
  const collectSecid = ref<string[]>([...DEFAULT_CONFIG.collectSecid]);
  const refreshRate = ref<number>(DEFAULT_CONFIG.refreshRate);
  const trendLineColor = ref<{ up: string; down: string }>({ ...DEFAULT_CONFIG.trendLineColor });
  const badgeType = ref<"changePercent" | "price" | "volume">(DEFAULT_CONFIG.badgeType);
  const badgeColor = ref<"auto" | "red" | "green" | "blue">(DEFAULT_CONFIG.badgeColor);
  const suspendConfig = ref<SuspendConfig>({ ...DEFAULT_SUSPEND_CONFIG });

  // Initialization lock to prevent concurrent loading
  let _inited = false;
  let _initPromise: Promise<void> | null = null;

  // Persist to localStorage
  function persistToStorage(): void {
    const config: AppConfig = {
      theme: theme.value,
      defaultGroupId: defaultGroupId.value,
      autoStart: autoStart.value,
      minimizeToTray: minimizeToTray.value,
      alertEnabled: alertEnabled.value,
      alertSound: alertSound.value,
      klineAdjust: klineAdjust.value,
      peDisplayType: peDisplayType.value,
      homeTableFields: homeTableFields.value,
      collectSecid: collectSecid.value,
      refreshRate: refreshRate.value,
      trendLineColor: trendLineColor.value,
      badgeType: badgeType.value,
      badgeColor: badgeColor.value,
      suspend: suspendConfig.value,
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

  // Merge suspend config with defaults
  function mergeSuspendConfig(partial?: Partial<SuspendConfig>): SuspendConfig {
    if (!partial) return { ...DEFAULT_SUSPEND_CONFIG };
    return {
      ...DEFAULT_SUSPEND_CONFIG,
      ...partial,
      group: {
        ...DEFAULT_SUSPEND_CONFIG.group,
        ...(partial.group || {}),
      },
    };
  }

  // Actions
  async function loadSettings(): Promise<void> {
    // Use initialization lock to prevent concurrent loading
    if (_initPromise) {
      return _initPromise;
    }

    _initPromise = (async () => {
      // First try to load from localStorage for fast UI render
      const localConfig = loadFromStorage();
      if (localConfig) {
        theme.value = localConfig.theme ?? DEFAULT_CONFIG.theme;
        defaultGroupId.value = localConfig.defaultGroupId ?? DEFAULT_CONFIG.defaultGroupId;
        autoStart.value = localConfig.autoStart ?? DEFAULT_CONFIG.autoStart;
        minimizeToTray.value = localConfig.minimizeToTray ?? DEFAULT_CONFIG.minimizeToTray;
        alertEnabled.value = localConfig.alertEnabled ?? DEFAULT_CONFIG.alertEnabled;
        alertSound.value = localConfig.alertSound ?? DEFAULT_CONFIG.alertSound;
        klineAdjust.value = localConfig.klineAdjust ?? DEFAULT_CONFIG.klineAdjust;
        peDisplayType.value = localConfig.peDisplayType ?? DEFAULT_CONFIG.peDisplayType;
        homeTableFields.value = localConfig.homeTableFields ?? [...DEFAULT_CONFIG.homeTableFields];
        collectSecid.value = localConfig.collectSecid ?? [...DEFAULT_CONFIG.collectSecid];
        refreshRate.value = localConfig.refreshRate ?? DEFAULT_CONFIG.refreshRate;
        trendLineColor.value = localConfig.trendLineColor ?? { ...DEFAULT_CONFIG.trendLineColor };
        badgeType.value = localConfig.badgeType ?? DEFAULT_CONFIG.badgeType;
        badgeColor.value = localConfig.badgeColor ?? DEFAULT_CONFIG.badgeColor;
        suspendConfig.value = mergeSuspendConfig(localConfig.suspend);
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
        if (settings.klineAdjust) {
          klineAdjust.value = settings.klineAdjust as "forward" | "backward" | "none";
        }
        if (settings.peDisplayType) {
          peDisplayType.value = settings.peDisplayType as "ttm" | "dynamic" | "static";
        }
        if (settings.refreshRate) {
          refreshRate.value = parseInt(settings.refreshRate, 10);
        }
        if (settings.badgeType) {
          badgeType.value = settings.badgeType as "changePercent" | "price" | "volume";
        }
        if (settings.badgeColor) {
          badgeColor.value = settings.badgeColor as "auto" | "red" | "green" | "blue";
        }
        // Persist synced config to localStorage
        persistToStorage();
      } catch (error) {
        console.error("Failed to load settings from backend:", error);
      }

      _inited = true;
    })();

    return _initPromise;
  }

  async function updateSettingByKey(
    key: keyof AppConfig,
    value: string | number | boolean | object
  ): Promise<void> {
    const strValue = typeof value === "object" ? JSON.stringify(value) : String(value);

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
      case "klineAdjust":
        klineAdjust.value = value as "forward" | "backward" | "none";
        break;
      case "peDisplayType":
        peDisplayType.value = value as "ttm" | "dynamic" | "static";
        break;
      case "homeTableFields":
        homeTableFields.value = value as string[];
        break;
      case "collectSecid":
        collectSecid.value = value as string[];
        break;
      case "refreshRate":
        refreshRate.value = value as number;
        break;
      case "trendLineColor":
        trendLineColor.value = value as { up: string; down: string };
        break;
      case "badgeType":
        badgeType.value = value as "changePercent" | "price" | "volume";
        break;
      case "badgeColor":
        badgeColor.value = value as "auto" | "red" | "green" | "blue";
        break;
      case "suspend":
        suspendConfig.value = mergeSuspendConfig(value as Partial<SuspendConfig>);
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
    klineAdjust,
    peDisplayType,
    homeTableFields,
    collectSecid,
    refreshRate,
    trendLineColor,
    badgeType,
    badgeColor,
    suspendConfig,
    // Actions
    loadSettings,
    updateSetting: updateSettingByKey,
    toggleTheme,
    mergeSuspendConfig,
  };
});
