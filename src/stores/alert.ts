import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { AlertRule } from "@/types/alert";

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

    // TODO: Sync with backend if needed
    // Currently alert rules are managed locally
  }

  function addRule(rule: Omit<AlertRule, "id" | "createdAt">): AlertRule {
    const newRule: AlertRule = {
      ...rule,
      id: crypto.randomUUID(),
      createdAt: Date.now(),
    };
    rules.value.push(newRule);
    persistToStorage();
    return newRule;
  }

  function removeRule(id: string): void {
    const index = rules.value.findIndex((r) => r.id === id);
    if (index !== -1) {
      rules.value.splice(index, 1);
      persistToStorage();
    }
  }

  function toggleRule(id: string): void {
    const rule = rules.value.find((r) => r.id === id);
    if (rule) {
      rule.enabled = !rule.enabled;
      persistToStorage();
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
