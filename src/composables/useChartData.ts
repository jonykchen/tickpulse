/**
 * 图表数据格式化
 * 将后端数据转换为 LightweightCharts / ECharts 所需格式
 */
import type { KlineBar, TimelinePoint } from "@/types/chart";

/**
 * LightweightCharts K线数据项
 * time 支持多种格式，此处使用 Unix 时间戳（秒）
 */
export interface KlineDataItem {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
}

/**
 * LightweightCharts 成交量数据项
 */
export interface VolumeDataItem {
  time: number;
  value: number;
  color: string;
}

/**
 * LightweightCharts 分时线数据项
 * time 为 HH:mm 格式字符串时需转换为时间戳
 */
export interface TimelineDataItem {
  time: number;
  value: number;
}

/**
 * 分时均价线数据项
 */
export interface AvgPriceDataItem {
  time: number;
  value: number;
}

export function useChartData() {
  /**
   * 将后端 KlineBar 数组格式化为 LightweightCharts K线数据
   * @param bars K线原始数据
   * @returns 格式化后的 K线数据 + 成交量数据
   */
  function formatKlineData(bars: KlineBar[]): {
    candlestick: KlineDataItem[];
    volume: VolumeDataItem[];
  } {
    if (!bars || bars.length === 0) {
      return { candlestick: [], volume: [] };
    }

    const candlestick: KlineDataItem[] = [];
    const volume: VolumeDataItem[] = [];

    for (const bar of bars) {
      // LightweightCharts 时间戳为秒级
      const time = bar.time;

      candlestick.push({
        time,
        open: bar.open,
        high: bar.high,
        low: bar.low,
        close: bar.close,
      });

      // 成交量颜色：收盘 >= 开盘 为涨（红），否则为跌（绿）
      const isUp = bar.close >= bar.open;
      volume.push({
        time,
        value: bar.volume,
        color: isUp
          ? "rgba(239,83,80,0.5)" // 涨 - 红色半透明
          : "rgba(38,166,154,0.5)", // 跌 - 绿色半透明
      });
    }

    return { candlestick, volume };
  }

  /**
   * 将后端分时走势数据格式化为 LightweightCharts 折线图数据
   * 含均价线计算
   * @param data 分时走势原始数据（含 preClose 和 points）
   * @returns 格式化后的价格线数据 + 均价线数据
   */
  function formatTimelineData(data: {
    preClose: number;
    points: TimelinePoint[];
  }): {
    priceLine: TimelineDataItem[];
    avgLine: AvgPriceDataItem[];
  } {
    if (!data?.points || data.points.length === 0) {
      return { priceLine: [], avgLine: [] };
    }

    const priceLine: TimelineDataItem[] = [];
    const avgLine: AvgPriceDataItem[] = [];

    let cumAmount = 0; // 累计成交金额
    let cumVolume = 0; // 累计成交量

    for (const point of data.points) {
      // 将 "HH:mm" 转换为当日 Unix 时间戳（秒）
      // LightweightCharts 支持业务日格式 "YYYY-MM-DD"，分时图使用时间戳
      const time = parseTimeToSeconds(point.time);

      priceLine.push({
        time,
        value: point.price,
      });

      // 均价线 = 累计成交金额 / 累计成交量
      cumAmount += point.price * point.volume;
      cumVolume += point.volume;
      avgLine.push({
        time,
        value: cumVolume > 0 ? cumAmount / cumVolume : point.price,
      });
    }

    return { priceLine, avgLine };
  }

  return {
    formatKlineData,
    formatTimelineData,
  };
}

/**
 * 将 "HH:mm" 格式时间转换为当日 Unix 时间戳（秒）
 * 用于 LightweightCharts 分时图时间轴
 */
function parseTimeToSeconds(timeStr: string): number {
  const today = new Date();
  const [hours, minutes] = timeStr.split(":").map(Number);
  const date = new Date(
    today.getFullYear(),
    today.getMonth(),
    today.getDate(),
    hours,
    minutes,
    0,
    0
  );
  return Math.floor(date.getTime() / 1000);
}
