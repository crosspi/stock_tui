use serde::Deserialize;

/// 实时行情数据
#[derive(Debug, Clone)]
pub struct StockQuote {
    /// 股票名称
    pub name: String,
    /// 股票代码 (如 sh600519)
    pub symbol: String,
    /// 今日开盘价
    pub open: f64,
    /// 昨日收盘价
    pub pre_close: f64,
    /// 当前价格
    pub current: f64,
    /// 今日最高价
    pub high: f64,
    /// 今日最低价
    pub low: f64,
    /// 成交量（股）
    pub volume: f64,
    /// 成交金额（元）
    pub turnover: f64,
    /// 日期
    pub date: String,
    /// 时间
    pub time: String,
}

impl StockQuote {
    /// 涨跌额
    pub fn change(&self) -> f64 {
        self.current - self.pre_close
    }

    /// 涨跌幅（百分比）
    pub fn change_percent(&self) -> f64 {
        if self.pre_close == 0.0 {
            0.0
        } else {
            (self.current - self.pre_close) / self.pre_close * 100.0
        }
    }

    /// 格式化成交量（万手）
    pub fn volume_display(&self) -> String {
        let lots = self.volume / 100.0; // 股 -> 手
        if lots >= 10000.0 {
            format!("{:.1}万手", lots / 10000.0)
        } else {
            format!("{:.0}手", lots)
        }
    }

    /// 格式化成交额（亿/万）
    pub fn turnover_display(&self) -> String {
        if self.turnover >= 1_0000_0000.0 {
            format!("{:.2}亿", self.turnover / 1_0000_0000.0)
        } else if self.turnover >= 10000.0 {
            format!("{:.1}万", self.turnover / 10000.0)
        } else {
            format!("{:.0}元", self.turnover)
        }
    }
}

/// K线数据（从新浪财经 JSON API 返回）
#[derive(Debug, Clone, Deserialize)]
pub struct KLineData {
    pub day: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

impl KLineData {
    pub fn open_f64(&self) -> f64 {
        self.open.parse().unwrap_or(0.0)
    }
    pub fn high_f64(&self) -> f64 {
        self.high.parse().unwrap_or(0.0)
    }
    pub fn low_f64(&self) -> f64 {
        self.low.parse().unwrap_or(0.0)
    }
    pub fn close_f64(&self) -> f64 {
        self.close.parse().unwrap_or(0.0)
    }
    pub fn volume_f64(&self) -> f64 {
        self.volume.parse().unwrap_or(0.0)
    }

    /// 是否为阳线（收盘价 >= 开盘价）
    pub fn is_bullish(&self) -> bool {
        self.close_f64() >= self.open_f64()
    }
}

/// K线周期
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFrame {
    Min5,
    Min15,
    Min30,
    Min60,
    Daily,
    Weekly,
    Monthly,
}

impl TimeFrame {
    /// 返回新浪 API 的 scale 参数
    pub fn scale(&self) -> u32 {
        match self {
            TimeFrame::Min5 => 5,
            TimeFrame::Min15 => 15,
            TimeFrame::Min30 => 30,
            TimeFrame::Min60 => 60,
            TimeFrame::Daily => 240,
            TimeFrame::Weekly => 1200,
            TimeFrame::Monthly => 7200,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimeFrame::Min5 => "5分钟",
            TimeFrame::Min15 => "15分钟",
            TimeFrame::Min30 => "30分钟",
            TimeFrame::Min60 => "60分钟",
            TimeFrame::Daily => "日K",
            TimeFrame::Weekly => "周K",
            TimeFrame::Monthly => "月K",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            TimeFrame::Min5 => "5m",
            TimeFrame::Min15 => "15m",
            TimeFrame::Min30 => "30m",
            TimeFrame::Min60 => "60m",
            TimeFrame::Daily => "日K",
            TimeFrame::Weekly => "周K",
            TimeFrame::Monthly => "月K",
        }
    }

    pub fn all() -> &'static [TimeFrame] {
        &[
            TimeFrame::Min5,
            TimeFrame::Min15,
            TimeFrame::Min30,
            TimeFrame::Min60,
            TimeFrame::Daily,
            TimeFrame::Weekly,
            TimeFrame::Monthly,
        ]
    }
}

/// 输入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// 正常浏览模式
    Normal,
    /// 输入股票代码模式
    AddStock,
}

/// 视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// 正常布局（行情 + K线 + 自选股）
    Normal,
    /// 全屏K线图
    FullscreenChart,
}
