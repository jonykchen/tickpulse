/** 预警规则类型 */
export enum AlertRuleType {
  /** 价格上限 */
  PriceAbove = "price_above",
  /** 价格下限 */
  PriceBelow = "price_below",
  /** 涨幅超限 */
  ChangePercentAbove = "change_percent_above",
  /** 跌幅超限 */
  ChangePercentBelow = "change_percent_below",
  /** 量比超限 */
  VolumeRatioAbove = "volume_ratio_above",
  /** 新高 */
  NewHigh = "new_high",
  /** 新低 */
  NewLow = "new_low",
  /** 涨停 */
  LimitUp = "limit_up",
  /** 跌停 */
  LimitDown = "limit_down",
  /** 异动 */
  Anomaly = "anomaly",
  /** 临时停牌 */
  TempSuspend = "temp_suspend",
}

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
