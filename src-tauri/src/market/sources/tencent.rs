//! 腾讯财经数据源
//! GBK 解码 + 字段映射 + 批量请求 qt.gtimg.cn

use crate::config::constants;
use crate::market::{
    AdjustType, ExRightInfo, KlineBar, KlinePeriod, MarketDataSource, SearchResult, StockQuote,
    TimelineData, TimelinePoint,
};
use async_trait::async_trait;

/// 腾讯财经数据源
pub struct TencentSource {
    client: reqwest::Client,
}

impl TencentSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(constants::HTTP_TIMEOUT_SECS))
            .gzip(true)
            .build()
            .expect("Failed to build Tencent HTTP client");
        Self { client }
    }

    /// 将东财 secid "1.600519" 转为腾讯格式 "sh600519"
    fn to_tencent_code(secid: &str) -> String {
        let (market, code) = secid.split_once('.').unwrap_or(("1", ""));
        let prefix = if market == "1" { "sh" } else { "sz" };
        format!("{}{}", prefix, code)
    }

    /// 将腾讯格式 "sh600519" 转回 secid
    fn from_tencent_code(tencent_code: &str) -> String {
        let (prefix, code) = if tencent_code.starts_with("sh") {
            ("1", &tencent_code[2..])
        } else if tencent_code.starts_with("sz") {
            ("0", &tencent_code[2..])
        } else {
            ("1", tencent_code)
        };
        format!("{}.{}", prefix, code)
    }

    /// 解析腾讯行情字段
    /// 格式: v_sh600519="1~贵州茅台~600519~1850.00~..."
    fn parse_quote_line(line: &str) -> Option<StockQuote> {
        let parts: Vec<&str> = line.split('~').collect();
        if parts.len() < 48 {
            return None;
        }

        let parse_f = |idx: usize| -> f64 {
            parts.get(idx).and_then(|s| s.parse().ok()).unwrap_or(0.0)
        };
        let parse_i = |idx: usize| -> i64 {
            parts.get(idx).and_then(|s| s.parse().ok()).unwrap_or(0)
        };

        let code = parts.get(2)?.to_string();
        let name = parts.get(1)?.to_string();
        let market_prefix = if line.starts_with("v_sh") { "1" } else { "0" };
        let secid = format!("{}.{}", market_prefix, code);

        let price = parse_f(3);
        let pre_close = parse_f(4);
        let change = price - pre_close;
        let change_percent = if pre_close > 0.0 { (price - pre_close) / pre_close * 100.0 } else { 0.0 };

        // 涨跌停价（腾讯数据索引 47=涨停价, 48=跌停价）
        let limit_up_price = if parts.len() > 47 { Some(parse_f(47)).filter(|v| *v > 0.0) } else { None };
        let limit_down_price = if parts.len() > 48 { Some(parse_f(48)).filter(|v| *v > 0.0) } else { None };

        // 涨跌停判断（与东财源一致：价格与涨停/跌停价差距 < 0.02）
        let is_limit_up = limit_up_price
            .map(|lp| price > 0.0 && (price - lp).abs() < 0.02)
            .unwrap_or(false);
        let is_limit_down = limit_down_price
            .map(|dp| price > 0.0 && (price - dp).abs() < 0.02)
            .unwrap_or(false);

        // 接近涨停判断（涨幅 ≥ 板块限制幅度 × 80%）
        let board_type = crate::limit::board_type::BoardType::from_code(&code);
        let is_st = crate::market::exchange::is_st_stock(&name);
        let limit_rate = board_type.limit_rate(is_st);
        let is_near_limit_up = !is_limit_up
            && change_percent >= limit_rate * 0.8
            && change_percent < limit_rate;

        let stock_status = if is_st {
            crate::limit::board_type::StockStatus::ST
        } else if price == 0.0 && parse_i(6) == 0 {
            crate::limit::board_type::StockStatus::Suspended
        } else {
            crate::limit::board_type::StockStatus::Normal
        };

        Some(StockQuote {
            secid,
            code,
            name,
            price,
            change,
            change_percent,
            volume: parse_i(6),
            amount: parse_f(37),
            high: parse_f(33),
            low: parse_f(34),
            open: parse_f(5),
            pre_close,
            total_market_cap: parse_f(45) * 1e8, // 万元 → 元
            main_net_inflow: 0.0,
            market: if market_prefix == "1" { 1 } else { 0 },
            turnover_rate: parse_f(38),
            total_turnover_rate: None,
            pe_ttm: parse_f(39),
            pe_dynamic: None,
            pe_static: 0.0,
            pb: 0.0,
            volume_ratio: 0.0,
            volume_ratio_note: None,
            change_speed: 0.0,
            ytd_change: 0.0,
            board_type,
            stock_status,
            is_limit_up,
            is_limit_down,
            is_near_limit_up,
            limit_up_price,
            limit_down_price,
            is_suspended: price == 0.0 && parse_i(6) == 0,
            is_temp_suspended: false,
            temp_suspend_reason: None,
            temp_suspend_resume_time: None,
            seal_strength: None,
            seal_break_count: 0,
            is_margin_target: false,
            margin_balance: None,
            short_volume: None,
        })
    }
}

#[async_trait]
impl MarketDataSource for TencentSource {
    async fn fetch_quotes(&self, secids: &[String]) -> Result<Vec<StockQuote>, String> {
        if secids.is_empty() {
            return Ok(vec![]);
        }

        // 腾讯批量请求: qt.gtimg.cn/q=sh600519,sz000001
        let codes: Vec<String> = secids.iter().map(|s| Self::to_tencent_code(s)).collect();
        let url = format!("https://qt.gtimg.cn/q={}", codes.join(","));

        let resp = self.client.get(&url)
            .header("Referer", "https://gu.qq.com/")
            .send().await
            .map_err(|e| format!("腾讯行情请求失败: {}", e))?;

        // 腾讯返回 GBK 编码，需转换
        let bytes = resp.bytes().await.map_err(|e| format!("读取腾讯响应失败: {}", e))?;
        let (text, _, _) = encoding_rs::GBK.decode(&bytes);

        let mut quotes = Vec::new();
        for line in text.lines() {
            if let Some(quote) = Self::parse_quote_line(line.trim()) {
                quotes.push(quote);
            }
        }

        Ok(quotes)
    }

    async fn fetch_kline(
        &self,
        secid: &str,
        period: KlinePeriod,
        limit: u32,
        _adjust: AdjustType,
    ) -> Result<Vec<KlineBar>, String> {
        let tencent_code = Self::to_tencent_code(secid);
        let period_type = match period {
            KlinePeriod::Daily => "day",
            KlinePeriod::Weekly => "week",
            KlinePeriod::Monthly => "month",
            KlinePeriod::Min1 => "m1",
            KlinePeriod::Min5 => "m5",
            KlinePeriod::Min15 => "m15",
            KlinePeriod::Min30 => "m30",
            KlinePeriod::Min60 => "m60",
        };

        let url = format!(
            "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={},{},,,{},,{}",
            tencent_code, period_type, limit, if _adjust == AdjustType::None { "0" } else { "1" }
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://gu.qq.com/")
            .send().await
            .map_err(|e| format!("腾讯K线请求失败: {}", e))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| format!("腾讯K线解析失败: {}", e))?;

        let mut bars = Vec::new();
        // 解析腾讯 K 线数据结构
        if let Some(data) = body.get("data").and_then(|d| d.get(&tencent_code)) {
            let key = if period == KlinePeriod::Daily || period == KlinePeriod::Weekly || period == KlinePeriod::Monthly {
                if let Some(_qfqday) = data.get("qfqday") {
                    "qfqday"
                } else {
                    "day"
                }
            } else {
                if let Some(_m_qfq) = data.get("m_qfq") {
                    "m_qfq"
                } else {
                    "m"
                }
            };

            if let Some(arr) = data.get(key).and_then(|v| v.as_array()) {
                for item in arr {
                    if let Some(item_arr) = item.as_array() {
                        let time_str = item_arr.get(0).and_then(|v| v.as_str()).unwrap_or("");
                        let time = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M")
                            .map(|d| d.and_utc().timestamp())
                            .unwrap_or_else(|_| {
                                chrono::NaiveDate::parse_from_str(time_str, "%Y-%m-%d")
                                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
                                    .unwrap_or(0)
                            });

                        bars.push(KlineBar {
                            time,
                            open: item_arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            close: item_arr.get(2).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            high: item_arr.get(3).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            low: item_arr.get(4).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            volume: item_arr.get(5).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0),
                            amount: 0.0,
                            change_percent: 0.0,
                        });
                    }
                }
            }
        }

        Ok(bars)
    }

    async fn fetch_timeline(&self, secid: &str) -> Result<TimelineData, String> {
        let tencent_code = Self::to_tencent_code(secid);

        // 腾讯分时接口: qt.gtimg.cn/q=s_sh600519 返回分时数据
        // 实际分时数据需用 data.gtimg.cn/flashdata 接口
        let url = format!(
            "https://web.ifzq.gtimg.cn/appstock/app/minute/query?_var=min_data&code={}",
            tencent_code
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://gu.qq.com/")
            .send().await
            .map_err(|e| format!("腾讯分时请求失败: {}", e))?;

        let bytes = resp.bytes().await.map_err(|e| format!("读取腾讯分时响应失败: {}", e))?;
        let (text, _, _) = encoding_rs::GBK.decode(&bytes);

        // 解析 JSONP: min_data={...}
        let json_str = if let Some(start) = text.find('=') {
            &text[start + 1..]
        } else {
            &text
        };

        let body: serde_json::Value = serde_json::from_str(json_str.trim())
            .map_err(|e| format!("腾讯分时解析失败: {}", e))?;

        let mut points = Vec::new();
        let mut pre_close = 0.0_f64;
        let mut name = String::new();

        // 解析分时数据
        if let Some(data) = body.get("data").and_then(|d| d.get(&tencent_code)) {
            // 获取昨收价和名称
            if let Some(info) = data.get("qt") {
                pre_close = info.get("4").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                name = info.get("1").and_then(|v| v.as_str()).unwrap_or("").to_string();
            }

            // 解析分时点数据: "0930 1850.00 1848.50 1234" → 时间 价格 均价 成交量
            if let Some(min_data) = data.get("data").and_then(|v| v.as_str()) {
                for line in min_data.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let time_str = if parts[0].len() == 4 {
                            format!("{}:{}", &parts[0][..2], &parts[0][2..])
                        } else {
                            parts[0].to_string()
                        };
                        let price = parts[1].parse().unwrap_or(0.0);
                        let avg_price = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(price);
                        let volume = parts.get(3).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);

                        points.push(TimelinePoint {
                            time: time_str,
                            price,
                            avg_price,
                            volume,
                        });
                    }
                }
            }
        }

        Ok(TimelineData {
            secid: secid.to_string(),
            name,
            pre_close,
            points,
        })
    }

    async fn search(&self, keyword: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!(
            "https://smartbox.gtimg.cn/s3/?q={}&t=all",
            urlencoding::encode(keyword)
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://gu.qq.com/")
            .send().await
            .map_err(|e| format!("腾讯搜索请求失败: {}", e))?;

        let bytes = resp.bytes().await.map_err(|e| format!("读取搜索响应失败: {}", e))?;
        let (text, _, _) = encoding_rs::GBK.decode(&bytes);

        let mut results = Vec::new();
        for line in text.lines() {
            // 格式: v_hint_sh600519="贵州茅台~sh600519~..."
            let parts: Vec<&str> = line.split('~').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                let code_full = parts.get(1).unwrap_or(&"").to_string();
                let (market, code) = if code_full.starts_with("sh") {
                    ("1", &code_full[2..])
                } else if code_full.starts_with("sz") {
                    ("0", &code_full[2..])
                } else {
                    continue;
                };

                results.push(SearchResult {
                    secid: format!("{}.{}", market, code),
                    code: code.to_string(),
                    name,
                    market: if market == "1" { "SH".to_string() } else { "SZ".to_string() },
                });
            }
        }

        Ok(results)
    }

    async fn fetch_exrights(&self, secid: &str) -> Result<Vec<ExRightInfo>, String> {
        let tencent_code = Self::to_tencent_code(secid);

        // 腾讯除权除息接口
        let url = format!(
            "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={},day,,,1,,0",
            tencent_code
        );

        let resp = self.client.get(&url)
            .header("Referer", "https://gu.qq.com/")
            .send().await
            .map_err(|e| format!("腾讯除权请求失败: {}", e))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| format!("腾讯除权解析失败: {}", e))?;

        let mut exrights = Vec::new();

        // 解析除权除息数据
        // 腾讯返回结构: data.{code}.xt -> 除权数组
        if let Some(data) = body.get("data").and_then(|d| d.get(&tencent_code)) {
            if let Some(xt_arr) = data.get("xt").and_then(|v| v.as_array()) {
                for item in xt_arr {
                    if let Some(item_arr) = item.as_array() {
                        // 格式: [日期, 送股, 配股, 配股价, 分红]
                        let ex_date = item_arr.get(0).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let bonus_share = item_arr.get(1).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let allot_share = item_arr.get(2).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let allot_price = item_arr.get(3).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let dividend = item_arr.get(4).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);

                        // 跳过空记录
                        if bonus_share == 0.0 && allot_share == 0.0 && dividend == 0.0 {
                            continue;
                        }

                        exrights.push(ExRightInfo {
                            secid: secid.to_string(),
                            ex_date,
                            bonus_share,
                            allot_share,
                            allot_price,
                            dividend,
                        });
                    }
                }
            }
        }

        Ok(exrights)
    }

    fn name(&self) -> &'static str {
        "腾讯财经"
    }

    fn priority(&self) -> u8 {
        2 // 东财(0)之后，同花顺(3)之前
    }
}
