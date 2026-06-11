//! 决策记忆闭环模块
//! 存储历史决策、T+1 检验、反思学习

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::db::DbPool;

/// 决策记忆管理器
pub struct DecisionMemory {
    db: Arc<DbPool>,
}

/// 历史决策记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastDecision {
    /// 唯一 ID
    pub id: String,
    /// 股票标识 (如 "600519.SH")
    pub secid: String,
    /// 股票名称
    pub stock_name: String,
    /// 决策日期 (Unix 时间戳)
    pub decision_date: i64,
    /// 评级: StrongBuy/Buy/Hold/Sell/StrongSell
    pub rating: String,
    /// 目标价
    pub target_price: Option<f64>,
    /// 止损价
    pub stop_loss: Option<f64>,
    /// 评级理由摘要
    pub reasoning_summary: String,
    /// PEG 值
    pub peg_value: Option<f64>,
    /// T+1 实际涨跌幅
    pub actual_return: Option<f64>,
    /// 相对沪深 300 超额收益
    pub alpha_return: Option<f64>,
    /// LLM 反思结论
    pub reflection: Option<String>,
    /// 创建时间
    pub created_at: i64,
}

/// 决策状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionStatus {
    /// 待检验
    Pending,
    /// 已反思
    Reflected,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Reflected => "reflected",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "reflected" => Self::Reflected,
            _ => Self::Pending,
        }
    }
}

impl DecisionMemory {
    /// 创建决策记忆管理器
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 存储分析决策
    /// 分析完成时调用，记录评级和理由
    pub fn store_decision(&self, decision: &PastDecision) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO decision_memory_v2
             (id, secid, stock_name, decision_date, rating, target_price, stop_loss,
              reasoning_summary, peg_value, actual_return, alpha_return, reflection,
              status, created_at, reflected_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, NULL)",
            rusqlite::params![
                decision.id,
                decision.secid,
                decision.stock_name,
                decision.decision_date,
                decision.rating,
                decision.target_price,
                decision.stop_loss,
                decision.reasoning_summary,
                decision.peg_value,
                decision.actual_return,
                decision.alpha_return,
                decision.reflection,
                DecisionStatus::Pending.as_str(),
                decision.created_at,
            ],
        ).map_err(|e| format!("存储决策失败: {}", e))?;

        Ok(())
    }

    /// T+1 检验昨天的评级是否正确
    /// 盘后复盘时调用，更新实际收益并生成反思
    pub fn reflect_on_decision(
        &self,
        secid: &str,
        decision_date: i64,
        actual_return: f64,
        benchmark_return: f64,
    ) -> Result<Option<String>, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        // 计算超额收益
        let alpha_return = actual_return - benchmark_return;

        // 更新实际收益
        conn.execute(
            "UPDATE decision_memory_v2
             SET actual_return = ?1, alpha_return = ?2
             WHERE secid = ?3 AND decision_date = ?4",
            rusqlite::params![actual_return, alpha_return, secid, decision_date],
        )
        .map_err(|e| format!("更新决策收益失败: {}", e))?;

        // 获取原始决策
        let decision = self.get_decision(secid, decision_date)?;

        if let Some(d) = decision {
            // 生成反思文本
            let reflection = self.generate_reflection(&d, actual_return, alpha_return);

            // 保存反思
            let now = chrono::Utc::now().timestamp();
            conn.execute(
                "UPDATE decision_memory_v2
                 SET reflection = ?1, status = ?2, reflected_at = ?3
                 WHERE secid = ?4 AND decision_date = ?5",
                rusqlite::params![
                    &reflection,
                    DecisionStatus::Reflected.as_str(),
                    now,
                    secid,
                    decision_date
                ],
            )
            .map_err(|e| format!("保存反思失败: {}", e))?;

            return Ok(Some(reflection));
        }

        Ok(None)
    }

    /// 获取历史上下文（最近 N 条同股决策）
    /// 用于分析时注入记忆
    pub fn get_past_context(&self, secid: &str, limit: u32) -> Result<Vec<PastDecision>, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, secid, stock_name, decision_date, rating, target_price,
                        stop_loss, reasoning_summary, peg_value, actual_return,
                        alpha_return, reflection, created_at
                 FROM decision_memory_v2
                 WHERE secid = ?1
                 ORDER BY decision_date DESC
                 LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;

        let decisions = stmt
            .query_map(rusqlite::params![secid, limit], |row| {
                Ok(PastDecision {
                    id: row.get(0)?,
                    secid: row.get(1)?,
                    stock_name: row.get(2)?,
                    decision_date: row.get(3)?,
                    rating: row.get(4)?,
                    target_price: row.get(5)?,
                    stop_loss: row.get(6)?,
                    reasoning_summary: row.get(7)?,
                    peg_value: row.get(8)?,
                    actual_return: row.get(9)?,
                    alpha_return: row.get(10)?,
                    reflection: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })
            .map_err(|e| format!("查询历史决策失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(decisions)
    }

    /// 批量获取待检验的决策
    /// 用于盘后复盘流程
    pub fn get_pending_reflections(&self) -> Result<Vec<PastDecision>, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, secid, stock_name, decision_date, rating, target_price,
                        stop_loss, reasoning_summary, peg_value, actual_return,
                        alpha_return, reflection, created_at
                 FROM decision_memory_v2
                 WHERE status = ?1
                 ORDER BY decision_date ASC",
            )
            .map_err(|e| e.to_string())?;

        let decisions = stmt
            .query_map(rusqlite::params![DecisionStatus::Pending.as_str()], |row| {
                Ok(PastDecision {
                    id: row.get(0)?,
                    secid: row.get(1)?,
                    stock_name: row.get(2)?,
                    decision_date: row.get(3)?,
                    rating: row.get(4)?,
                    target_price: row.get(5)?,
                    stop_loss: row.get(6)?,
                    reasoning_summary: row.get(7)?,
                    peg_value: row.get(8)?,
                    actual_return: row.get(9)?,
                    alpha_return: row.get(10)?,
                    reflection: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })
            .map_err(|e| format!("查询待检验决策失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(decisions)
    }

    /// 获取单个决策
    fn get_decision(&self, secid: &str, decision_date: i64) -> Result<Option<PastDecision>, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let result = conn.query_row(
            "SELECT id, secid, stock_name, decision_date, rating, target_price,
                    stop_loss, reasoning_summary, peg_value, actual_return,
                    alpha_return, reflection, created_at
             FROM decision_memory_v2
             WHERE secid = ?1 AND decision_date = ?2",
            rusqlite::params![secid, decision_date],
            |row| {
                Ok(PastDecision {
                    id: row.get(0)?,
                    secid: row.get(1)?,
                    stock_name: row.get(2)?,
                    decision_date: row.get(3)?,
                    rating: row.get(4)?,
                    target_price: row.get(5)?,
                    stop_loss: row.get(6)?,
                    reasoning_summary: row.get(7)?,
                    peg_value: row.get(8)?,
                    actual_return: row.get(9)?,
                    alpha_return: row.get(10)?,
                    reflection: row.get(11)?,
                    created_at: row.get(12)?,
                })
            },
        );

        match result {
            Ok(d) => Ok(Some(d)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("查询决策失败: {}", e)),
        }
    }

    /// 生成反思文本
    /// 基于原始决策和实际收益生成结构化反思
    fn generate_reflection(
        &self,
        decision: &PastDecision,
        actual_return: f64,
        alpha_return: f64,
    ) -> String {
        let direction_correct = match decision.rating.as_str() {
            "StrongBuy" | "Buy" => actual_return > 0.0,
            "Hold" => actual_return.abs() < 3.0, // 中性判断，涨跌 3% 内视为正确
            "Sell" | "StrongSell" => actual_return < 0.0,
            _ => false,
        };

        let direction_text = if direction_correct { "正确" } else { "错误" };

        let lesson = self.extract_lesson(decision, actual_return, alpha_return);

        format!(
            "【决策回顾】\n\
             - 评级: {}\n\
             - 实际收益: {:+.2}%\n\
             - 超额收益(vs沪深300): {:+.2}%\n\
             - 方向判断: {}\n\n\
             【经验教训】\n\
             {}",
            decision.rating,
            actual_return,
            alpha_return,
            direction_text,
            lesson
        )
    }

    /// 提取经验教训
    fn extract_lesson(&self, decision: &PastDecision, actual_return: f64, alpha_return: f64) -> String {
        let mut lessons = Vec::new();

        // 根据评级和实际收益分析
        match decision.rating.as_str() {
            "StrongBuy" | "Buy" => {
                if actual_return < -5.0 {
                    lessons.push("买入评级但大幅下跌，可能忽略了系统性风险或基本面恶化信号".to_string());
                } else if actual_return < 0.0 {
                    lessons.push("买入评级但小幅下跌，需检查入场时机是否合适".to_string());
                } else if actual_return > 5.0 {
                    lessons.push("买入评级验证成功，方向判断正确".to_string());
                }
            }
            "Hold" => {
                if actual_return.abs() > 5.0 {
                    lessons.push("中性评级但波动较大，可能低估了趋势动能".to_string());
                }
            }
            "Sell" | "StrongSell" => {
                if actual_return > 5.0 {
                    lessons.push("卖出评级但大幅上涨，可能过度悲观或忽视了利好因素".to_string());
                } else if actual_return > 0.0 {
                    lessons.push("卖出评级但上涨，需检查做空逻辑是否成立".to_string());
                } else if actual_return < -5.0 {
                    lessons.push("卖出评级验证成功，规避了重大损失".to_string());
                }
            }
            _ => {}
        }

        // 超额收益分析
        if alpha_return < -3.0 {
            lessons.push(format!("跑输基准 {:.1}%，需关注选股与择时能力", alpha_return.abs()));
        } else if alpha_return > 3.0 {
            lessons.push(format!("跑赢基准 {:.1}%，选股逻辑值得坚持", alpha_return));
        }

        if lessons.is_empty() {
            "无明显教训，决策与结果基本一致".to_string()
        } else {
            lessons.join("；")
        }
    }

    /// 格式化历史上下文为 Prompt 注入文本
    pub fn format_context_for_prompt(decisions: &[PastDecision]) -> String {
        if decisions.is_empty() {
            return "【历史决策】无历史决策记录".to_string();
        }

        let mut text = String::from("【历史决策参考】\n");

        for (i, d) in decisions.iter().enumerate() {
            let return_str = d.actual_return
                .map(|r| format!("{:+.2}%", r))
                .unwrap_or_else(|| "待检验".to_string());

            let alpha_str = d.alpha_return
                .map(|r| format!("{:+.2}%", r))
                .unwrap_or_else(|| "-".to_string());

            let date_str = chrono::DateTime::from_timestamp(d.decision_date, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| d.decision_date.to_string());

            text.push_str(&format!(
                "{}. {} | {} | 收益: {} | 超额: {}\n",
                i + 1,
                date_str,
                d.rating,
                return_str,
                alpha_str
            ));

            if let Some(ref reflection) = d.reflection {
                // 只取反思的第一行（教训摘要）
                if let Some(first_line) = reflection.lines().find(|l| l.starts_with("【经验教训】")) {
                    text.push_str(&format!("   {}\n", first_line));
                }
            }
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbPool;
    use std::sync::Mutex;
    use rusqlite::Connection;

    fn test_pool() -> DbPool {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS decision_memory_v2 (
                id TEXT PRIMARY KEY,
                secid TEXT NOT NULL,
                stock_name TEXT NOT NULL,
                decision_date INTEGER NOT NULL,
                rating TEXT NOT NULL,
                target_price REAL,
                stop_loss REAL,
                reasoning_summary TEXT NOT NULL,
                peg_value REAL,
                actual_return REAL,
                alpha_return REAL,
                reflection TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at INTEGER NOT NULL,
                reflected_at INTEGER,
                UNIQUE(secid, decision_date)
            );
            CREATE INDEX IF NOT EXISTS idx_decision_memory_v2_secid ON decision_memory_v2(secid);
            CREATE INDEX IF NOT EXISTS idx_decision_memory_v2_date ON decision_memory_v2(decision_date);",
        ).unwrap();
        DbPool {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn test_store_decision() {
        let pool = test_pool();
        let memory = DecisionMemory::new(Arc::new(pool));

        let decision = PastDecision {
            id: "test-1".to_string(),
            secid: "600519.SH".to_string(),
            stock_name: "贵州茅台".to_string(),
            decision_date: 1704067200, // 2024-01-01
            rating: "Buy".to_string(),
            target_price: Some(2000.0),
            stop_loss: Some(1800.0),
            reasoning_summary: "估值合理，成长性好".to_string(),
            peg_value: Some(1.2),
            actual_return: None,
            alpha_return: None,
            reflection: None,
            created_at: 1704067200,
        };

        let result = memory.store_decision(&decision);
        assert!(result.is_ok());

        let stored = memory.get_decision("600519.SH", 1704067200).unwrap();
        assert!(stored.is_some());
        let stored = stored.unwrap();
        assert_eq!(stored.rating, "Buy");
        assert_eq!(stored.target_price, Some(2000.0));
    }

    #[test]
    fn test_reflect_on_decision() {
        let pool = test_pool();
        let memory = DecisionMemory::new(Arc::new(pool));

        let decision = PastDecision {
            id: "test-2".to_string(),
            secid: "000001.SZ".to_string(),
            stock_name: "平安银行".to_string(),
            decision_date: 1704153600,
            rating: "Hold".to_string(),
            target_price: None,
            stop_loss: None,
            reasoning_summary: "观望为主".to_string(),
            peg_value: None,
            actual_return: None,
            alpha_return: None,
            reflection: None,
            created_at: 1704153600,
        };

        memory.store_decision(&decision).unwrap();

        // 模拟 T+1 检验
        let result = memory.reflect_on_decision("000001.SZ", 1704153600, 2.5, 1.0);
        assert!(result.is_ok());

        let reflection = result.unwrap();
        assert!(reflection.is_some());

        // 验证更新后的状态
        let updated = memory.get_decision("000001.SZ", 1704153600).unwrap().unwrap();
        assert_eq!(updated.actual_return, Some(2.5));
        assert_eq!(updated.alpha_return, Some(1.5)); // 2.5 - 1.0
        assert!(updated.reflection.is_some());
    }

    #[test]
    fn test_get_past_context() {
        let pool = test_pool();
        let memory = DecisionMemory::new(Arc::new(pool));

        // 存储多条决策
        for i in 0..5 {
            let decision = PastDecision {
                id: format!("test-{}", i),
                secid: "600519.SH".to_string(),
                stock_name: "贵州茅台".to_string(),
                decision_date: 1704067200 + i * 86400,
                rating: "Buy".to_string(),
                target_price: None,
                stop_loss: None,
                reasoning_summary: "测试".to_string(),
                peg_value: None,
                actual_return: None,
                alpha_return: None,
                reflection: None,
                created_at: 1704067200 + i * 86400,
            };
            memory.store_decision(&decision).unwrap();
        }

        // 获取最近 3 条
        let context = memory.get_past_context("600519.SH", 3).unwrap();
        assert_eq!(context.len(), 3);

        // 验证按日期降序
        assert!(context[0].decision_date > context[1].decision_date);
    }

    #[test]
    fn test_get_pending_reflections() {
        let pool = test_pool();
        let memory = DecisionMemory::new(Arc::new(pool));

        // 存储 2 条待检验决策
        for i in 0..2 {
            let decision = PastDecision {
                id: format!("pending-{}", i),
                secid: format!("{:06}.SH", 600000 + i),
                stock_name: format!("股票{}", i),
                decision_date: 1704067200 + i * 86400,
                rating: "Buy".to_string(),
                target_price: None,
                stop_loss: None,
                reasoning_summary: "测试".to_string(),
                peg_value: None,
                actual_return: None,
                alpha_return: None,
                reflection: None,
                created_at: 1704067200 + i * 86400,
            };
            memory.store_decision(&decision).unwrap();
        }

        let pending = memory.get_pending_reflections().unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_format_context_for_prompt() {
        let decisions = vec![
            PastDecision {
                id: "1".to_string(),
                secid: "600519.SH".to_string(),
                stock_name: "贵州茅台".to_string(),
                decision_date: 1704067200,
                rating: "Buy".to_string(),
                target_price: None,
                stop_loss: None,
                reasoning_summary: "测试".to_string(),
                peg_value: None,
                actual_return: Some(5.0),
                alpha_return: Some(2.0),
                reflection: Some("【经验教训】\n买入评级验证成功".to_string()),
                created_at: 1704067200,
            },
        ];

        let text = DecisionMemory::format_context_for_prompt(&decisions);
        assert!(text.contains("Buy"));
        assert!(text.contains("+5.00%"));
        assert!(text.contains("+2.00%"));
    }
}
