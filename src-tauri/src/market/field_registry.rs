//! 字段映射集中管理
//! EM_FIELD_MAP + TENCENT_FIELD_MAP + 警告注释
//!
//! ⚠️ 注意：
//! - 东方财富字段编号可能随版本更新变化
//! - 腾讯字段位置由协议约定，相对稳定
//! - 所有字段映射需在数据源解析中实际验证

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// 东方财富 API 字段映射
/// key = API 返回的字段编号(f2, f3, ...)
/// value = StockQuote 字段名
pub static EM_FIELD_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // 基础字段
    m.insert("f2", "price");           // 最新价
    m.insert("f3", "changePercent");   // 涨跌幅
    m.insert("f4", "change");          // 涨跌额
    m.insert("f5", "volume");          // 成交量(手)
    m.insert("f6", "amount");          // 成交额
    m.insert("f7", "changeSpeed");     // 涨速
    m.insert("f8", "turnoverRate");    // 换手率(流通)
    m.insert("f9", "pe_ttm");          // 市盈率(TTM) — 注意：f9 实测为 TTM
    m.insert("f10", "volumeRatio");    // 量比
    m.insert("f11", "ytdChange");      // 年初至今涨幅
    m.insert("f12", "code");           // 股票代码
    m.insert("f13", "market");         // 市场编号
    m.insert("f14", "open");           // 开盘价
    m.insert("f15", "high");           // 最高价
    m.insert("f16", "low");            // 最低价
    m.insert("f17", "preClose");       // 昨收价
    m.insert("f18", "pe_static");      // 静态市盈率
    m.insert("f20", "totalMarketCap"); // 总市值
    m.insert("f23", "pb");             // 市净率
    m.insert("f62", "mainNetInflow");  // 主力净流入
    m.insert("f136", "pb_secondary");  // 市净率(备用)
    m.insert("f100", "isSuspended");   // 停牌标志
    m.insert("f105", "sealBreakCount");// 炸板次数
    m.insert("f184", "sealStrength");  // 封板强度
    m.insert("f221", "isMarginTarget");// 是否两融标的
    m
});

/// 腾讯财经 API 字段映射
/// key = 字段在 split('~') 后的索引位置
/// value = StockQuote 字段名
pub static TENCENT_FIELD_MAP: Lazy<HashMap<usize, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(1, "name");           // 股票名称
    m.insert(2, "code");           // 股票代码
    m.insert(3, "price");          // 最新价
    m.insert(4, "preClose");       // 昨收价
    m.insert(5, "open");           // 开盘价
    m.insert(6, "volume");         // 成交量(手)
    m.insert(8, "turnoverRate");   // 换手率
    m.insert(33, "high");          // 最高价
    m.insert(34, "low");           // 最低价
    m.insert(37, "amount");        // 成交额
    m.insert(38, "turnoverRate_secondary"); // 换手率(备用)
    m.insert(39, "pe_ttm");        // 市盈率
    m.insert(44, "totalMarketCap_secondary"); // 总市值(万元)
    m.insert(45, "totalMarketCap"); // 总市值(万元，另一字段)
    m
});
