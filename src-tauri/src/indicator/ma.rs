/// 移动平均线计算

/// 简单移动平均 (SMA)
pub fn sma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let sum: f64 = data[i + 1 - period..=i].iter().sum();
            result.push(Some(sum / period as f64));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&data, 3);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!((result[2].unwrap() - 2.0).abs() < 0.001);
        assert!((result[3].unwrap() - 3.0).abs() < 0.001);
        assert!((result[4].unwrap() - 4.0).abs() < 0.001);
    }
}
