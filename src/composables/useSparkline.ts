/**
 * SVG 迷你分时线（Sparkline）生成器
 * 根据价格数组生成 SVG path d 属性
 * A 股配色：涨红跌绿
 */
export function useSparkline() {
  /**
   * 生成 SVG sparkline 的 path d 属性
   * @param prices 价格数组（按时间正序）
   * @param width SVG 画布宽度
   * @param height SVG 画布高度
   * @returns SVG path 的 d 属性字符串
   */
  function generateSparkline(
    prices: number[],
    width: number,
    height: number
  ): string {
    if (!prices || prices.length < 2) return "";

    const min = Math.min(...prices);
    const max = Math.max(...prices);
    const range = max - min;

    // 避免除零：所有价格相同时画水平线
    const safeRange = range === 0 ? 1 : range;

    // 上下留 10% padding
    const padding = height * 0.1;
    const drawHeight = height - 2 * padding;

    const stepX = width / (prices.length - 1);

    const points = prices.map((price, i) => {
      const x = i * stepX;
      const y = padding + drawHeight * (1 - (price - min) / safeRange);
      return { x, y };
    });

    // 生成 path d — 使用直线段（迷你图不需要平滑曲线）
    const d = points
      .map((p, i) => {
        const prefix = i === 0 ? "M" : "L";
        return `${prefix}${p.x.toFixed(2)},${p.y.toFixed(2)}`;
      })
      .join(" ");

    return d;
  }

  /**
   * 根据首尾价格判断 sparkline 颜色
   * A 股惯例：涨红跌绿
   * @param prices 价格数组
   * @returns 颜色值
   */
  function getSparklineColor(prices: number[]): string {
    if (!prices || prices.length < 2) return "#999999";
    const first = prices[0];
    const last = prices[prices.length - 1];
    if (last > first) return "#ef5350"; // 涨 - 红
    if (last < first) return "#26a69a"; // 跌 - 绿
    return "#999999"; // 平 - 灰
  }

  return {
    generateSparkline,
    getSparklineColor,
  };
}
