/**
 * 预警规则管理
 * 通过 alert store 和 Tauri IPC 实现预警规则的增删启停
 */
import { invoke } from "@tauri-apps/api/core";
import { AlertRuleType } from "@/types/alert";
import type { AlertRule } from "@/types/alert";

/** Alert store 接口 — 与 stores/alert.ts 解耦 */
interface AlertStore {
  rules: { value: AlertRule[] };
  fetchRules: () => Promise<void>;
}

/** Alert store 单例引用，延迟绑定 */
let _alertStore: AlertStore | null = null;

/** 绑定 alert store（在 main.ts 或 App.vue 中调用一次） */
export function bindAlertStore(store: AlertStore): void {
  _alertStore = store;
}

export function useAlert() {
  /**
   * 添加预警规则
   * @param secid 证券ID，如 "0.000001"
   * @param stockName 股票名称
   * @param type 预警规则类型
   * @param threshold 阈值
   */
  async function addAlertRule(
    secid: string,
    stockName: string,
    type: AlertRuleType,
    threshold: number
  ): Promise<void> {
    try {
      await invoke("add_alert_rule", {
        rule: {
          secid,
          stockName,
          ruleType: type,
          threshold,
          enabled: true,
        },
      });

      // 同步刷新 store
      if (_alertStore) {
        await _alertStore.fetchRules();
      }
    } catch (e) {
      console.error("[useAlert] 添加预警规则失败:", e);
      throw e;
    }
  }

  /**
   * 删除预警规则
   * @param id 规则ID
   */
  async function removeAlertRule(id: string): Promise<void> {
    try {
      await invoke("remove_alert_rule", { ruleId: id });

      // 同步刷新 store
      if (_alertStore) {
        await _alertStore.fetchRules();
      }
    } catch (e) {
      console.error("[useAlert] 删除预警规则失败:", e);
      throw e;
    }
  }

  /**
   * 切换预警规则启用/禁用
   * @param id 规则ID
   */
  async function toggleAlertRule(id: string): Promise<void> {
    // 从 store 中查找当前规则状态
    const rule = _alertStore?.rules.value.find((r) => r.id === id);
    if (!rule) {
      console.warn("[useAlert] 未找到规则:", id);
      return;
    }

    try {
      await invoke("toggle_alert_rule", {
        ruleId: id,
        enabled: !rule.enabled,
      });

      // 同步刷新 store
      if (_alertStore) {
        await _alertStore.fetchRules();
      }
    } catch (e) {
      console.error("[useAlert] 切换预警规则失败:", e);
      throw e;
    }
  }

  return {
    addAlertRule,
    removeAlertRule,
    toggleAlertRule,
  };
}
