/**
 * 市场标签判断工具
 */
import { MarketTag } from "@/types/stock";

/** 从股票代码推断市场标签 */
export function getMarketTagFromCode(code: string): MarketTag {
  if (code.startsWith("688")) return MarketTag.StarMarket;
  if (code.startsWith("300") || code.startsWith("301")) return MarketTag.ChiNext;
  if (code.startsWith("8") || code.startsWith("4")) return MarketTag.BSE;
  if (code.startsWith("5") || code.startsWith("1")) return MarketTag.MainBoard;
  return MarketTag.MainBoard;
}

/** 从 secid 获取交易所市场标识 */
export function getMarketFromSecid(secid: string): number {
  const parts = secid.split(".");
  return parts.length >= 1 ? parseInt(parts[0]!, 10) : 0;
}

/** 从 secid 获取股票代码 */
export function getCodeFromSecid(secid: string): string {
  const parts = secid.split(".");
  return parts.length >= 2 ? parts[1]! : secid;
}
