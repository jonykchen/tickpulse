/// 除权除息信息 + 复权因子计算
use serde::{Deserialize, Serialize};

/// 除权除息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExRightData {
    pub secid: String,
    pub ex_date: String,
    pub bonus_share: f64,
    pub allot_share: f64,
    pub allot_price: f64,
    pub dividend: f64,
}

impl ExRightData {
    /// 计算单次除权的复权因子
    /// 复权因子 = (送转股 + 配股 + 1) / (1 - 每股派息/昨收 + 配股*配股价/昨收)
    pub fn calc_factor(&self, pre_close: f64) -> f64 {
        if pre_close <= 0.0 {
            return 1.0;
        }
        let numerator = self.bonus_share + self.allot_share + 1.0;
        let denominator = 1.0 - self.dividend / pre_close + self.allot_share * self.allot_price / pre_close;
        if denominator.abs() < 1e-10 {
            return 1.0;
        }
        numerator / denominator
    }
}
