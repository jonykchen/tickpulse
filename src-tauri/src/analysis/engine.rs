//! 分析引擎骨架 + 数据结构 + 端到端流程

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::DbPool;
use crate::market::MarketDataSource;

use super::debate::DebateEngine;
use super::dimensions::competitive::analyze_competitive_position;
use super::dimensions::financial::analyze_financial_health;
use super::dimensions::growth::analyze_growth_potential;
use super::dimensions::industry::analyze_industry_trend;
use super::dimensions::management::analyze_management_quality;
use super::dimensions::technical::analyze_technical_signals;
use super::dimensions::valuation::analyze_valuation;
use super::llm::create_client;
use super::quality_gate::evaluate_quality;

/// API Key 混淆密钥（简单 XOR 保护，防止明文存储）
const OBFUSCATION_KEY: &[u8] = b"TickPulse2024!Secure";

/// XOR 混淆加密（API Key → hex 编码存储）
fn xor_obfuscate(input: &str) -> String {
    input
        .bytes()
        .enumerate()
        .map(|(i, b)| format!("{:02x}", b ^ OBFUSCATION_KEY[i % OBFUSCATION_KEY.len()]))
        .collect()
}

/// XOR 混淆解密（hex 编码 → 原始 API Key）
fn xor_deobfuscate(hex: &str) -> String {
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect();
    bytes
        .iter()
        .enumerate()
        .map(|(i, b)| (b ^ OBFUSCATION_KEY[i % OBFUSCATION_KEY.len()]) as char)
        .collect()
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 供应商: anthropic / openai / ollama / deepseek
    pub provider: CloudProvider,
    /// API Key
    pub api_key: Option<String>,
    /// 模型名称
    pub model: String,
    /// Base URL（Ollama 使用本地地址）
    pub base_url: Option<String>,
    /// LLM 模式：本地 / 云端
    pub mode: LlmMode,
}

/// LLM 模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LlmMode {
    /// 云端 LLM（Anthropic/OpenAI/DeepSeek）
    Cloud,
    /// 本地 LLM（Ollama）
    Local,
}

/// 云端供应商
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CloudProvider {
    Anthropic,
    OpenAI,
    DeepSeek,
    Ollama,
}

impl CloudProvider {
    pub fn from_str(s: &str) -> Self {
        match s {
            "anthropic" => Self::Anthropic,
            "openai" => Self::OpenAI,
            "deepseek" => Self::DeepSeek,
            "ollama" => Self::Ollama,
            _ => Self::Anthropic,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAI => "openai",
            Self::DeepSeek => "deepseek",
            Self::Ollama => "ollama",
        }
    }
}

/// 分析维度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnalysisDimension {
    /// 行业趋势
    IndustryTrend,
    /// 竞争格局
    CompetitivePosition,
    /// 财务健康
    FinancialHealth,
    /// 管理层质量
    ManagementQuality,
    /// 成长性评估
    GrowthPotential,
    /// 估值分析（含 PEG）
    Valuation,
    /// 技术面信号
    TechnicalSignals,
}

impl AnalysisDimension {
    pub fn all() -> Vec<Self> {
        vec![
            Self::IndustryTrend,
            Self::CompetitivePosition,
            Self::FinancialHealth,
            Self::ManagementQuality,
            Self::GrowthPotential,
            Self::Valuation,
            Self::TechnicalSignals,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::IndustryTrend => "行业趋势",
            Self::CompetitivePosition => "竞争格局",
            Self::FinancialHealth => "财务健康",
            Self::ManagementQuality => "管理层质量",
            Self::GrowthPotential => "成长性评估",
            Self::Valuation => "估值分析",
            Self::TechnicalSignals => "技术面信号",
        }
    }
}

/// 维度报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionReport {
    pub dimension: AnalysisDimension,
    pub rating: DimensionRating,
    pub summary: String,
    pub key_points: Vec<String>,
    pub risks: Vec<String>,
    pub opportunities: Vec<String>,
    pub confidence: f64,
}

/// 维度评级（5级）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DimensionRating {
    /// 优秀 (A)
    A,
    /// 良好 (B)
    B,
    /// 中等 (C)
    C,
    /// 较差 (D)
    D,
    /// 极差 (F)
    F,
}

impl DimensionRating {
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 { Self::A }
        else if score >= 60.0 { Self::B }
        else if score >= 40.0 { Self::C }
        else if score >= 20.0 { Self::D }
        else { Self::F }
    }

    pub fn display(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            Self::A => 90.0,
            Self::B => 70.0,
            Self::C => 50.0,
            Self::D => 30.0,
            Self::F => 10.0,
        }
    }
}

/// 综合评级
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OverallRating {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

impl OverallRating {
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 { Self::StrongBuy }
        else if score >= 60.0 { Self::Buy }
        else if score >= 40.0 { Self::Hold }
        else if score >= 20.0 { Self::Sell }
        else { Self::StrongSell }
    }

    pub fn display(&self) -> &'static str {
        match self {
            Self::StrongBuy => "强烈推荐",
            Self::Buy => "推荐",
            Self::Hold => "观望",
            Self::Sell => "回避",
            Self::StrongSell => "强烈回避",
        }
    }
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: String,
    pub secid: String,
    pub stock_name: String,
    pub dimensions: HashMap<String, DimensionReport>,
    pub overall_rating: OverallRating,
    pub overall_score: f64,
    pub bull_argument: String,
    pub bear_argument: String,
    pub verdict: String,
    pub quality_score: f64,
    pub quality_grade: String,
    pub readable_report: String,
    pub created_at: i64,
}

/// 分析引擎
pub struct AnalysisEngine {
    db: Arc<DbPool>,
}

impl AnalysisEngine {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 执行完整分析流程
    /// 维度分析 → 多空辩论 → 质量门控 → 报告生成
    pub async fn run(
        &self,
        secid: &str,
        stock_name: &str,
        llm_config: &LlmConfig,
    ) -> Result<AnalysisResult, String> {
        let start_time = chrono::Utc::now().timestamp();

        // 1. 收集股票数据（从数据源获取）
        let stock_data = self.collect_stock_data(secid).await?;

        // 2. 7 维度分析（含 LLM 增强如果配置了 API Key）
        let mut dimension_reports = HashMap::new();
        let llm_client = if llm_config.api_key.is_some() || llm_config.provider == CloudProvider::Ollama {
            Some(create_client(llm_config))
        } else {
            None
        };

        let dimensions = AnalysisDimension::all();
        for dim in &dimensions {
            let report = self.analyze_dimension(dim, stock_name, &stock_data, llm_client.as_deref(), llm_config).await;
            dimension_reports.insert(format!("{:?}", dim), report);
        }

        // 3. 多空辩论
        let dimension_scores: Vec<(AnalysisDimension, f64)> = dimension_reports
            .iter()
            .map(|(k, r)| {
                let dim = match k.as_str() {
                    "IndustryTrend" => AnalysisDimension::IndustryTrend,
                    "CompetitivePosition" => AnalysisDimension::CompetitivePosition,
                    "FinancialHealth" => AnalysisDimension::FinancialHealth,
                    "ManagementQuality" => AnalysisDimension::ManagementQuality,
                    "GrowthPotential" => AnalysisDimension::GrowthPotential,
                    "Valuation" => AnalysisDimension::Valuation,
                    "TechnicalSignals" => AnalysisDimension::TechnicalSignals,
                    _ => AnalysisDimension::IndustryTrend,
                };
                (dim, r.rating.score())
            })
            .collect();

        let bull_points: Vec<String> = dimension_reports.values()
            .flat_map(|r| r.opportunities.clone())
            .collect();
        let bear_points: Vec<String> = dimension_reports.values()
            .flat_map(|r| r.risks.clone())
            .collect();

        let debate_result = DebateEngine::debate(&dimension_scores, &bull_points, &bear_points);

        // 4. 质量门控
        let confidences: Vec<f64> = dimension_reports.values().map(|r| r.confidence).collect();
        let avg_confidence = if confidences.is_empty() { 0.5 } else { confidences.iter().sum::<f64>() / confidences.len() as f64 };
        let quality = evaluate_quality(&confidences, avg_confidence, avg_confidence);

        // 5. 生成结果
        let result = AnalysisResult {
            id: uuid::Uuid::new_v4().to_string(),
            secid: secid.to_string(),
            stock_name: stock_name.to_string(),
            dimensions: dimension_reports,
            overall_rating: debate_result.overall_rating,
            overall_score: debate_result.final_score,
            bull_argument: debate_result.bull_argument,
            bear_argument: debate_result.bear_argument,
            verdict: debate_result.verdict,
            quality_score: quality.score,
            quality_grade: quality.grade.display().to_string(),
            readable_report: String::new(), // 会在保存前填充
            created_at: start_time,
        };

        // 6. 生成可读报告
        let readable = generate_readable_report_from_result(&result);

        // 7. 保存到数据库
        self.save_result(&result)?;

        Ok(AnalysisResult {
            readable_report: readable,
            ..result
        })
    }

    /// 收集股票数据
    async fn collect_stock_data(&self, secid: &str) -> Result<String, String> {
        let source = crate::market::sources::eastmoney::EastMoneySource::new();
        let quotes = source.fetch_quotes(&[secid.to_string()]).await?;
        let quote = quotes.first();

        // 获取 K 线数据
        let kline = source.fetch_kline(secid, crate::market::KlinePeriod::Daily, 60, crate::market::AdjustType::Forward).await?;

        let mut data = String::new();
        if let Some(q) = quote {
            data.push_str(&format!(
                "股票: {} ({})\n当前价: {:.2} 涨跌幅: {:.2}%\nPE(TTM): {:.1} PB: {:.1}\n换手率: {:.2}% 量比: {:.2}\n市值: {:.0}亿\n",
                q.name, q.code, q.price, q.change_percent,
                q.pe_ttm, q.pb, q.turnover_rate, q.volume_ratio,
                q.total_market_cap / 1e8
            ));
        }
        if !kline.is_empty() {
            data.push_str(&format!("最近{}日K线数据可用\n", kline.len()));
        }

        Ok(data)
    }

    /// 分析单个维度（含 LLM 增强路径）
    async fn analyze_dimension(
        &self,
        dim: &AnalysisDimension,
        stock_name: &str,
        data: &str,
        llm_client: Option<&dyn super::llm::LlmClient>,
        llm_config: &LlmConfig,
    ) -> DimensionReport {
        // 先用规则引擎获取基础报告
        let mut report = match dim {
            AnalysisDimension::IndustryTrend => analyze_industry_trend(stock_name, data),
            AnalysisDimension::CompetitivePosition => analyze_competitive_position(stock_name, data),
            AnalysisDimension::FinancialHealth => analyze_financial_health(stock_name, data),
            AnalysisDimension::ManagementQuality => analyze_management_quality(stock_name, data),
            AnalysisDimension::GrowthPotential => analyze_growth_potential(stock_name, data),
            AnalysisDimension::Valuation => analyze_valuation(stock_name, data),
            AnalysisDimension::TechnicalSignals => analyze_technical_signals(stock_name, data),
        };

        // 如果有 LLM 客户端，则增强分析
        if let Some(client) = llm_client {
            if let Ok(llm_response) = client.chat(
                &[
                    super::llm::ChatMessage {
                        role: super::llm::MessageRole::System,
                        content: super::prompts::SYSTEM_PROMPT.to_string(),
                    },
                    super::llm::ChatMessage {
                        role: super::llm::MessageRole::User,
                        content: super::prompts::dimension_prompt(dim.display_name(), stock_name, data),
                    },
                ],
                llm_config,
            ).await {
                // 尝试从 LLM 响应中提取结构化数据
                if let Ok(parsed) = serde_json::from_str::<super::schemas::DimensionOutputSchema>(&llm_response.content) {
                    report.summary = parsed.summary;
                    report.key_points = parsed.key_points;
                    report.risks = parsed.risks;
                    report.opportunities = parsed.opportunities;
                    report.confidence = parsed.confidence;
                    report.rating = DimensionRating::from_score(
                        match parsed.rating.as_str() {
                            "A" => 90.0, "B" => 70.0, "C" => 50.0, "D" => 30.0, "F" => 10.0,
                            _ => 50.0,
                        }
                    );
                }
            }
        }

        report
    }

    /// 保存分析结果到数据库
    fn save_result(&self, result: &AnalysisResult) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let dimensions_json = serde_json::to_string(&result.dimensions)
            .map_err(|e| format!("序列化维度数据失败: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO analysis_results (id, secid, stock_name, overall_rating, overall_score, bull_argument, bear_argument, verdict, quality_score, dimensions_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                result.id,
                result.secid,
                result.stock_name,
                format!("{:?}", result.overall_rating),
                result.overall_score,
                result.bull_argument,
                result.bear_argument,
                result.verdict,
                result.quality_score,
                dimensions_json,
                result.created_at,
            ],
        ).map_err(|e| format!("保存分析结果失败: {}", e))?;

        Ok(())
    }

    /// 获取历史分析结果
    pub fn get_history(&self, secid: Option<&str>, limit: u32) -> Result<Vec<AnalysisResult>, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;

        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(sid) = secid {
            ("SELECT * FROM analysis_results WHERE secid = ?1 ORDER BY created_at DESC LIMIT ?2",
             vec![Box::new(sid.to_string()), Box::new(limit)])
        } else {
            ("SELECT * FROM analysis_results ORDER BY created_at DESC LIMIT ?1",
             vec![Box::new(limit)])
        };

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows: Vec<AnalysisResult> = stmt
            .query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
                let id: String = row.get(0)?;
                let secid: String = row.get(1)?;
                let stock_name: String = row.get(2)?;
                let overall_rating_str: String = row.get(3)?;
                let overall_score: f64 = row.get(4)?;
                let bull_argument: String = row.get(5)?;
                let bear_argument: String = row.get(6)?;
                let verdict: String = row.get(7)?;
                let quality_score: f64 = row.get(8)?;
                let dimensions_json: String = row.get(9)?;
                let created_at: i64 = row.get(10)?;

                let dimensions: HashMap<String, DimensionReport> = serde_json::from_str(&dimensions_json).unwrap_or_default();

                let overall_rating = match overall_rating_str.as_str() {
                    "StrongBuy" => OverallRating::StrongBuy,
                    "Buy" => OverallRating::Buy,
                    "Sell" => OverallRating::Sell,
                    "StrongSell" => OverallRating::StrongSell,
                    _ => OverallRating::Hold,
                };

                Ok(AnalysisResult {
                    id, secid, stock_name, dimensions, overall_rating, overall_score,
                    bull_argument, bear_argument, verdict, quality_score,
                    quality_grade: String::new(), readable_report: String::new(),
                    created_at,
                })
            })
            .map_err(|e| format!("查询分析结果失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }

    /// 保存 LLM 配置到数据库
    pub fn save_llm_config(&self, config: &LlmConfig) -> Result<(), String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().timestamp();

        // 先将所有配置设为非活跃
        conn.execute("UPDATE llm_config SET is_active = 0", [])
            .map_err(|e| format!("更新LLM配置失败: {}", e))?;

        // 插入或更新
        conn.execute(
            "INSERT INTO llm_config (provider, model, api_key_encrypted, base_url, mode, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
            rusqlite::params![
                config.provider.as_str(),
                config.model,
                config.api_key.as_deref().map(xor_obfuscate).unwrap_or_default(),
                config.base_url.as_deref().unwrap_or(""),
                match config.mode { LlmMode::Cloud => "cloud", LlmMode::Local => "local" },
                now,
            ],
        ).map_err(|e| format!("保存LLM配置失败: {}", e))?;

        Ok(())
    }

    /// 获取当前活跃的 LLM 配置
    pub fn get_llm_config(&self) -> Result<LlmConfig, String> {
        let conn = self.db.conn().map_err(|e| e.to_string())?;
        let result = conn.query_row(
            "SELECT provider, model, api_key_encrypted, base_url, mode FROM llm_config WHERE is_active = 1 ORDER BY updated_at DESC LIMIT 1",
            [],
            |row| {
                let provider_str: String = row.get(0)?;
                let model: String = row.get(1)?;
                let api_key: String = row.get(2)?;
                let base_url: String = row.get(3)?;
                let mode_str: String = row.get(4)?;
                Ok((provider_str, model, api_key, base_url, mode_str))
            },
        );

        match result {
            Ok((provider_str, model, api_key, base_url, mode_str)) => Ok(LlmConfig {
                provider: CloudProvider::from_str(&provider_str),
                model,
                api_key: if api_key.is_empty() { None } else { Some(xor_deobfuscate(&api_key)) },
                base_url: if base_url.is_empty() { None } else { Some(base_url) },
                mode: if mode_str == "local" { LlmMode::Local } else { LlmMode::Cloud },
            }),
            Err(_) => Ok(LlmConfig {
                provider: CloudProvider::Anthropic,
                model: "claude-sonnet-4-6".to_string(),
                api_key: None,
                base_url: None,
                mode: LlmMode::Cloud,
            }),
        }
    }
}

/// 从 AnalysisResult 生成可读报告（不依赖 report 模块的引用问题）
fn generate_readable_report_from_result(result: &AnalysisResult) -> String {
    let mut report = String::new();
    report.push_str(&format!("# {} 分析报告\n\n", result.stock_name));
    report.push_str(&format!("**综合评级：{}** (评分: {:.1}/100)\n\n", result.overall_rating.display(), result.overall_score));
    report.push_str(&format!("**质量门控：{}** (评分: {:.1})\n\n", result.quality_grade, result.quality_score));

    report.push_str("## 多空观点\n\n");
    report.push_str(&format!("### 🐂 多方观点\n{}\n\n", result.bull_argument));
    report.push_str(&format!("### 🐻 空方观点\n{}\n\n", result.bear_argument));
    report.push_str(&format!("### ⚖️ 裁决\n{}\n\n", result.verdict));

    report.push_str("## 维度详情\n\n");
    for (key, dim) in &result.dimensions {
        report.push_str(&format!("### {} [{}]\n\n", key, dim.rating.display()));
        report.push_str(&format!("{}\n\n", dim.summary));

        if !dim.key_points.is_empty() {
            report.push_str("**关键要点：**\n");
            for p in &dim.key_points { report.push_str(&format!("- {}\n", p)); }
            report.push('\n');
        }
        if !dim.risks.is_empty() {
            report.push_str("**风险：**\n");
            for r in &dim.risks { report.push_str(&format!("- ⚠️ {}\n", r)); }
            report.push('\n');
        }
        if !dim.opportunities.is_empty() {
            report.push_str("**机会：**\n");
            for o in &dim.opportunities { report.push_str(&format!("- ✅ {}\n", o)); }
            report.push('\n');
        }
        report.push_str(&format!("置信度: {:.0}%\n\n", dim.confidence * 100.0));
    }

    report
}
