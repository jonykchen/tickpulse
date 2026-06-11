/** 持仓汇总类型 */
export interface PortfolioSummary {
  /** 总市值 */
  totalMarketValue: string;
  /** 总浮动盈亏 */
  totalFloatPnl: string;
  /** 总盈亏率(%) */
  totalPnlRate: string;
  /** 当日盈亏 */
  totalTodayPnl: string;
  /** 当日盈亏率(%) */
  todayPnlRate: string;
  /** 基准指数涨跌幅(%) */
  benchmarkChange: string | null;
  /** 超额收益(%) */
  excessReturn: string | null;
}

/** 大宗交易类型 */
export interface BlockTrade {
  secid: string;
  code: string;
  name: string;
  tradeDate: string;
  price: number;
  volume: number;
  amount: number;
  buyer: string | null;
  seller: string | null;
  premiumRate: number | null;
}
