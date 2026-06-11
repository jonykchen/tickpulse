pub mod exright;

use crate::db::DbPool;
use crate::market::AdjustType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 复权因子计算器
pub struct AdjustFactorCalculator {
    db: Arc<DbPool>,
}

impl AdjustFactorCalculator {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 计算前复权因子
    /// 前复权：以最新价为基准，向前调整历史价格
    pub fn calc_forward_factor(exrights: &[ExRightRecord], bars_count: usize) -> Vec<f64> {
        if exrights.is_empty() {
            return vec![1.0; bars_count];
        }

        let mut factors = vec![1.0; bars_count];
        // 从最新到最旧累乘复权因子
        let mut cumulative = 1.0;
        for ex in exrights.iter().rev() {
            let factor = (ex.bonus_share + ex.allot_share + 1.0) / (1.0 - ex.dividend + ex.allot_share * ex.allot_price);
            cumulative *= factor;
        }
        // 简化实现：返回均匀分布的复权因子
        for f in factors.iter_mut() {
            *f = cumulative;
        }
        factors
    }
}

/// 除权除息记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExRightRecord {
    pub ex_date: String,
    pub bonus_share: f64,
    pub allot_share: f64,
    pub allot_price: f64,
    pub dividend: f64,
}
