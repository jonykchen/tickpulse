//! 数据供应商路由
//!
//! 提供数据源能力抽象和自动降级机制，支持多数据源优先级调度。

use crate::market::MarketDataSource;
use crate::sidecar::SidecarManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 数据能力枚举
///
/// 定义系统支持的19种数据获取能力，每种能力可配置多个数据源降级链。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataCapability {
    /// K线数据（日K、周K、月K、分钟K）
    Kline,
    /// 市盈率、市净率、总市值
    PePbMarketCap,
    /// 实时行情
    RealtimeQuote,
    /// 分时走势数据
    Timeline,
    /// 北向资金流向
    NorthboundFlow,
    /// 龙虎榜数据
    DragonTiger,
    /// 大宗交易数据
    BlockTrade,
    /// 融资融券数据
    MarginData,
    /// 财务报表（三大表）
    FinancialReport,
    /// 股东户数
    ShareholderCount,
    /// 解禁计划
    LockupSchedule,
    /// 行业市盈率
    IndustryPe,
    /// 概念板块
    ConceptBlocks,
    /// 资金流向
    FundFlow,
    /// 分时资金流向
    FundFlowMinute,
    /// 一致预期EPS
    ConsensusEps,
    /// 新闻资讯
    News,
    /// 公告信息
    Announcement,
    /// 研报数据
    ResearchReport,
}

impl DataCapability {
    /// 获取能力名称（用于日志和调试）
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kline => "K线数据",
            Self::PePbMarketCap => "PE/PB/市值",
            Self::RealtimeQuote => "实时行情",
            Self::Timeline => "分时走势",
            Self::NorthboundFlow => "北向资金",
            Self::DragonTiger => "龙虎榜",
            Self::BlockTrade => "大宗交易",
            Self::MarginData => "融资融券",
            Self::FinancialReport => "财务报表",
            Self::ShareholderCount => "股东户数",
            Self::LockupSchedule => "解禁计划",
            Self::IndustryPe => "行业PE",
            Self::ConceptBlocks => "概念板块",
            Self::FundFlow => "资金流向",
            Self::FundFlowMinute => "分时资金流",
            Self::ConsensusEps => "一致预期EPS",
            Self::News => "新闻资讯",
            Self::Announcement => "公告信息",
            Self::ResearchReport => "研报数据",
        }
    }
}

/// 数据供应商路由器
///
/// 管理多个数据源的优先级调度，支持自动降级和能力路由。
pub struct DataVendorRouter {
    /// 已注册的数据源列表
    sources: Vec<Arc<dyn MarketDataSource>>,
    /// Sidecar 管理器（用于 Python 扩展能力）
    sidecar: Option<Arc<SidecarManager>>,
    /// 能力到数据源优先级列表的映射
    fallback_chains: HashMap<DataCapability, Vec<String>>,
}

impl DataVendorRouter {
    /// 创建新的路由器实例
    ///
    /// # Arguments
    /// * `sources` - 数据源列表
    /// * `sidecar` - Sidecar 管理器（可选）
    ///
    /// # Returns
    /// 初始化后的路由器，包含默认降级链配置
    pub fn new(
        sources: Vec<Arc<dyn MarketDataSource>>,
        sidecar: Option<Arc<SidecarManager>>,
    ) -> Self {
        let fallback_chains = Self::build_default_fallback_chains();
        Self {
            sources,
            sidecar,
            fallback_chains,
        }
    }

    /// 构建默认降级链
    fn build_default_fallback_chains() -> HashMap<DataCapability, Vec<String>> {
        let mut chains = HashMap::new();

        // K线数据：东方财富 -> 腾讯 -> 新浪
        chains.insert(DataCapability::Kline, vec![
            "eastmoney".to_string(),
            "tencent".to_string(),
            "sina".to_string(),
        ]);

        // PE/PB/市值：东方财富 -> 腾讯
        chains.insert(DataCapability::PePbMarketCap, vec![
            "eastmoney".to_string(),
            "tencent".to_string(),
        ]);

        // 实时行情：东方财富 -> 腾讯
        chains.insert(DataCapability::RealtimeQuote, vec![
            "eastmoney".to_string(),
            "tencent".to_string(),
        ]);

        // 分时走势：东方财富 -> 腾讯
        chains.insert(DataCapability::Timeline, vec![
            "eastmoney".to_string(),
            "tencent".to_string(),
        ]);

        // 北向资金：东方财富 -> Sidecar（Python扩展）
        chains.insert(DataCapability::NorthboundFlow, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 龙虎榜：东方财富 -> Sidecar
        chains.insert(DataCapability::DragonTiger, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 大宗交易：东方财富 -> Sidecar
        chains.insert(DataCapability::BlockTrade, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 融资融券：东方财富 -> Sidecar
        chains.insert(DataCapability::MarginData, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 财务报表：仅 Sidecar 支持
        chains.insert(DataCapability::FinancialReport, vec![
            "sidecar".to_string(),
        ]);

        // 股东户数：东方财富 -> Sidecar
        chains.insert(DataCapability::ShareholderCount, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 解禁计划：东方财富 -> Sidecar
        chains.insert(DataCapability::LockupSchedule, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 行业PE：东方财富 -> Sidecar
        chains.insert(DataCapability::IndustryPe, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 概念板块：东方财富 -> 同花顺
        chains.insert(DataCapability::ConceptBlocks, vec![
            "eastmoney".to_string(),
            "ths".to_string(),
        ]);

        // 资金流向：东方财富 -> 腾讯
        chains.insert(DataCapability::FundFlow, vec![
            "eastmoney".to_string(),
            "tencent".to_string(),
        ]);

        // 分时资金流向：东方财富
        chains.insert(DataCapability::FundFlowMinute, vec![
            "eastmoney".to_string(),
        ]);

        // 一致预期EPS：Sidecar
        chains.insert(DataCapability::ConsensusEps, vec![
            "sidecar".to_string(),
        ]);

        // 新闻资讯：东方财富 -> Sidecar
        chains.insert(DataCapability::News, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 公告信息：东方财富 -> Sidecar
        chains.insert(DataCapability::Announcement, vec![
            "eastmoney".to_string(),
            "sidecar".to_string(),
        ]);

        // 研报数据：Sidecar
        chains.insert(DataCapability::ResearchReport, vec![
            "sidecar".to_string(),
        ]);

        chains
    }

    /// 按能力获取数据，自动降级
    ///
    /// 按照配置的优先级依次尝试数据源，直到成功获取数据或所有数据源均失败。
    ///
    /// # Arguments
    /// * `capability` - 数据能力类型
    /// * `params` - 请求参数（JSON格式）
    ///
    /// # Returns
    /// 成功时返回数据，失败时返回错误信息
    pub async fn fetch(
        &self,
        capability: DataCapability,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let chain = self.get_vendor_chain(&capability);

        if chain.is_empty() {
            return Err(format!("能力 {} 没有配置数据源", capability.name()));
        }

        let mut last_error = String::new();

        for vendor in chain {
            tracing::debug!(
                "尝试从 {} 获取 {} 数据，参数: {:?}",
                vendor,
                capability.name(),
                params
            );

            match self.try_vendor(vendor, &capability, params).await {
                Ok(data) => {
                    tracing::info!("成功从 {} 获取 {} 数据", vendor, capability.name());
                    return Ok(data);
                }
                Err(e) => {
                    tracing::warn!("从 {} 获取 {} 数据失败: {}", vendor, capability.name(), e);
                    let prefix = if last_error.is_empty() { String::new() } else { format!("{}, ", last_error) };
                    last_error = format!("{}{}: {}", prefix, vendor, e);
                }
            }
        }

        Err(format!("所有数据源均失败: {}", last_error))
    }

    /// 获取能力对应的数据源列表
    ///
    /// # Arguments
    /// * `capability` - 数据能力类型
    ///
    /// # Returns
    /// 数据源名称列表（按优先级排序）
    fn get_vendor_chain(&self, capability: &DataCapability) -> Vec<&str> {
        self.fallback_chains
            .get(capability)
            .map(|chain| chain.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// 尝试从指定数据源获取数据
    ///
    /// # Arguments
    /// * `vendor` - 数据源名称
    /// * `capability` - 数据能力类型
    /// * `params` - 请求参数
    ///
    /// # Returns
    /// 成功时返回数据，失败时返回错误信息
    async fn try_vendor(
        &self,
        vendor: &str,
        capability: &DataCapability,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // 处理 sidecar 特殊情况
        if vendor == "sidecar" {
            return self.try_sidecar(capability, params).await;
        }

        // 查找对应的数据源
        let source = self.sources
            .iter()
            .find(|s| s.name() == vendor)
            .ok_or_else(|| format!("数据源 {} 未注册", vendor))?;

        // 根据能力类型调用相应方法
        self.dispatch_to_source(source.as_ref(), capability, params).await
    }

    /// 分发请求到具体数据源
    async fn dispatch_to_source(
        &self,
        source: &dyn MarketDataSource,
        capability: &DataCapability,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match capability {
            DataCapability::Kline => {
                let secid = params.get("secid")
                    .and_then(|v| v.as_str())
                    .ok_or("缺少 secid 参数")?;
                let period = params.get("period")
                    .and_then(|v| v.as_str())
                    .unwrap_or("daily");
                let limit = params.get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as u32;
                let adjust = params.get("adjust")
                    .and_then(|v| v.as_str())
                    .unwrap_or("forward");

                let kline_period = self.parse_kline_period(period)?;
                let adjust_type = self.parse_adjust_type(adjust)?;

                let bars = source.fetch_kline(secid, kline_period, limit, adjust_type).await?;
                Ok(serde_json::to_value(bars).map_err(|e| e.to_string())?)
            }
            DataCapability::RealtimeQuote => {
                let secids = params.get("secids")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                    .ok_or("缺少 secids 参数")?;

                let quotes = source.fetch_quotes(&secids).await?;
                Ok(serde_json::to_value(quotes).map_err(|e| e.to_string())?)
            }
            DataCapability::Timeline => {
                let secid = params.get("secid")
                    .and_then(|v| v.as_str())
                    .ok_or("缺少 secid 参数")?;

                let timeline = source.fetch_timeline(secid).await?;
                Ok(serde_json::to_value(timeline).map_err(|e| e.to_string())?)
            }
            _ => {
                // 对于暂不直接支持的能力，返回提示
                Err(format!("数据源 {} 暂不支持 {} 能力", source.name(), capability.name()))
            }
        }
    }

    /// 尝试通过 Sidecar 获取数据
    async fn try_sidecar(
        &self,
        capability: &DataCapability,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let sidecar = self.sidecar
            .as_ref()
            .ok_or("Sidecar 未配置")?;

        if !sidecar.is_running().await {
            return Err("Sidecar 未运行".to_string());
        }

        let action = capability_to_sidecar_action(capability);
        sidecar.send_request(action, params.clone()).await
    }

    /// 解析K线周期参数
    fn parse_kline_period(&self, period: &str) -> Result<crate::market::KlinePeriod, String> {
        use crate::market::KlinePeriod;
        match period.to_lowercase().as_str() {
            "1min" | "min1" => Ok(KlinePeriod::Min1),
            "5min" | "min5" => Ok(KlinePeriod::Min5),
            "15min" | "min15" => Ok(KlinePeriod::Min15),
            "30min" | "min30" => Ok(KlinePeriod::Min30),
            "60min" | "min60" | "hour" => Ok(KlinePeriod::Min60),
            "daily" | "day" | "d" => Ok(KlinePeriod::Daily),
            "weekly" | "week" | "w" => Ok(KlinePeriod::Weekly),
            "monthly" | "month" | "m" => Ok(KlinePeriod::Monthly),
            _ => Err(format!("不支持的K线周期: {}", period)),
        }
    }

    /// 解析复权类型参数
    fn parse_adjust_type(&self, adjust: &str) -> Result<crate::market::AdjustType, String> {
        use crate::market::AdjustType;
        match adjust.to_lowercase().as_str() {
            "none" | "no" | "0" => Ok(AdjustType::None),
            "forward" | "qfq" | "1" => Ok(AdjustType::Forward),
            "backward" | "hfq" | "2" => Ok(AdjustType::Backward),
            _ => Err(format!("不支持的复权类型: {}", adjust)),
        }
    }

    /// 设置自定义降级链
    ///
    /// # Arguments
    /// * `capability` - 数据能力类型
    /// * `vendors` - 数据源优先级列表
    pub fn set_fallback_chain(&mut self, capability: DataCapability, vendors: Vec<String>) {
        self.fallback_chains.insert(capability, vendors);
    }

    /// 获取指定数据源
    ///
    /// # Arguments
    /// * `name` - 数据源名称
    pub fn get_source(&self, name: &str) -> Option<Arc<dyn MarketDataSource>> {
        self.sources
            .iter()
            .find(|s| s.name() == name)
            .cloned()
    }

    /// 获取所有已注册的数据源名称
    pub fn get_registered_sources(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.name()).collect()
    }

    /// 检查 Sidecar 是否可用
    pub async fn is_sidecar_available(&self) -> bool {
        match self.sidecar.as_ref() {
            Some(s) => s.is_running().await,
            None => false,
        }
    }
}

/// 将数据能力转换为 Sidecar action 名称
fn capability_to_sidecar_action(capability: &DataCapability) -> &'static str {
    match capability {
        DataCapability::Kline => "fetch_kline",
        DataCapability::PePbMarketCap => "fetch_pe_pb",
        DataCapability::RealtimeQuote => "fetch_quotes",
        DataCapability::Timeline => "fetch_timeline",
        DataCapability::NorthboundFlow => "fetch_northbound",
        DataCapability::DragonTiger => "fetch_dragon_tiger",
        DataCapability::BlockTrade => "fetch_block_trade",
        DataCapability::MarginData => "fetch_margin",
        DataCapability::FinancialReport => "fetch_financial_report",
        DataCapability::ShareholderCount => "fetch_shareholder_count",
        DataCapability::LockupSchedule => "fetch_lockup",
        DataCapability::IndustryPe => "fetch_industry_pe",
        DataCapability::ConceptBlocks => "fetch_concept_blocks",
        DataCapability::FundFlow => "fetch_fund_flow",
        DataCapability::FundFlowMinute => "fetch_fund_flow_minute",
        DataCapability::ConsensusEps => "fetch_consensus_eps",
        DataCapability::News => "fetch_news",
        DataCapability::Announcement => "fetch_announcement",
        DataCapability::ResearchReport => "fetch_research_report",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_name() {
        assert_eq!(DataCapability::Kline.name(), "K线数据");
        assert_eq!(DataCapability::NorthboundFlow.name(), "北向资金");
        assert_eq!(DataCapability::FinancialReport.name(), "财务报表");
    }

    #[test]
    fn test_default_fallback_chain_kline() {
        let chains = DataVendorRouter::build_default_fallback_chains();
        let kline_chain = chains.get(&DataCapability::Kline).unwrap();
        assert_eq!(kline_chain, &vec!["eastmoney", "tencent", "sina"]);
    }

    #[test]
    fn test_default_fallback_chain_financial_report() {
        let chains = DataVendorRouter::build_default_fallback_chains();
        let chain = chains.get(&DataCapability::FinancialReport).unwrap();
        assert_eq!(chain, &vec!["sidecar"]);
    }

    #[test]
    fn test_kline_period_parsing() {
        let router = DataVendorRouter::new(vec![], None);

        assert!(matches!(router.parse_kline_period("daily").unwrap(), crate::market::KlinePeriod::Daily));
        assert!(matches!(router.parse_kline_period("5min").unwrap(), crate::market::KlinePeriod::Min5));
        assert!(matches!(router.parse_kline_period("week").unwrap(), crate::market::KlinePeriod::Weekly));
    }

    #[test]
    fn test_adjust_type_parsing() {
        let router = DataVendorRouter::new(vec![], None);

        assert!(matches!(router.parse_adjust_type("forward").unwrap(), crate::market::AdjustType::Forward));
        assert!(matches!(router.parse_adjust_type("qfq").unwrap(), crate::market::AdjustType::Forward));
        assert!(matches!(router.parse_adjust_type("none").unwrap(), crate::market::AdjustType::None));
    }

    #[test]
    fn test_capability_serialization() {
        let cap = DataCapability::Kline;
        let json = serde_json::to_string(&cap).unwrap();
        assert_eq!(json, "\"kline\"");

        let parsed: DataCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, DataCapability::Kline);
    }
}
