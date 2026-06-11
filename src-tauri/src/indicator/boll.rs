/// 布林带
use super::ma::sma;

/// 布林带结果
pub struct BollResult {
    pub mid: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

/// 计算布林带
/// period: 周期 (默认20)
/// multiplier: 乘数 (默认2.0)
pub fn boll(data: &[f64], period: usize, multiplier: f64) -> BollResult {
    let mid = sma(data, period);

    let mut upper = Vec::with_capacity(data.len());
    let mut lower = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        if i < period - 1 {
            upper.push(None);
            lower.push(None);
        } else {
            let slice = &data[i + 1 - period..=i];
            let mean = mid[i].unwrap();
            let variance: f64 = slice.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period as f64;
            let std_dev = variance.sqrt();
            upper.push(Some(mean + multiplier * std_dev));
            lower.push(Some(mean - multiplier * std_dev));
        }
    }

    BollResult { mid, upper, lower }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boll() {
        let data: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let result = boll(&data, 20, 2.0);
        assert_eq!(result.mid.len(), data.len());
        assert!(result.upper[25].is_some());
        assert!(result.lower[25].is_some());
    }
}
