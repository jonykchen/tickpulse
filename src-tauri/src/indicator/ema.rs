/// EMA 指数移动平均
pub fn ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    if data.is_empty() || period == 0 {
        return vec![None; data.len()];
    }

    let k = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());
    let mut prev_ema: Option<f64> = None;

    for &val in data {
        match prev_ema {
            None => {
                prev_ema = Some(val);
                result.push(Some(val));
            }
            Some(prev) => {
                let current = val * k + prev * (1.0 - k);
                prev_ema = Some(current);
                result.push(Some(current));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema() {
        let data = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let result = ema(&data, 5);
        assert!(result[0].is_some());
        assert!(result[4].is_some());
    }
}
