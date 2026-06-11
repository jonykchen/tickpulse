/** 异动类型 */
export enum AnomalyType {
  /** 快速拉升 */
  Surge = "surge",
  /** 快速下跌 */
  Dive = "dive",
  /** 量比突增 */
  VolumeSpike = "volume_spike",
  /** 封板 */
  SealBoard = "seal_board",
  /** 炸板 */
  BreakBoard = "break_board",
  /** 临时停牌 */
  TempSuspend = "temp_suspend",
}

/** 异动事件 */
export interface AnomalyEvent {
  id: string;
  secid: string;
  stockName: string;
  type: AnomalyType;
  /** 触发值，如涨幅、量比等 */
  value: number;
  /** 触发阈值 */
  threshold: number;
  /** 触发时间戳 */
  timestamp: number;
  /** 附加信息 */
  detail?: string;
}
