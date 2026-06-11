//! 分析检查点续传
//! 支持分析中断后从已完成维度恢复，避免重复计算

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::DbPool;

/// 分析检查点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCheckpoint {
    /// 确定性ID: format!("{}:{}", secid, date) 的 SHA256 前16位
    pub id: String,
    /// 股票标识 (如 "sh600519")
    pub secid: String,
    /// 股票名称
    pub stock_name: String,
    /// 分析日期 (YYYY-MM-DD)
    pub analysis_date: String,
    /// 已完成的维度列表
    pub completed_dimensions: Vec<String>,
    /// 待完成的维度列表
    pub pending_dimensions: Vec<String>,
    /// 部分结果 (维度名 -> JSON)
    pub partial_results: HashMap<String, serde_json::Value>,
    /// 创建时间戳
    pub created_at: i64,
    /// 更新时间戳
    pub updated_at: i64,
}

/// 检查点管理器
pub struct CheckpointManager {
    db: Arc<DbPool>,
}

impl CheckpointManager {
    /// 创建检查点管理器
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 生成确定性分析ID
    /// 同一只股票同一天 = 同一个ID
    pub fn analysis_id(secid: &str, date: &str) -> String {
        let data = format!("{}:{}", secid, date);
        let hash = Sha256::digest(data.as_bytes());
        format!("{:x}", hash)[..16].to_string()
    }

    /// 保存分析进度
    pub fn save_progress(&self, checkpoint: &AnalysisCheckpoint) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let completed_json = serde_json::to_string(&checkpoint.completed_dimensions)
            .map_err(|e| format!("序列化已完成维度失败: {}", e))?;
        let pending_json = serde_json::to_string(&checkpoint.pending_dimensions)
            .map_err(|e| format!("序列化待完成维度失败: {}", e))?;
        let partial_results_json = serde_json::to_string(&checkpoint.partial_results)
            .map_err(|e| format!("序列化部分结果失败: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO analysis_checkpoints
             (id, secid, stock_name, analysis_date, completed_dimensions, pending_dimensions, partial_results, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                checkpoint.id,
                checkpoint.secid,
                checkpoint.stock_name,
                checkpoint.analysis_date,
                completed_json,
                pending_json,
                partial_results_json,
                checkpoint.created_at,
                checkpoint.updated_at,
            ],
        ).map_err(|e| format!("保存检查点失败: {}", e))?;

        Ok(())
    }

    /// 检查是否有可恢复的检查点
    pub fn has_checkpoint(&self, secid: &str, date: &str) -> Result<bool, String> {
        let id = Self::analysis_id(secid, date);
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM analysis_checkpoints WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|e| format!("查询检查点失败: {}", e))?;

        Ok(count > 0)
    }

    /// 获取检查点
    pub fn get_checkpoint(&self, secid: &str, date: &str) -> Result<Option<AnalysisCheckpoint>, String> {
        let id = Self::analysis_id(secid, date);
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let result = conn.query_row(
            "SELECT id, secid, stock_name, analysis_date, completed_dimensions, pending_dimensions, partial_results, created_at, updated_at
             FROM analysis_checkpoints WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                let id: String = row.get(0)?;
                let secid: String = row.get(1)?;
                let stock_name: String = row.get(2)?;
                let analysis_date: String = row.get(3)?;
                let completed_json: String = row.get(4)?;
                let pending_json: String = row.get(5)?;
                let partial_results_json: String = row.get(6)?;
                let created_at: i64 = row.get(7)?;
                let updated_at: i64 = row.get(8)?;

                let completed_dimensions: Vec<String> = serde_json::from_str(&completed_json).unwrap_or_default();
                let pending_dimensions: Vec<String> = serde_json::from_str(&pending_json).unwrap_or_default();
                let partial_results: HashMap<String, serde_json::Value> = serde_json::from_str(&partial_results_json).unwrap_or_default();

                Ok(AnalysisCheckpoint {
                    id,
                    secid,
                    stock_name,
                    analysis_date,
                    completed_dimensions,
                    pending_dimensions,
                    partial_results,
                    created_at,
                    updated_at,
                })
            },
        );

        match result {
            Ok(checkpoint) => Ok(Some(checkpoint)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("获取检查点失败: {}", e)),
        }
    }

    /// 从检查点恢复分析
    /// 返回 Some(checkpoint) 如果有可恢复的检查点，否则返回 None
    pub fn resume_from_checkpoint(&self, secid: &str, date: &str) -> Result<Option<AnalysisCheckpoint>, String> {
        let checkpoint = self.get_checkpoint(secid, date)?;

        if let Some(ref cp) = checkpoint {
            // 检查检查点是否过期（超过24小时）
            let now = chrono::Utc::now().timestamp();
            let age_hours = (now - cp.updated_at) / 3600;

            if age_hours > 24 {
                // 检查点已过期，删除它
                tracing::info!(
                    "检查点已过期 ({}小时前)，删除并重新开始: {}",
                    age_hours,
                    cp.id
                );
                self.delete_checkpoint(secid, date)?;
                return Ok(None);
            }

            // 检查是否还有待完成的维度
            if cp.pending_dimensions.is_empty() {
                // 所有维度已完成，删除检查点
                tracing::info!("检查点所有维度已完成，删除: {}", cp.id);
                self.delete_checkpoint(secid, date)?;
                return Ok(None);
            }

            tracing::info!(
                "从检查点恢复分析: {} (已完成 {} 个维度，待完成 {} 个)",
                cp.id,
                cp.completed_dimensions.len(),
                cp.pending_dimensions.len()
            );
        }

        Ok(checkpoint)
    }

    /// 删除检查点
    pub fn delete_checkpoint(&self, secid: &str, date: &str) -> Result<(), String> {
        let id = Self::analysis_id(secid, date);
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        conn.execute(
            "DELETE FROM analysis_checkpoints WHERE id = ?1",
            rusqlite::params![id],
        ).map_err(|e| format!("删除检查点失败: {}", e))?;

        Ok(())
    }

    /// 清理过期检查点（>24小时）
    /// 返回删除的记录数
    pub fn cleanup_old_checkpoints(&self) -> Result<usize, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let cutoff = chrono::Utc::now().timestamp() - 24 * 3600;

        let rows_affected = conn
            .execute(
                "DELETE FROM analysis_checkpoints WHERE updated_at < ?1",
                rusqlite::params![cutoff],
            )
            .map_err(|e| format!("清理过期检查点失败: {}", e))?;

        if rows_affected > 0 {
            tracing::info!("已清理 {} 个过期检查点", rows_affected);
        }

        Ok(rows_affected)
    }

    /// 更新检查点进度（完成一个维度后调用）
    pub fn update_progress(
        &self,
        secid: &str,
        date: &str,
        completed_dimension: &str,
        partial_result: serde_json::Value,
    ) -> Result<(), String> {
        let mut checkpoint = match self.get_checkpoint(secid, date)? {
            Some(cp) => cp,
            None => {
                return Err("检查点不存在".to_string());
            }
        };

        // 从待完成列表移除
        checkpoint.pending_dimensions.retain(|d| d != completed_dimension);

        // 添加到已完成列表（如果不在）
        if !checkpoint.completed_dimensions.contains(&completed_dimension.to_string()) {
            checkpoint.completed_dimensions.push(completed_dimension.to_string());
        }

        // 保存部分结果
        checkpoint.partial_results.insert(completed_dimension.to_string(), partial_result);

        // 更新时间戳
        checkpoint.updated_at = chrono::Utc::now().timestamp();

        self.save_progress(&checkpoint)?;

        // 如果所有维度完成，删除检查点
        if checkpoint.pending_dimensions.is_empty() {
            self.delete_checkpoint(secid, date)?;
            tracing::info!("所有维度已完成，已删除检查点: {}", checkpoint.id);
        }

        Ok(())
    }

    /// 创建新的检查点
    pub fn create_checkpoint(
        &self,
        secid: &str,
        stock_name: &str,
        date: &str,
        dimensions: &[String],
    ) -> Result<AnalysisCheckpoint, String> {
        let id = Self::analysis_id(secid, date);
        let now = chrono::Utc::now().timestamp();

        let checkpoint = AnalysisCheckpoint {
            id,
            secid: secid.to_string(),
            stock_name: stock_name.to_string(),
            analysis_date: date.to_string(),
            completed_dimensions: Vec::new(),
            pending_dimensions: dimensions.to_vec(),
            partial_results: HashMap::new(),
            created_at: now,
            updated_at: now,
        };

        self.save_progress(&checkpoint)?;

        tracing::info!(
            "创建新检查点: {} (待完成 {} 个维度)",
            checkpoint.id,
            checkpoint.pending_dimensions.len()
        );

        Ok(checkpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_id_deterministic() {
        let id1 = CheckpointManager::analysis_id("sh600519", "2024-01-15");
        let id2 = CheckpointManager::analysis_id("sh600519", "2024-01-15");

        // 同样的输入应该产生同样的ID
        assert_eq!(id1, id2);

        // ID长度应为16
        assert_eq!(id1.len(), 16);
    }

    #[test]
    fn test_analysis_id_different_inputs() {
        let id1 = CheckpointManager::analysis_id("sh600519", "2024-01-15");
        let id2 = CheckpointManager::analysis_id("sh600519", "2024-01-16");
        let id3 = CheckpointManager::analysis_id("sz000001", "2024-01-15");

        // 不同的输入应该产生不同的ID
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_checkpoint_serialization() {
        let checkpoint = AnalysisCheckpoint {
            id: "abc123".to_string(),
            secid: "sh600519".to_string(),
            stock_name: "贵州茅台".to_string(),
            analysis_date: "2024-01-15".to_string(),
            completed_dimensions: vec!["IndustryTrend".to_string()],
            pending_dimensions: vec!["FinancialHealth".to_string(), "Valuation".to_string()],
            partial_results: {
                let mut map = HashMap::new();
                map.insert("IndustryTrend".to_string(), serde_json::json!({"score": 80}));
                map
            },
            created_at: 1705305600,
            updated_at: 1705305600,
        };

        // 验证序列化和反序列化
        let json = serde_json::to_string(&checkpoint).unwrap();
        let deserialized: AnalysisCheckpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(checkpoint.id, deserialized.id);
        assert_eq!(checkpoint.secid, deserialized.secid);
        assert_eq!(checkpoint.stock_name, deserialized.stock_name);
        assert_eq!(checkpoint.completed_dimensions.len(), 1);
        assert_eq!(checkpoint.pending_dimensions.len(), 2);
    }
}
