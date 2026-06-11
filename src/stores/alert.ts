import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { AlertRule } from "@/types/alert";
import { AlertRuleType } from "@/types/alert";
import { getAlertRules, addAlertRule, removeAlertRule, toggleAlertRule } from "@/lib/tauri";

const STORAGE_KEY = "stock-monitor-alert-rules";

export const useAlertStore = defineStore("alert", () => {
  // State
  const rules = ref<AlertRule[]>([]);

  // Getters
  const enabledRules = computed(() => {
    return rules.value.filter((r) => r.enabled);
  });

  const rulesByStock = computed(() => {
    return (secid: string) => {
      return rules.value.filter((r) => r.secid === secid);
    };
  });

  const ruleCount = computed(() => {
    return rules.value.length;
  });

  const enabledRuleCount = computed(() => {
    return enabledRules.value.length;
  });

  // Persist to localStorage
  function persistToStorage(): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(rules.value));
  }

  // Load from localStorage
  function loadFromStorage(): AlertRule[] | null {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored) as AlertRule[];
      } catch {
        return null;
      }
    }
    return null;
  }

  // Actions
  async function loadRules(): Promise<void> {
    // Load from localStorage for fast UI render
    const storedRules = loadFromStorage();
    if (storedRules) {
      rules.value = storedRules;
    }

    // Sync with backend
    try {
      const backendRules = await getAlertRules();
      if (backendRules && backendRules.length > 0) {
        rules.value = backendRules;
        persistToStorage();
      }
    } catch (error) {
      console.error("Failed to sync alert rules from backend:", error);
    }
  }

  async function addRule(rule: Omit<AlertRule, "id" | "createdAt">): Promise<AlertRule> {
    const newRule: AlertRule = {
      ...rule,
      id: crypto.randomUUID(),
      createdAt: Date.now(),
    };

    // Optimistic local update
    rules.value.push(newRule);
    persistToStorage();

    // Sync with backend
    try {
      await addAlertRule(rule.secid, rule.stockName, rule.ruleType, rule.threshold);
    } catch (error) {
      console.error("Failed to add alert rule to backend:", error);
    }

    return newRule;
  }

  async function removeRule(id: string): Promise<void> {
    const index = rules.value.findIndex((r) => r.id === id);
    if (index !== -1) {
      rules.value.splice(index, 1);
      persistToStorage();
    }

    try {
      await removeAlertRule(id);
    } catch (error) {
      console.error("Failed to remove alert rule from backend:", error);
    }
  }

  async function toggleRule(id: string): Promise<void> {
    const rule = rules.value.find((r) => r.id === id);
    if (rule) {
      rule.enabled = !rule.enabled;
      persistToStorage();

      try {
        await toggleAlertRule(id, rule.enabled);
      } catch (error) {
        console.error("Failed to toggle alert rule on backend:", error);
      }
    }
  }

  function updateRule(
    id: string,
    updates: Partial<Omit<AlertRule, "id" | "createdAt">>
  ): void {
    const rule = rules.value.find((r) => r.id === id);
    if (rule) {
      Object.assign(rule, updates);
      persistToStorage();
    }
  }

  function markTriggered(id: string): void {
    const rule = rules.value.find((r) => r.id === id);
    if (rule) {
      rule.triggered = true;
      persistToStorage();
    }
  }

  function clearTriggered(): void {
    for (const rule of rules.value) {
      rule.triggered = false;
    }
    persistToStorage();
  }

  return {
    // State
    rules,
    // Getters
    enabledRules,
    rulesByStock,
    ruleCount,
    enabledRuleCount,
    // Actions
    loadRules,
    addRule,
    removeRule,
    toggleRule,
    updateRule,
    markTriggered,
    clearTriggered,
  };
});
