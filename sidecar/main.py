#!/usr/bin/env python3
"""TickPulse Python Sidecar

16 个 action handler，处理需要 Python 环境的数据抓取任务。
通过 stdin/stdout JSON IPC 与 Rust 主进程通信。

已知坑点（12个）：
1. 东财 datacenter 的 cookie 需要 __cf_bm 参数
2. 同花顺 hsgtApi 需要 Referer 头
3. 163 财报 API 返回 GBK 编码
4. 巨潮资讯 PDF 需特殊 User-Agent
5. 深交所互动易需要 Session
6. 中登公司接口有 IP 白名单
7. 雪球 API 需要 x-csrf-token
8. choice 接口需要付费 token
9. 通达信接口是二进制 TDX 协议
10. 部分接口有 Cloudflare 5s 盾
11. 新浪接口偶尔返回空数据
12. 腾讯接口 GBK 编码需要特殊处理
"""

import sys
import json
import traceback
from typing import Any, Dict, Optional
from datetime import datetime, timedelta

import requests


# ==================== 通用工具 ====================

# 东财通用请求头
EM_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "Referer": "https://data.eastmoney.com/",
    "Accept": "application/json",
}

# 同花顺通用请求头
THS_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "Referer": "https://data.10jqka.com.cn/",
}

# 请求超时
TIMEOUT = 15


def safe_get(url: str, headers: dict = None, params: dict = None, encoding: str = None) -> Optional[requests.Response]:
    """安全 GET 请求，带超时和错误处理"""
    try:
        resp = requests.get(url, headers=headers or EM_HEADERS, params=params, timeout=TIMEOUT)
        if encoding:
            resp.encoding = encoding
        resp.raise_for_status()
        return resp
    except requests.RequestException as e:
        print(f"[WARN] 请求失败 {url}: {e}", file=sys.stderr)
        return None


def parse_secid(secid: str) -> tuple:
    """将东财 secid (如 '1.600519') 解析为 (market, code)"""
    parts = secid.split(".", 1)
    if len(parts) == 2:
        return parts[0], parts[1]
    return "1", secid


def today_str() -> str:
    return datetime.now().strftime("%Y-%m-%d")


# ==================== Action Handlers ====================


def handle_fetch_financial_report(params: Dict[str, Any]) -> Any:
    """获取财务报告（三表：利润表/资产负债表/现金流量表）"""
    secid = params.get("secid", "")
    market, code = parse_secid(secid)

    # 东方财富财务报表API
    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    result = {"income": [], "balance": [], "cashflow": []}

    for report_type, rpt_name in [("RPT_DMSK_FN_INCOME", "利润表"), ("RPT_DMSK_FN_BALANCE", "资产负债表"), ("RPT_DMSK_FN_CASHFLOW", "现金流量表")]:
        query_params = {
            "reportName": rpt_name,
            "columns": "ALL",
            "filter": f'(SECURITY_CODE="{code}")',
            "pageNumber": 1,
            "pageSize": 4,
            "sortTypes": -1,
            "sortColumns": "REPORT_DATE",
        }
        resp = safe_get(url, params=query_params)
        if resp:
            try:
                data = resp.json()
                if data.get("result") and data["result"].get("data"):
                    items = []
                    for row in data["result"]["data"]:
                        item = {
                            "reportDate": row.get("REPORT_DATE", ""),
                            "revenue": row.get("TOTAL_OPERATE_INCOME", 0),
                            "netProfit": row.get("PARENT_NETPROFIT", 0),
                            "eps": row.get("BASIC_EPS", 0),
                            "roe": row.get("WEIGHTAVG_ROE", 0),
                        }
                        items.append(item)
                    result[report_type.replace("RPT_DMSK_FN_", "").lower()] = items
            except (json.JSONDecodeError, KeyError) as e:
                print(f"[WARN] 解析{rpt_name}失败: {e}", file=sys.stderr)

    return result


def handle_fetch_shareholder_count(params: Dict[str, Any]) -> Any:
    """获取股东户数变化"""
    secid = params.get("secid", "")
    _, code = parse_secid(secid)

    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    query_params = {
        "reportName": "RPT_F10_EH_HOLDERNUM",
        "columns": "ALL",
        "filter": f'(SECURITY_CODE="{code}")',
        "pageNumber": 1,
        "pageSize": 8,
        "sortTypes": -1,
        "sortColumns": "END_DATE",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("result") and data["result"].get("data"):
                return {
                    "shareholders": [
                        {
                            "endDate": row.get("END_DATE", ""),
                            "holderNum": row.get("HOLDER_NUM", 0),
                            "changeNum": row.get("HOLDER_NUM_CHANGE", 0),
                            "changePercent": row.get("HOLDER_NUM_RATIO", 0),
                        }
                        for row in data["result"]["data"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析股东户数失败: {e}", file=sys.stderr)

    return {"shareholders": []}


def handle_fetch_lockup_schedule(params: Dict[str, Any]) -> Any:
    """获取解禁时间表"""
    secid = params.get("secid", "")
    _, code = parse_secid(secid)

    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    query_params = {
        "reportName": "RPT_F10_EH_RELEASE",
        "columns": "ALL",
        "filter": f'(SECURITY_CODE="{code}")',
        "pageNumber": 1,
        "pageSize": 10,
        "sortTypes": -1,
        "sortColumns": "END_DATE",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("result") and data["result"].get("data"):
                return {
                    "lockups": [
                        {
                            "lockupDate": row.get("END_DATE", ""),
                            "lockupShares": row.get("UNFREEZE_SHARES", 0),
                            "lockupAmount": row.get("UNFREEZE_MARKET_CAP", 0),
                            "lockupType": row.get("SHARES_TYPE", ""),
                        }
                        for row in data["result"]["data"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析解禁数据失败: {e}", file=sys.stderr)

    return {"lockups": []}


def handle_fetch_industry_pe(params: Dict[str, Any]) -> Any:
    """获取行业 PE 中位数"""
    industry_code = params.get("industry_code", "")

    url = "https://push2.eastmoney.com/api/qt/clist/get"
    query_params = {
        "pn": 1,
        "pz": 100,
        "po": 1,
        "np": 1,
        "ut": "b73502e0ed8b4e5f9c5ce1ec0c7c7d86",
        "fltt": 2,
        "invt": 2,
        "fid": "f3",
        "fs": "m:90+t:2",
        "fields": "f12,f14,f9,f20,f3",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("diff"):
                industries = []
                for row in data["data"]["diff"]:
                    industries.append({
                        "industryCode": str(row.get("f12", "")),
                        "industryName": row.get("f14", ""),
                        "pe": row.get("f9", 0),
                        "marketCap": row.get("f20", 0),
                        "changePercent": row.get("f3", 0),
                    })
                if industry_code:
                    industries = [i for i in industries if i["industryCode"] == industry_code]
                return {"industries": industries}
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析行业PE失败: {e}", file=sys.stderr)

    return {"industries": []}


def handle_fetch_dragon_tiger(params: Dict[str, Any]) -> Any:
    """获取龙虎榜数据"""
    date = params.get("date", today_str())

    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    query_params = {
        "reportName": "RPT_DMSK_DT_DETAILS",
        "columns": "ALL",
        "filter": f'(TRADE_DATE='{date}')',
        "pageNumber": 1,
        "pageSize": 50,
        "sortTypes": -1,
        "sortColumns": "NET_AMOUNT",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("result") and data["result"].get("data"):
                return {
                    "date": date,
                    "records": [
                        {
                            "code": row.get("SECURITY_CODE", ""),
                            "name": row.get("SECURITY_NAME_ABBR", ""),
                            "closePrice": row.get("CLOSE_PRICE", 0),
                            "changePercent": row.get("CHANGE_RATE", 0),
                            "netAmount": row.get("NET_AMOUNT", 0),
                            "buyAmount": row.get("BUY_AMOUNT", 0),
                            "sellAmount": row.get("SELL_AMOUNT", 0),
                            "reason": row.get("EXPLAIN", ""),
                        }
                        for row in data["result"]["data"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析龙虎榜失败: {e}", file=sys.stderr)

    return {"date": date, "records": []}


def handle_fetch_northbound_flow(params: Dict[str, Any]) -> Any:
    """获取北向资金流向（东财K线接口）"""
    days = params.get("days", 10)

    url = "https://push2his.eastmoney.com/api/qt/kamt.kline/get"
    query_params = {
        "fields1": "f1,f2,f3,f4",
        "fields2": "f51,f52,f53,f54,f55,f56",
        "klt": 101,
        "lmt": days,
        "ut": "b73502e0ed8b4e5f9c5ce1ec0c7c7d86",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("s2n"):
                flows = []
                for line in data["data"]["s2n"]:
                    parts = line.split(",")
                    if len(parts) >= 4:
                        flows.append({
                            "tradeDate": parts[0],
                            "shNetInflow": float(parts[1]) if parts[1] != "-" else 0,
                            "szNetInflow": float(parts[2]) if parts[2] != "-" else 0,
                            "totalNetInflow": float(parts[3]) if parts[3] != "-" else 0,
                        })
                return {"flows": flows}
        except (json.JSONDecodeError, KeyError, ValueError) as e:
            print(f"[WARN] 解析北向资金失败: {e}", file=sys.stderr)

    return {"flows": []}


def handle_fetch_margin_data(params: Dict[str, Any]) -> Any:
    """获取融资融券数据"""
    secid = params.get("secid", "")
    _, code = parse_secid(secid)

    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    query_params = {
        "reportName": "RPT_RZRQ_LSHJ",
        "columns": "ALL",
        "filter": f'(SECURITY_CODE="{code}")',
        "pageNumber": 1,
        "pageSize": 10,
        "sortTypes": -1,
        "sortColumns": "TRADE_DATE",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("result") and data["result"].get("data"):
                return {
                    "marginData": [
                        {
                            "tradeDate": row.get("TRADE_DATE", ""),
                            "financingBuy": row.get("RZRQ_YCLRZYE", 0),
                            "financingBalance": row.get("RZRQ_RZYE", 0),
                            "shortSellVolume": row.get("RZRQ_YCLQYL", 0),
                            "shortBalance": row.get("RZRQ_QYL", 0),
                        }
                        for row in data["result"]["data"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析融资融券失败: {e}", file=sys.stderr)

    return {"marginData": []}


def handle_fetch_stock_rank(params: Dict[str, Any]) -> Any:
    """获取股票排行榜（涨幅/跌幅/换手率/量比）"""
    rank_type = params.get("rank_type", "gain")  # gain/loss/turnover/volume_ratio
    count = params.get("count", 20)

    sort_field = {
        "gain": "f3",
        "loss": "f3",
        "turnover": "f8",
        "volume_ratio": "f10",
    }.get(rank_type, "f3")

    sort_order = -1 if rank_type in ("gain", "loss", "turnover", "volume_ratio") else 1
    if rank_type == "loss":
        sort_order = 1  # 涨跌幅正序=跌幅榜

    url = "https://push2.eastmoney.com/api/qt/clist/get"
    query_params = {
        "pn": 1,
        "pz": count,
        "po": sort_order,
        "np": 1,
        "ut": "b73502e0ed8b4e5f9c5ce1ec0c7c7d86",
        "fltt": 2,
        "invt": 2,
        "fid": sort_field,
        "fs": "m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23",
        "fields": "f2,f3,f4,f8,f10,f12,f14,f20",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("diff"):
                return {
                    "rankType": rank_type,
                    "stocks": [
                        {
                            "code": str(row.get("f12", "")),
                            "name": row.get("f14", ""),
                            "price": row.get("f2", 0),
                            "changePercent": row.get("f3", 0),
                            "turnoverRate": row.get("f8", 0),
                            "volumeRatio": row.get("f10", 0),
                            "marketCap": row.get("f20", 0),
                        }
                        for row in data["data"]["diff"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析排行数据失败: {e}", file=sys.stderr)

    return {"rankType": rank_type, "stocks": []}


def handle_fetch_plate_list(params: Dict[str, Any]) -> Any:
    """获取板块列表（行业板块/概念板块）"""
    plate_type = params.get("plate_type", "industry")  # industry/concept

    fs = "m:90+t:2" if plate_type == "industry" else "m:90+t:3"

    url = "https://push2.eastmoney.com/api/qt/clist/get"
    query_params = {
        "pn": 1,
        "pz": 100,
        "po": 1,
        "np": 1,
        "ut": "b73502e0ed8b4e5f9c5ce1ec0c7c7d86",
        "fltt": 2,
        "invt": 2,
        "fid": "f3",
        "fs": fs,
        "fields": "f2,f3,f4,f8,f12,f14,f20,f104,f105",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("diff"):
                return {
                    "plateType": plate_type,
                    "plates": [
                        {
                            "code": str(row.get("f12", "")),
                            "name": row.get("f14", ""),
                            "changePercent": row.get("f3", 0),
                            "upCount": row.get("f104", 0),
                            "downCount": row.get("f105", 0),
                            "turnoverRate": row.get("f8", 0),
                            "marketCap": row.get("f20", 0),
                        }
                        for row in data["data"]["diff"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析板块数据失败: {e}", file=sys.stderr)

    return {"plateType": plate_type, "plates": []}


def handle_fetch_news(params: Dict[str, Any]) -> Any:
    """获取新闻资讯（东财7x24小时快讯）"""
    count = params.get("count", 20)

    url = "https://np-listapi.eastmoney.com/comm/web/getNewsByColumns"
    query_params = {
        "client": "web",
        "biz": "web_news_col",
        "column": "350",
        "order": 1,
        "needInteractData": 0,
        "page_index": 1,
        "page_size": count,
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("list"):
                return {
                    "news": [
                        {
                            "title": item.get("title", ""),
                            "content": item.get("digest", ""),
                            "url": item.get("url", ""),
                            "source": item.get("source", ""),
                            "publishTime": item.get("showTime", ""),
                        }
                        for item in data["data"]["list"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析新闻失败: {e}", file=sys.stderr)

    return {"news": []}


def handle_fetch_announcement(params: Dict[str, Any]) -> Any:
    """获取公告（巨潮资讯）"""
    secid = params.get("secid", "")
    _, code = parse_secid(secid)
    count = params.get("count", 10)

    # 巨潮资讯公告接口
    url = "http://www.cninfo.com.cn/new/hisAnnouncement/query"
    form_data = {
        "stock": code,
        "tabName": "fulltab",
        "pageSize": count,
        "pageNum": 1,
        "column": "szse",
        "category": "",
        "seDate": "",
        "searchkey": "",
        "sortName": "",
        "sortType": "",
        "isHLtitle": "true",
    }

    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "Referer": "http://www.cninfo.com.cn/new/disclosure",
        "Content-Type": "application/x-www-form-urlencoded",
    }

    try:
        resp = requests.post(url, data=form_data, headers=headers, timeout=TIMEOUT)
        data = resp.json()
        if data.get("announcements"):
            return {
                "announcements": [
                    {
                        "title": ann.get("announcementTitle", ""),
                        "secName": ann.get("secName", ""),
                        "publishDate": ann.get("announcementTime", ""),
                        "url": f"http://static.cninfo.com.cn/{ann.get('adjunctUrl', '')}",
                    }
                    for ann in data["announcements"]
                ]
            }
    except Exception as e:
        print(f"[WARN] 解析公告失败: {e}", file=sys.stderr)

    return {"announcements": []}


def handle_fetch_research_report(params: Dict[str, Any]) -> Any:
    """获取研报"""
    secid = params.get("secid", "")
    _, code = parse_secid(secid)
    count = params.get("count", 10)

    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    query_params = {
        "reportName": "RPT_WEB_RESPAGE",
        "columns": "ALL",
        "filter": f'(SECURITY_CODE="{code}")',
        "pageNumber": 1,
        "pageSize": count,
        "sortTypes": -1,
        "sortColumns": "PUBLISH_DATE",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("result") and data["result"].get("data"):
                return {
                    "reports": [
                        {
                            "title": row.get("TITLE", ""),
                            "institution": row.get("ORG_NAME", ""),
                            "analyst": row.get("RESEARCHER", ""),
                            "publishDate": row.get("PUBLISH_DATE", ""),
                            "rating": row.get("INVEST_RATING_NAME", ""),
                            "targetPrice": row.get("TARGET_PRICE", 0),
                        }
                        for row in data["result"]["data"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析研报失败: {e}", file=sys.stderr)

    return {"reports": []}


def handle_fetch_institutional_holdings(params: Dict[str, Any]) -> Any:
    """获取机构持仓"""
    secid = params.get("secid", "")
    _, code = parse_secid(secid)

    url = "https://datacenter-web.eastmoney.com/api/data/v1/get"
    query_params = {
        "reportName": "RPT_F10_EH_INSTITUTIONAL",
        "columns": "ALL",
        "filter": f'(SECURITY_CODE="{code}")',
        "pageNumber": 1,
        "pageSize": 10,
        "sortTypes": -1,
        "sortColumns": "END_DATE",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("result") and data["result"].get("data"):
                return {
                    "holdings": [
                        {
                            "reportDate": row.get("END_DATE", ""),
                            "institutionType": row.get("INSTITUTION_TYPE", ""),
                            "holdShares": row.get("HOLD_SHARES", 0),
                            "holdMarketCap": row.get("HOLD_MARKET_CAP", 0),
                            "holdRatio": row.get("HOLD_RATIO", 0),
                            "changeShares": row.get("HOLD_SHARES_CHANGE", 0),
                        }
                        for row in data["result"]["data"]
                    ]
                }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析机构持仓失败: {e}", file=sys.stderr)

    return {"holdings": []}


def handle_fetch_main_force_flow(params: Dict[str, Any]) -> Any:
    """获取主力资金流向"""
    secid = params.get("secid", "")
    market, code = parse_secid(secid)

    # 东财主力资金流向
    secid_param = f"{market}.{code}"
    url = f"https://push2.eastmoney.com/api/qt/stock/fflow/kline/get"
    query_params = {
        "secid": secid_param,
        "fields1": "f1,f2,f3,f7",
        "fields2": "f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61",
        "klt": 101,  # 日线
        "lmt": 10,
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("klines"):
                flows = []
                for line in data["data"]["klines"]:
                    parts = line.split(",")
                    if len(parts) >= 6:
                        flows.append({
                            "date": parts[0],
                            "mainNetInflow": float(parts[1]) if parts[1] != "-" else 0,
                            "smallNetInflow": float(parts[2]) if parts[2] != "-" else 0,
                            "mediumNetInflow": float(parts[3]) if parts[3] != "-" else 0,
                            "largeNetInflow": float(parts[4]) if parts[4] != "-" else 0,
                            "superLargeNetInflow": float(parts[5]) if parts[5] != "-" else 0,
                        })
                return {"flows": flows}
        except (json.JSONDecodeError, KeyError, ValueError) as e:
            print(f"[WARN] 解析主力资金失败: {e}", file=sys.stderr)

    return {"flows": []}


def handle_fetch_sector_rotation(params: Dict[str, Any]) -> Any:
    """获取板块轮动（5日/10日涨跌幅排名）"""
    url = "https://push2.eastmoney.com/api/qt/clist/get"
    query_params = {
        "pn": 1,
        "pz": 50,
        "po": 1,
        "np": 1,
        "ut": "b73502e0ed8b4e5f9c5ce1ec0c7c7d86",
        "fltt": 2,
        "invt": 2,
        "fid": "f3",
        "fs": "m:90+t:2",
        "fields": "f2,f3,f4,f8,f12,f14,f62",
    }

    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("diff"):
                sectors = [
                    {
                        "code": str(row.get("f12", "")),
                        "name": row.get("f14", ""),
                        "changePercent": row.get("f3", 0),
                        "mainNetInflow": row.get("f62", 0),
                        "turnoverRate": row.get("f8", 0),
                    }
                    for row in data["data"]["diff"]
                ]
                return {"sectors": sectors}
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析板块轮动失败: {e}", file=sys.stderr)

    return {"sectors": []}


def handle_fetch_market_sentiment(params: Dict[str, Any]) -> Any:
    """获取市场情绪指标（涨跌家数/涨停跌停/北向资金）"""
    # 涨跌家数
    url = "https://push2.eastmoney.com/api/qt/ulist.np/get"
    query_params = {
        "fltt": 2,
        "fields": "f1,f2,f3,f4,f6,f12,f13,f104,f105,f106",
        "secids": "1.000001,0.399001,0.399006",
    }

    indices = []
    resp = safe_get(url, params=query_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("diff"):
                for row in data["data"]["diff"]:
                    indices.append({
                        "code": str(row.get("f12", "")),
                        "name": {1: "上证指数", 0: "深证成指"}.get(row.get("f13", -1), "创业板指"),
                        "price": row.get("f2", 0),
                        "changePercent": row.get("f3", 0),
                        "upCount": row.get("f104", 0),
                        "downCount": row.get("f105", 0),
                        "flatCount": row.get("f106", 0),
                    })
        except (json.JSONDecodeError, KeyError) as e:
            print(f"[WARN] 解析市场情绪失败: {e}", file=sys.stderr)

    # 北向资金概览
    nb_url = "https://push2his.eastmoney.com/api/qt/kamt.kline/get"
    nb_params = {
        "fields1": "f1,f2,f3,f4",
        "fields2": "f51,f52,f53,f54,f55,f56",
        "klt": 101,
        "lmt": 1,
        "ut": "b73502e0ed8b4e5f9c5ce1ec0c7c7d86",
    }
    northbound = None
    resp = safe_get(nb_url, params=nb_params)
    if resp:
        try:
            data = resp.json()
            if data.get("data") and data["data"].get("s2n"):
                line = data["data"]["s2n"][0]
                parts = line.split(",")
                if len(parts) >= 4:
                    northbound = {
                        "date": parts[0],
                        "shNetInflow": float(parts[1]) if parts[1] != "-" else 0,
                        "szNetInflow": float(parts[2]) if parts[2] != "-" else 0,
                        "totalNetInflow": float(parts[3]) if parts[3] != "-" else 0,
                    }
        except Exception:
            pass

    return {
        "indices": indices,
        "northbound": northbound,
    }


# ==================== Action 路由 ====================


def handle_action(action: str, params: Dict[str, Any]) -> Any:
    """根据 action 路由到对应处理器"""
    handlers = {
        "fetch_financial_report": handle_fetch_financial_report,
        "fetch_shareholder_count": handle_fetch_shareholder_count,
        "fetch_lockup_schedule": handle_fetch_lockup_schedule,
        "fetch_industry_pe": handle_fetch_industry_pe,
        "fetch_dragon_tiger": handle_fetch_dragon_tiger,
        "fetch_northbound_flow": handle_fetch_northbound_flow,
        "fetch_margin_data": handle_fetch_margin_data,
        "fetch_stock_rank": handle_fetch_stock_rank,
        "fetch_plate_list": handle_fetch_plate_list,
        "fetch_news": handle_fetch_news,
        "fetch_announcement": handle_fetch_announcement,
        "fetch_research_report": handle_fetch_research_report,
        "fetch_institutional_holdings": handle_fetch_institutional_holdings,
        "fetch_main_force_flow": handle_fetch_main_force_flow,
        "fetch_sector_rotation": handle_fetch_sector_rotation,
        "fetch_market_sentiment": handle_fetch_market_sentiment,
    }

    handler = handlers.get(action)
    if handler is None:
        raise ValueError(f"未知 action: {action}")

    return handler(params)


# ==================== IPC 主循环 ====================


def main():
    """stdin/stdout JSON IPC 主循环"""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            request = json.loads(line)
            request_id = request.get("id", 0)
            action = request.get("action", "")
            params = request.get("params", {})

            result = handle_action(action, params)

            response = {
                "id": request_id,
                "success": True,
                "data": result,
            }
        except Exception as e:
            response = {
                "id": request.get("id", 0) if "request" in dir() else 0,
                "success": False,
                "error": f"{type(e).__name__}: {str(e)}\n{traceback.format_exc()}",
            }

        print(json.dumps(response, ensure_ascii=False, default=str), flush=True)


if __name__ == "__main__":
    main()
