use anyhow::{Context, Result};
use encoding_rs::GBK;

use crate::models::{KLineData, StockQuote};

const REALTIME_URL: &str = "http://hq.sinajs.cn/list=";
const KLINE_URL: &str = "http://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData";

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
    let end = text.rfind('"').context("行情数据格式错误: 未找到结束引号")?;

    if start >= end {
        anyhow::bail!("行情数据为空，可能是无效的股票代码: {}", symbol);
    }

    let data = &text[start..end];
    let fields: Vec<&str> = data.split(',').collect();

    if fields.len() < 32 {
        anyhow::bail!(
            "行情数据字段不足: 期望至少32个字段，实际{}个",
            fields.len()
        );
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

/// 获取K线数据
pub fn fetch_kline_data(symbol: &str, scale: u32, datalen: u32) -> Result<Vec<KLineData>> {
    let url = format!(
        "{}?symbol={}&scale={}&ma=no&datalen={}",
        KLINE_URL, symbol, scale, datalen
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

/// 批量获取多只股票实时行情
pub fn fetch_multiple_quotes(symbols: &[String]) -> Vec<Result<StockQuote>> {
    symbols.iter().map(|s| fetch_realtime_quote(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_realtime_quote() {
        let raw = r#"var hq_str_sh600519="贵州茅台,1731.500,1732.000,1755.000,1760.000,1728.000,1754.980,1755.000,25432100,44539876543.000,100,1754.980,200,1754.970,300,1754.960,400,1754.950,500,1754.940,100,1755.000,200,1755.010,300,1755.020,400,1755.030,500,1755.040,2025-02-11,15:00:00,00,"; "#;
        let result = parse_realtime_quote("sh600519", raw);
        assert!(result.is_ok());
        let quote = result.unwrap();
        assert_eq!(quote.name, "贵州茅台");
        assert_eq!(quote.open, 1731.5);
        assert_eq!(quote.current, 1755.0);
        assert_eq!(quote.high, 1760.0);
        assert_eq!(quote.low, 1728.0);
    }

    #[test]
    fn test_parse_empty_quote() {
        let raw = r#"var hq_str_sh000000="";"#;
        let result = parse_realtime_quote("sh000000", raw);
        assert!(result.is_err());
    }
}
