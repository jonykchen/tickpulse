/** 预警规则类型（12 变体，v2.0 完整版） */
export enum AlertRuleType {
  /** 价格上穿阈值 */
  PriceAbove = "PriceAbove",
  /** 价格下穿阈值 */
  PriceBelow = "PriceBelow",
  /** 涨幅超过阈值(%) */
  ChangePercentAbove = "ChangePercentAbove",
  /** 跌幅超过阈值(%) */
  ChangePercentBelow = "ChangePercentBelow",
  /** 量比超过阈值 */
  VolumeRatioAbove = "VolumeRatioAbove",
  /** 换手率超过阈值(%) */
  TurnoverRateAbove = "TurnoverRateAbove",
  /** 涨停 */
  LimitUp = "LimitUp",
  /** 跌停 */
  LimitDown = "LimitDown",
  /** 异动拉升 */
  AnomalyRise = "AnomalyRise",
  /** 异动下跌 */
  AnomalyFall = "AnomalyFall",
  /** 临时停牌 */
  TempSuspend = "TempSuspend",
  /** 接近涨停 */
  NearLimitUp = "NearLimitUp",
}

/** 预警规则类型显示名称 */
export const ALERT_TYPE_LABELS: Record<AlertRuleType, string> = {
  [AlertRuleType.PriceAbove]: "价格上穿",
  [AlertRuleType.PriceBelow]: "价格下穿",
  [AlertRuleType.ChangePercentAbove]: "涨幅超限",
  [AlertRuleType.ChangePercentBelow]: "跌幅超限",
  [AlertRuleType.VolumeRatioAbove]: "量比超限",
  [AlertRuleType.TurnoverRateAbove]: "换手率超限",
  [AlertRuleType.LimitUp]: "涨停",
  [AlertRuleType.LimitDown]: "跌停",
  [AlertRuleType.AnomalyRise]: "异动拉升",
  [AlertRuleType.AnomalyFall]: "异动下跌",
  [AlertRuleType.TempSuspend]: "临时停牌",
  [AlertRuleType.NearLimitUp]: "接近涨停",
};

/** 预警规则类型是否需要阈值输入 */
export const ALERT_TYPE_NEEDS_THRESHOLD: Record<AlertRuleType, boolean> = {
  [AlertRuleType.PriceAbove]: true,
  [AlertRuleType.PriceBelow]: true,
  [AlertRuleType.ChangePercentAbove]: true,
  [AlertRuleType.ChangePercentBelow]: true,
  [AlertRuleType.VolumeRatioAbove]: true,
  [AlertRuleType.TurnoverRateAbove]: true,
  [AlertRuleType.LimitUp]: false,
  [AlertRuleType.LimitDown]: false,
  [AlertRuleType.AnomalyRise]: true,
  [AlertRuleType.AnomalyFall]: true,
  [AlertRuleType.TempSuspend]: false,
  [AlertRuleType.NearLimitUp]: false,
};

/** 预警规则 */
export interface AlertRule {
  id: string;
  secid: string;
  stockName: string;
  ruleType: AlertRuleType;
  /** 阈值 */
  threshold: number;
  enabled: boolean;
  triggered: boolean;
  createdAt: number;
}

/** 预警触发事件 */
export interface AlertTriggered {
  secid: string;
  stockName: string;
  ruleType: AlertRuleType;
  value: number;
  message: string;
}
