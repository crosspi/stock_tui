use crate::api;
use crate::config::Config;
use crate::models::*;
use ratatui::widgets::TableState;

/// 应用主状态
pub struct App {
    /// 是否退出
    pub should_quit: bool,
    /// 自选股列表（股票代码）
    pub watchlist: Vec<String>,
    /// 自选股列表状态（用于滚动）
    pub watchlist_state: TableState,
    /// 当前激活显示的股票索引（用于详情和K线显示）
    pub active_index: usize,
    /// 各股票的实时行情缓存
    pub quotes: Vec<Option<StockQuote>>,
    /// 当前股票的K线数据
    pub kline_data: Vec<KLineData>,
    /// 当前K线周期
    pub timeframe: TimeFrame,
    /// 输入模式
    pub input_mode: InputMode,
    /// 视图模式（正常 / 全屏K线）
    pub view_mode: ViewMode,
    /// 股票代码输入缓冲区
    pub input_buffer: String,
    /// K线图水平滚动偏移（从右往左）
    pub kline_offset: usize,
    /// K线游标位置（在可见K线中的索引，None表示未激活）
    pub kline_cursor: Option<usize>,
    /// 状态栏消息
    pub status_message: String,
    /// 是否正在加载
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        // Load config from file
        let config = Config::load();
        let watchlist = config.watchlist;

        let quotes = vec![None; watchlist.len()];
        let mut watchlist_state = TableState::default();
        if !watchlist.is_empty() {
            watchlist_state.select(Some(0));
        }

        let mut app = Self {
            should_quit: false,
            watchlist,
            watchlist_state,
            active_index: 0,
            quotes,
            kline_data: Vec::new(),
            timeframe: TimeFrame::Daily,
            input_mode: InputMode::Normal,
            view_mode: ViewMode::Normal,
            input_buffer: String::new(),
            kline_offset: 0,
            kline_cursor: None,
            status_message: "正在加载数据...".to_string(),
            loading: true,
        };

        app.refresh_all();
        app
    }

    /// 获取当前列表高亮索引
    pub fn highlighted_index(&self) -> usize {
        self.watchlist_state.selected().unwrap_or(0)
    }

    /// 刷新所有数据
    pub fn refresh_all(&mut self) {
        self.refresh_quotes();
        self.refresh_kline();
        self.loading = false;
    }

    /// 刷新所有股票的实时行情
    pub fn refresh_quotes(&mut self) {
        if self.watchlist.is_empty() {
            self.quotes.clear();
            return;
        }

        let results = api::fetch_multiple_quotes(&self.watchlist);
        self.quotes = results
            .into_iter()
            .map(|r| match r {
                Ok(q) => Some(q),
                Err(e) => {
                    self.status_message = format!("获取行情失败: {}", e);
                    None
                }
            })
            .collect();

        // 更新状态消息
        if let Some(Some(q)) = self.quotes.get(self.active_index) {
            self.status_message =
                format!("{} {} 最后更新: {} {}", q.symbol, q.name, q.date, q.time);
        }
    }

    /// 刷新当前选中股票的K线数据
    pub fn refresh_kline(&mut self) {
        if let Some(symbol) = self.watchlist.get(self.active_index) {
            match api::fetch_kline_data(symbol, self.timeframe.scale(), 120) {
                Ok(data) => {
                    self.kline_data = data;
                    self.kline_offset = 0;
                    self.kline_cursor = None;
                }
                Err(e) => {
                    self.status_message = format!("获取K线数据失败: {}", e);
                    self.kline_data.clear();
                }
            }
        } else {
            self.kline_data.clear();
        }
    }

    /// 获取当前激活股票的行情
    pub fn current_quote(&self) -> Option<&StockQuote> {
        self.quotes.get(self.active_index).and_then(|q| q.as_ref())
    }

    /// 上移选中
    pub fn select_prev(&mut self) {
        let i = match self.watchlist_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.watchlist.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.watchlist_state.select(Some(i));
    }

    /// 下移选中
    pub fn select_next(&mut self) {
        let i = match self.watchlist_state.selected() {
            Some(i) => {
                if i >= self.watchlist.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.watchlist_state.select(Some(i));
    }

    /// 处理Enter键：激活选中股票 或 切换全屏
    pub fn on_enter(&mut self) {
        let highlighted = self.highlighted_index();
        if highlighted != self.active_index {
            self.active_index = highlighted;
            self.status_message = "正在加载...".to_string();
            self.refresh_kline();
            // Optional: refresh quotes too, or just wait for next tick
            // self.refresh_quotes();
        } else {
            self.toggle_fullscreen();
        }
    }

    /// 切换K线周期
    pub fn set_timeframe(&mut self, tf: TimeFrame) {
        if self.timeframe != tf {
            self.timeframe = tf;
            self.refresh_kline();
        }
    }

    /// K线图左滚
    pub fn scroll_kline_left(&mut self) {
        if self.kline_offset + 10 < self.kline_data.len() {
            self.kline_offset += 5;
        }
        self.kline_cursor = None;
    }

    /// K线图右滚
    pub fn scroll_kline_right(&mut self) {
        if self.kline_offset >= 5 {
            self.kline_offset -= 5;
        } else {
            self.kline_offset = 0;
        }
        self.kline_cursor = None;
    }

    /// 切换全屏K线模式
    pub fn toggle_fullscreen(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Normal => ViewMode::FullscreenChart,
            ViewMode::FullscreenChart => ViewMode::Normal,
        };
    }

    /// 获取当前可见K线数量（用于游标边界检查）
    pub fn visible_kline_count(&self, chart_width: usize) -> usize {
        let candle_width = 3;
        // 去掉外框边框(2) + 左侧价格轴(10)
        let inner_width = chart_width.saturating_sub(12);
        (inner_width / candle_width).min(self.kline_data.len())
    }

    /// K线游标左移
    pub fn cursor_left(&mut self, max_visible: usize) {
        match self.kline_cursor {
            Some(pos) => {
                if pos > 0 {
                    self.kline_cursor = Some(pos - 1);
                }
            }
            None => {
                // 首次激活游标，放在最右边（最新K线）
                if !self.kline_data.is_empty() {
                    let last = max_visible.min(self.kline_data.len()).saturating_sub(1);
                    self.kline_cursor = Some(last);
                }
            }
        }
    }

    /// K线游标右移
    pub fn cursor_right(&mut self, max_visible: usize) {
        match self.kline_cursor {
            Some(pos) => {
                let limit = max_visible.min(self.kline_data.len()).saturating_sub(1);
                if pos < limit {
                    self.kline_cursor = Some(pos + 1);
                }
            }
            None => {
                // 首次激活游标，放在最右边（最新K线）
                if !self.kline_data.is_empty() {
                    let last = max_visible.min(self.kline_data.len()).saturating_sub(1);
                    self.kline_cursor = Some(last);
                }
            }
        }
    }

    /// 获取游标指向的K线数据
    pub fn cursor_kline(&self, chart_width: usize) -> Option<&KLineData> {
        let cursor_pos = self.kline_cursor?;
        let candle_width = 3;
        let inner_width = chart_width.saturating_sub(2);
        let visible_count = (inner_width / candle_width).min(self.kline_data.len());

        let start_idx = if self.kline_data.len() > visible_count + self.kline_offset {
            self.kline_data.len() - visible_count - self.kline_offset
        } else {
            0
        };

        self.kline_data.get(start_idx + cursor_pos)
    }

    /// 进入添加股票模式
    pub fn start_add_stock(&mut self) {
        self.input_mode = InputMode::AddStock;
        self.input_buffer.clear();
        self.status_message = "输入代码 (sh600519/hk00700/gb_aapl)，Enter确认，Esc取消".to_string();
    }

    /// 确认添加股票
    pub fn confirm_add_stock(&mut self) {
        let mut symbol = self.input_buffer.trim().to_lowercase();
        if symbol.is_empty() {
            self.status_message = "股票代码不能为空".to_string();
            self.input_mode = InputMode::Normal;
            return;
        }

        // 处理美股 us 前缀转 gb_
        if symbol.starts_with("us") {
            symbol = symbol.replacen("us", "gb_", 1);
        }

        // 检查格式: sh, sz, bj, hk, gb_
        if !symbol.starts_with("sh")
            && !symbol.starts_with("sz")
            && !symbol.starts_with("bj")
            && !symbol.starts_with("hk")
            && !symbol.starts_with("gb_")
        {
            self.status_message =
                "格式错误，支持前缀: sh/sz/bj(A股/北交), hk(港股), gb_/us(美股)".to_string();
            self.input_mode = InputMode::Normal;
            return;
        }

        // 检查重复
        if self.watchlist.contains(&symbol) {
            self.status_message = format!("{} 已在自选股列表中", symbol);
            self.input_mode = InputMode::Normal;
            return;
        }

        self.watchlist.push(symbol.clone());
        self.quotes.push(None);

        // 获取新股票行情
        match api::fetch_realtime_quote(&symbol) {
            Ok(q) => {
                self.status_message = format!("已添加: {} {}", q.symbol, q.name);
                let idx = self.quotes.len() - 1;
                self.quotes[idx] = Some(q);
                self.save_config();
            }
            Err(e) => {
                self.status_message = format!("添加成功但获取行情失败: {}", e);
                // 即使行情获取失败也保存，因为已经添加到watchlist了
                self.save_config();
            }
        }

        self.input_mode = InputMode::Normal;
    }

    /// 取消输入
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.status_message = "已取消".to_string();
    }

    /// 删除当前选中的股票
    pub fn delete_selected(&mut self) {
        if self.watchlist.len() <= 1 {
            self.status_message = "至少保留一只自选股".to_string();
            return;
        }

        let idx = self.highlighted_index();
        let removed = self.watchlist.remove(idx);
        self.quotes.remove(idx);
        self.status_message = format!("已删除: {}", removed);

        // 更新选中状态
        if idx >= self.watchlist.len() {
            self.watchlist_state.select(Some(self.watchlist.len() - 1));
        } else {
            self.watchlist_state.select(Some(idx));
        }

        // 删除后，强制激活当前高亮的股票
        self.active_index = self.highlighted_index();

        self.save_config();
        self.refresh_kline();
    }
    fn save_config(&mut self) {
        let config = Config {
            watchlist: self.watchlist.clone(),
        };
        if let Err(e) = config.save() {
            self.status_message = format!("配置保存失败: {}", e);
        }
    }
}
