/// MACD 指标
use super::ema::ema;

/// MACD 计算结果
pub struct MacdResult {
    pub dif: Vec<Option<f64>>,
    pub dea: Vec<Option<f64>>,
    pub macd: Vec<Option<f64>>,
}

/// 计算 MACD
/// fast_period: 快线周期 (默认12)
/// slow_period: 慢线周期 (默认26)
/// signal_period: 信号线周期 (默认9)
pub fn macd(data: &[f64], fast_period: usize, slow_period: usize, signal_period: usize) -> MacdResult {
    let fast_ema = ema(data, fast_period);
    let slow_ema = ema(data, slow_period);

    // DIF = 快线EMA - 慢线EMA
    let dif: Vec<Option<f64>> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(f, s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect();

    // DEA = DIF的EMA
    let dif_values: Vec<f64> = dif.iter().map(|v| v.unwrap_or(0.0)).collect();
    let dea = ema(&dif_values, signal_period);

    // MACD = 2 * (DIF - DEA)
    let macd: Vec<Option<f64>> = dif
        .iter()
        .zip(dea.iter())
        .map(|(d, e)| match (d, e) {
            (Some(dv), Some(ev)) => Some(2.0 * (dv - ev)),
            _ => None,
        })
        .collect();

    MacdResult { dif, dea, macd }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macd() {
        let data: Vec<f64> = (1..=50).map(|i| i as f64 * 0.5).collect();
        let result = macd(&data, 12, 26, 9);
        assert_eq!(result.dif.len(), data.len());
        assert_eq!(result.dea.len(), data.len());
        assert_eq!(result.macd.len(), data.len());
    }
}
