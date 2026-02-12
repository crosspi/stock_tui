use anyhow::{Context, Result};
use encoding_rs::GBK;
use serde_json::Value;

use crate::models::{KLineData, StockQuote};

const REALTIME_URL: &str = "http://hq.sinajs.cn/list=";
const KLINE_URL_CN: &str =
    "http://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData";
const KLINE_URL_US: &str =
    "http://stock.finance.sina.com.cn/usstock/api/jsonp.php/IO/US_MinKService.getDailyK";

/// 从新浪财经获取实时行情
pub fn fetch_realtime_quote(symbol: &str) -> Result<StockQuote> {
    let url = format!("{}{}", REALTIME_URL, symbol);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .header("Referer", "http://finance.sina.com.cn")
        .send()
        .context("请求实时行情失败")?;

    let bytes = resp.bytes().context("读取响应数据失败")?;

    // 新浪实时行情接口返回 GBK 编码
    let (decoded, _, _) = GBK.decode(&bytes);
    let text = decoded.to_string();

    parse_realtime_quote(symbol, &text)
}

/// 解析实时行情数据
/// 格式: var hq_str_sh600519="贵州茅台,1731.50,...";
fn parse_realtime_quote(symbol: &str, text: &str) -> Result<StockQuote> {
    // 提取引号内的数据
    let start = text.find('"').context("行情数据格式错误: 未找到引号")? + 1;
    let end = text
        .rfind('"')
        .context("行情数据格式错误: 未找到结束引号")?;

    if start >= end {
        anyhow::bail!("行情数据为空，可能是无效的股票代码: {}", symbol);
    }

    let data = &text[start..end];
    let fields: Vec<&str> = data.split(',').collect();

    if symbol.starts_with("hk") {
        parse_hk_quote(symbol, &fields)
    } else if symbol.starts_with("gb_") {
        parse_us_quote(symbol, &fields)
    } else {
        parse_cn_quote(symbol, &fields)
    }
}

fn parse_cn_quote(symbol: &str, fields: &[&str]) -> Result<StockQuote> {
    if fields.len() < 32 {
        anyhow::bail!("A股行情数据字段不足: 期望32+，实际{}", fields.len());
    }
    Ok(StockQuote {
        name: fields[0].to_string(),
        symbol: symbol.to_string(),
        open: fields[1].parse().unwrap_or(0.0),
        pre_close: fields[2].parse().unwrap_or(0.0),
        current: fields[3].parse().unwrap_or(0.0),
        high: fields[4].parse().unwrap_or(0.0),
        low: fields[5].parse().unwrap_or(0.0),
        volume: fields[8].parse().unwrap_or(0.0),
        turnover: fields[9].parse().unwrap_or(0.0),
        date: fields[30].to_string(),
        time: fields[31].to_string(),
    })
}

fn parse_hk_quote(symbol: &str, fields: &[&str]) -> Result<StockQuote> {
    if fields.len() < 19 {
        anyhow::bail!("港股行情数据字段不足: 期望19+，实际{}", fields.len());
    }
    // hk00700="TENCENT,腾讯控股,543.000,551.000,550.500,543.000,548.000,..."
    Ok(StockQuote {
        name: fields[1].to_string(), // 中文名
        symbol: symbol.to_string(),
        open: fields[2].parse().unwrap_or(0.0),
        pre_close: fields[3].parse().unwrap_or(0.0),
        high: fields[4].parse().unwrap_or(0.0),
        low: fields[5].parse().unwrap_or(0.0),
        current: fields[6].parse().unwrap_or(0.0),
        // fields[12] is volume, fields[11] is turnover
        volume: fields[12].parse().unwrap_or(0.0),
        turnover: fields[11].parse().unwrap_or(0.0),
        date: fields[17].replace('/', "-"), // 2026/02/11 -> 2026-02-11
        time: fields[18].to_string(),
    })
}

fn parse_us_quote(symbol: &str, fields: &[&str]) -> Result<StockQuote> {
    if fields.len() < 27 {
        anyhow::bail!("美股行情数据字段不足: 期望27+，实际{}", fields.len());
    }
    // gb_aapl="苹果,276.0800,0.88,2026-02-12 04:17:52,..."
    let datetime = fields[3];
    let (date, time) = if let Some((d, t)) = datetime.split_once(' ') {
        (d.to_string(), t.to_string())
    } else {
        (datetime.to_string(), "".to_string())
    };

    Ok(StockQuote {
        name: fields[0].to_string(),
        symbol: symbol.to_string(),
        current: fields[1].parse().unwrap_or(0.0),
        // fields[2] is pct change
        open: fields[5].parse().unwrap_or(0.0),
        high: fields[6].parse().unwrap_or(0.0),
        low: fields[7].parse().unwrap_or(0.0),
        volume: fields[10].parse().unwrap_or(0.0),
        turnover: 0.0, // 美股接口通常不返回成交额
        pre_close: fields[26].parse().unwrap_or(0.0),
        date,
        time,
    })
}

/// 获取K线数据
pub fn fetch_kline_data(symbol: &str, scale: u32, datalen: u32) -> Result<Vec<KLineData>> {
    if symbol.starts_with("gb_") {
        return fetch_us_kline(symbol, scale, datalen);
    } else if symbol.starts_with("hk") {
        // 暂时不支持港股K线，返回空列表以免报错
        return Ok(Vec::new());
    }

    let url = format!(
        "{}?symbol={}&scale={}&ma=no&datalen={}",
        KLINE_URL_CN, symbol, scale, datalen
    );

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .header("Referer", "http://finance.sina.com.cn")
        .send()
        .context("请求K线数据失败")?;

    let text = resp.text().context("读取K线数据失败")?;
    let klines: Vec<KLineData> = serde_json::from_str(&text).context("解析K线 JSON 失败")?;

    Ok(klines)
}

fn fetch_us_kline(symbol: &str, _scale: u32, _datalen: u32) -> Result<Vec<KLineData>> {
    // gb_aapl -> aapl
    let raw_symbol = symbol.trim_start_matches("gb_");
    let url = format!("{}?symbol={}", KLINE_URL_US, raw_symbol);

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&url)
        .header("Referer", "http://finance.sina.com.cn")
        .send()
        .context("请求美股K线数据失败")?;

    let text = resp.text().context("读取美股K线数据失败")?;

    // 解析 JSONP: IO({...}) 或 IO([...])
    let start_idx = text.find("IO(").context("解析美股K线失败: 未找到 IO(")? + 3;
    let json_str = text[start_idx..]
        .trim()
        .trim_end_matches(");")
        .trim_end_matches(")"); // 容错

    let json_val: Value = serde_json::from_str(json_str).context("解析美股K线 JSONP 失败")?;

    let mut klines = Vec::new();

    if let Some(arr) = json_val.as_array() {
        for item in arr {
            // 格式: {"d":"2020-01-01","o":"100.0","h":"105.0","l":"99.0","c":"102.0","v":"10000"}
            if let (Some(d), Some(o), Some(h), Some(l), Some(c), Some(v)) = (
                item.get("d").and_then(|v| v.as_str()),
                item.get("o").and_then(|v| v.as_str()),
                item.get("h").and_then(|v| v.as_str()),
                item.get("l").and_then(|v| v.as_str()),
                item.get("c").and_then(|v| v.as_str()),
                item.get("v").and_then(|v| v.as_str()),
            ) {
                // 美股日期可能带时间 "2026-01-23 16:00:00"，截取日期部分
                let day = d.split_whitespace().next().unwrap_or(d).to_string();

                klines.push(KLineData {
                    day,
                    open: o.to_string(),
                    high: h.to_string(),
                    low: l.to_string(),
                    close: c.to_string(),
                    volume: v.to_string(),
                });
            }
        }
    }

    Ok(klines)
}

/// 批量获取多只股票实时行情
pub fn fetch_multiple_quotes(symbols: &[String]) -> Vec<Result<StockQuote>> {
    symbols.iter().map(|s| fetch_realtime_quote(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cn_quote() {
        let raw = r#"var hq_str_sh600519="贵州茅台,1731.500,1732.000,1755.000,1760.000,1728.000,1754.980,1755.000,25432100,44539876543.000,100,1754.980,200,1754.970,300,1754.960,400,1754.950,500,1754.940,100,1755.000,200,1755.010,300,1755.020,400,1755.030,500,1755.040,2025-02-11,15:00:00,00,";"#;
        let q = parse_realtime_quote("sh600519", raw).unwrap();
        assert_eq!(q.name, "贵州茅台");
        assert_eq!(q.current, 1755.0);
    }

    #[test]
    fn test_parse_hk_quote() {
        let raw = r#"var hq_str_hk00700="TENCENT,腾讯控股,543.000,551.000,550.500,543.000,548.000,-3.000,-0.544,547.50000,548.00000,12991880860,23759058,0.000,0.000,683.000,415.374,2026/02/11,16:08";"#;
        let q = parse_realtime_quote("hk00700", raw).unwrap();
        assert_eq!(q.name, "腾讯控股");
        assert_eq!(q.open, 543.0);
        assert_eq!(q.pre_close, 551.0);
        assert_eq!(q.current, 548.0);
        assert_eq!(q.date, "2026-02-11");
    }

    #[test]
    fn test_parse_us_quote() {
        let raw = r#"var hq_str_gb_aapl="苹果,276.0800,0.88,2026-02-12 04:17:52,2.4000,274.6950,280.1800,274.4500,288.6200,168.4300,37329226,61226827,4053169131200,7.93,34.810000,0.00,0.00,0.26,0.00,14681140000,63,0.0000,0.00,0.00,,Feb 11 03:17PM EST,273.6800,0,1,2026,10353387124.0000,0.0000,0.0000,0.0000,0.0000,273.6800";"#;
        let q = parse_realtime_quote("gb_aapl", raw).unwrap();
        assert_eq!(q.name, "苹果");
        assert_eq!(q.current, 276.08);
        assert_eq!(q.open, 274.695);
        assert_eq!(q.pre_close, 273.68);
        assert_eq!(q.date, "2026-02-12");
        assert_eq!(q.time, "04:17:52");
    }
}
