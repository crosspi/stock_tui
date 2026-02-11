use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Context as CanvasContext, Line as CanvasLine},
        Block, Borders, Clear, List, ListItem, Paragraph,
    },
    Frame,
};

use crate::app::App;
use crate::models::*;

/// 涨的颜色（红色）
const COLOR_UP: Color = Color::Red;
/// 跌的颜色（绿色）
const COLOR_DOWN: Color = Color::Green;
/// 平的颜色
const COLOR_FLAT: Color = Color::White;
/// 游标颜色
const COLOR_CURSOR: Color = Color::Yellow;

/// 均线颜色
const COLOR_MA5: Color = Color::White;
const COLOR_MA10: Color = Color::Yellow;
const COLOR_MA20: Color = Color::Magenta;

/// 主渲染函数
pub fn draw(f: &mut Frame, app: &App) {
    match app.view_mode {
        ViewMode::Normal => draw_normal_layout(f, app),
        ViewMode::FullscreenChart => draw_fullscreen_chart(f, app),
    }

    // 如果在输入模式，绘制输入弹窗（两种视图下都可用）
    if app.input_mode == InputMode::AddStock {
        draw_input_popup(f, app);
    }

    // 快捷键帮助弹窗
    if app.input_mode == InputMode::HelpScreen {
        draw_help_popup(f, app);
    }
}

/// 正常布局
fn draw_normal_layout(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // 行情概览
            Constraint::Min(12),   // K线图
            Constraint::Length(8), // 自选股列表
            Constraint::Length(1), // 状态栏
        ])
        .split(f.area());

    draw_quote_info(f, app, chunks[0]);
    draw_kline_chart(f, app, chunks[1]);
    draw_watchlist(f, app, chunks[2]);
    draw_status_bar(f, app, chunks[3]);
}

/// 全屏K线图布局
fn draw_fullscreen_chart(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 精简行情信息
            Constraint::Min(10),   // K线图（占满）
            Constraint::Length(1), // 状态栏
        ])
        .split(f.area());

    // 精简行情头部
    draw_compact_quote(f, app, chunks[0]);
    draw_kline_chart(f, app, chunks[1]);
    draw_fullscreen_status(f, app, chunks[2]);
}

/// 精简行情信息（全屏模式用）
fn draw_compact_quote(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(quote) = app.current_quote() {
        let change = quote.change();
        let change_pct = quote.change_percent();
        let color = if change > 0.0 {
            COLOR_UP
        } else if change < 0.0 {
            COLOR_DOWN
        } else {
            COLOR_FLAT
        };
        let sign = if change > 0.0 { "+" } else { "" };

        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", quote.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}]", quote.symbol),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:.2}", quote.current),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{}{:.2} ({}{:.2}%)", sign, change, sign, change_pct),
                Style::default().fg(color),
            ),
            Span::raw("    "),
            Span::styled(
                format!(
                    "高:{:.2} 低:{:.2} 量:{}",
                    quote.high,
                    quote.low,
                    quote.volume_display()
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        let p = Paragraph::new(line).block(block);
        f.render_widget(p, area);
    } else {
        f.render_widget(Paragraph::new(" 加载中...").block(block), area);
    }
}

/// 全屏模式状态栏
fn draw_fullscreen_status(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(" ?", Style::default().fg(Color::Yellow)),
        Span::styled(" 快捷键", Style::default().fg(Color::DarkGray)),
    ];

    // 显示当前周期
    spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(
        app.timeframe.label(),
        Style::default().fg(Color::Cyan),
    ));

    // 如果有游标数据，显示在状态栏
    if let Some(kline) = app.cursor_kline(area.width as usize) {
        spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            format!(
                "{} 开:{} 高:{} 低:{} 收:{} 量:{}",
                kline.day, kline.open, kline.high, kline.low, kline.close, kline.volume
            ),
            Style::default().fg(Color::White),
        ));
    }

    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}

/// 绘制行情概览
fn draw_quote_info(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" 📈 股票行情 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(quote) = app.current_quote() {
        let change = quote.change();
        let change_pct = quote.change_percent();
        let color = if change > 0.0 {
            COLOR_UP
        } else if change < 0.0 {
            COLOR_DOWN
        } else {
            COLOR_FLAT
        };

        let sign = if change > 0.0 { "+" } else { "" };

        let lines = vec![
            Line::from(vec![
                Span::styled(
                    format!(" {} ", quote.name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}]", quote.symbol),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:.2}", quote.current),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{}{:.2} ({}{:.2}%)", sign, change, sign, change_pct),
                    Style::default().fg(color),
                ),
            ]),
            Line::from(vec![
                Span::styled(" 开盘: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2}", quote.open),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  "),
                Span::styled("最高: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.2}", quote.high), Style::default().fg(COLOR_UP)),
                Span::raw("  "),
                Span::styled("最低: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:.2}", quote.low), Style::default().fg(COLOR_DOWN)),
                Span::raw("  "),
                Span::styled("昨收: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2}", quote.pre_close),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(" 成交量: ", Style::default().fg(Color::DarkGray)),
                Span::styled(quote.volume_display(), Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled("成交额: ", Style::default().fg(Color::DarkGray)),
                Span::styled(quote.turnover_display(), Style::default().fg(Color::Cyan)),
                Span::raw("  "),
                Span::styled(
                    format!("{} {}", quote.date, quote.time),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(" 加载中...")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
    }
}

/// 绘制K线蜡烛图（带游标支持 + 坐标轴 + 均线）
fn draw_kline_chart(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.kline_cursor.is_some() {
        format!(" K线图 - {} [游标模式] ", app.timeframe.label())
    } else {
        format!(" K线图 - {} ", app.timeframe.label())
    };

    let outer_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if app.kline_data.is_empty() {
        let paragraph = Paragraph::new(" 无K线数据")
            .block(outer_block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(paragraph, area);
        return;
    }

    // 先渲染外框
    f.render_widget(outer_block, area);

    // 内部区域（去掉外框边框）
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.width < 15 || inner.height < 5 {
        return;
    }

    // 布局：[价格轴(10列)] [K线图画布]
    //                       [日期轴(1行)]
    let price_axis_width: u16 = 10;
    let date_axis_height: u16 = 1;
    let chart_width = inner.width.saturating_sub(price_axis_width);
    let chart_height = inner.height.saturating_sub(date_axis_height);

    let price_axis_area = Rect {
        x: inner.x,
        y: inner.y,
        width: price_axis_width,
        height: chart_height,
    };
    let chart_area = Rect {
        x: inner.x + price_axis_width,
        y: inner.y,
        width: chart_width,
        height: chart_height,
    };
    let date_axis_area = Rect {
        x: inner.x + price_axis_width,
        y: inner.y + chart_height,
        width: chart_width,
        height: date_axis_height,
    };

    // 计算均线数据 (全局计算)
    let ma5 = calculate_ma(&app.kline_data, 5);
    let ma10 = calculate_ma(&app.kline_data, 10);
    let ma20 = calculate_ma(&app.kline_data, 20);

    // 计算可显示的K线数量（每根蜡烛占3列宽度）
    let candle_width = 3usize;
    let visible_count = (chart_width as usize / candle_width).min(app.kline_data.len());

    // 根据偏移量截取可见的K线数据
    let start_idx = if app.kline_data.len() > visible_count + app.kline_offset {
        app.kline_data.len() - visible_count - app.kline_offset
    } else {
        0
    };
    let end_idx = (start_idx + visible_count).min(app.kline_data.len());
    let visible_data = &app.kline_data[start_idx..end_idx];

    if visible_data.is_empty() {
        return;
    }

    // 计算价格范围 (包含K线和均线)
    let mut min_price = f64::MAX;
    let mut max_price = f64::MIN;
    for (i, k) in visible_data.iter().enumerate() {
        min_price = min_price.min(k.low_f64());
        max_price = max_price.max(k.high_f64());

        // 考虑均线范围
        let global_idx = start_idx + i;
        if let Some(v) = ma5.get(global_idx).and_then(|&v| v) {
            min_price = min_price.min(v);
            max_price = max_price.max(v);
        }
        if let Some(v) = ma10.get(global_idx).and_then(|&v| v) {
            min_price = min_price.min(v);
            max_price = max_price.max(v);
        }
        if let Some(v) = ma20.get(global_idx).and_then(|&v| v) {
            min_price = min_price.min(v);
            max_price = max_price.max(v);
        }
    }

    let price_range = max_price - min_price;
    let margin = price_range * 0.05;
    min_price -= margin;
    max_price += margin;
    let final_range = max_price - min_price;

    // 计算网格线的价格级别
    let num_grid_lines = (chart_height as usize).min(6).max(2);
    let grid_step = (chart_height as usize) / num_grid_lines.max(1);
    let mut grid_prices: Vec<f64> = Vec::new();
    for i in 0..chart_height {
        if grid_step == 0 || (i as usize) % grid_step == 0 || i == chart_height - 1 {
            let ratio = 1.0 - (i as f64 / (chart_height.saturating_sub(1).max(1)) as f64);
            grid_prices.push(min_price + final_range * ratio);
        }
    }

    // ── 绘制K线蜡烛图 + 网格线 + 均线 ──
    let canvas_w = (visible_data.len() * candle_width) as f64;
    let cursor_pos = app.kline_cursor;
    let grid_prices_clone = grid_prices.clone();

    // Clone MA data for closure (efficient enough for TUI)
    // Actually we can move them if we don't need them outside.
    // We need them for cursor info later, so let's use a reference or clone needed parts?
    // Rust closures and borrowing... we can't easily capture slices if they reference `app`.
    // But `ma5` is a local Vec, so we can clone it.
    let ma5_clone = ma5.clone();
    let ma10_clone = ma10.clone();
    let ma20_clone = ma20.clone();

    let canvas = Canvas::default()
        .x_bounds([0.0, canvas_w])
        .y_bounds([min_price, max_price])
        .marker(symbols::Marker::Braille)
        .paint(move |ctx: &mut CanvasContext| {
            // 先绘制网格线（最底层）
            for &gp in &grid_prices_clone {
                let grid_steps = (canvas_w as usize) / 2;
                for gs in 0..grid_steps {
                    let gx = (gs * 2) as f64 + 0.5;
                    ctx.print(
                        gx,
                        gp,
                        ratatui::text::Line::from(Span::styled(
                            "┈",
                            Style::default().fg(Color::Indexed(236)),
                        )),
                    );
                }
            }

            // 绘制均线 (Line chart)
            // Draw lines between adjacent points
            for i in 1..visible_data.len() {
                let x_prev = ((i - 1) * candle_width) as f64 + 1.0;
                let x_curr = (i * candle_width) as f64 + 1.0;
                let global_prev = start_idx + i - 1;
                let global_curr = start_idx + i;

                if let (Some(prev), Some(curr)) = (
                    ma5_clone.get(global_prev).and_then(|&v| v),
                    ma5_clone.get(global_curr).and_then(|&v| v),
                ) {
                    ctx.draw(&CanvasLine::new(x_prev, prev, x_curr, curr, COLOR_MA5));
                }
                if let (Some(prev), Some(curr)) = (
                    ma10_clone.get(global_prev).and_then(|&v| v),
                    ma10_clone.get(global_curr).and_then(|&v| v),
                ) {
                    ctx.draw(&CanvasLine::new(x_prev, prev, x_curr, curr, COLOR_MA10));
                }
                if let (Some(prev), Some(curr)) = (
                    ma20_clone.get(global_prev).and_then(|&v| v),
                    ma20_clone.get(global_curr).and_then(|&v| v),
                ) {
                    ctx.draw(&CanvasLine::new(x_prev, prev, x_curr, curr, COLOR_MA20));
                }
            }

            // 绘制蜡烛（逐行连续绘制，避免断裂）
            let inner_h = chart_area.height as f64;
            let row_step = if inner_h > 0.0 {
                final_range / inner_h
            } else {
                1.0
            };

            for (i, kline) in visible_data.iter().enumerate() {
                let x = (i * candle_width) as f64 + 1.0;
                let open = kline.open_f64();
                let close = kline.close_f64();
                let high = kline.high_f64();
                let low = kline.low_f64();

                let is_cursor = cursor_pos == Some(i);
                let base_color = if close >= open { COLOR_UP } else { COLOR_DOWN };
                let color = if is_cursor { COLOR_CURSOR } else { base_color };

                let body_top = open.max(close);
                let body_bottom = open.min(close);
                let body_char = if is_cursor { "▓" } else { "█" };

                if row_step <= 0.0 || final_range <= 0.0 {
                    // 无法计算步长，画一个点
                    ctx.print(
                        x,
                        close,
                        ratatui::text::Line::from(Span::styled("─", Style::default().fg(color))),
                    );
                    continue;
                }

                // 从 low 到 high 逐行绘制
                let mut y = low;
                while y <= high + row_step * 0.5 {
                    let ch = if y >= body_bottom - row_step * 0.5 && y <= body_top + row_step * 0.5
                    {
                        body_char
                    } else {
                        "│"
                    };
                    ctx.print(
                        x,
                        y,
                        ratatui::text::Line::from(Span::styled(ch, Style::default().fg(color))),
                    );
                    y += row_step;
                }

                // 补充端点
                ctx.print(
                    x,
                    low,
                    ratatui::text::Line::from(Span::styled(
                        if low >= body_bottom - row_step * 0.5 {
                            body_char
                        } else {
                            "│"
                        },
                        Style::default().fg(color),
                    )),
                );
                ctx.print(
                    x,
                    high,
                    ratatui::text::Line::from(Span::styled(
                        if high <= body_top + row_step * 0.5 {
                            body_char
                        } else {
                            "│"
                        },
                        Style::default().fg(color),
                    )),
                );
            }
        });

    f.render_widget(canvas, chart_area);

    // ── 绘制价格Y轴（左侧） ──
    let mut price_lines: Vec<Line> = Vec::new();
    for i in 0..chart_height {
        let ratio = 1.0 - (i as f64 / (chart_height.saturating_sub(1).max(1)) as f64);
        let price_val = min_price + final_range * ratio;
        if grid_step == 0 || (i as usize) % grid_step == 0 || i == chart_height - 1 {
            price_lines.push(Line::from(Span::styled(
                format!("{:>9.2}", price_val),
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            price_lines.push(Line::from(Span::styled("          ", Style::default())));
        }
    }
    let price_axis = Paragraph::new(price_lines);
    f.render_widget(price_axis, price_axis_area);

    // ── 绘制日期X轴（底部） ──
    let mut date_str = String::new();
    let date_interval = visible_data.len() / 5_usize.max(1);
    let date_interval = date_interval.max(1);
    for (i, kline) in visible_data.iter().enumerate() {
        if i % date_interval == 0 || i == visible_data.len() - 1 {
            // 截取日期的月-日部分
            let label = if kline.day.len() >= 10 {
                &kline.day[5..10] // MM-DD
            } else {
                &kline.day
            };
            // 填充到蜡烛宽度
            let padded = format!("{:<width$}", label, width = candle_width);
            date_str.push_str(&padded);
        } else {
            for _ in 0..candle_width {
                date_str.push(' ');
            }
        }
    }
    // 截断到区域宽度
    let display_date: String = date_str.chars().take(chart_width as usize).collect();
    let date_line = Paragraph::new(Line::from(Span::styled(
        display_date,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(date_line, date_axis_area);

    // ── 绘制游标信息覆盖层 ──
    if let Some(cursor_idx) = app.kline_cursor {
        if let Some(kline) = visible_data.get(cursor_idx) {
            let color = if kline.close_f64() >= kline.open_f64() {
                COLOR_UP
            } else {
                COLOR_DOWN
            };

            // 获取当前游标位置的均线值
            let global_idx = start_idx + cursor_idx;
            let ma5_val = ma5.get(global_idx).and_then(|v| *v);
            let ma10_val = ma10.get(global_idx).and_then(|v| *v);
            let ma20_val = ma20.get(global_idx).and_then(|v| *v);

            let mut info_spans = vec![
                Span::styled(" ▸ ", Style::default().fg(COLOR_CURSOR)),
                Span::styled(
                    format!("{} ", kline.day),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("开:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} ", kline.open_f64()),
                    Style::default().fg(color),
                ),
                Span::styled("高:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} ", kline.high_f64()),
                    Style::default().fg(COLOR_UP),
                ),
                Span::styled("低:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} ", kline.low_f64()),
                    Style::default().fg(COLOR_DOWN),
                ),
                Span::styled("收:", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:.2} ", kline.close_f64()),
                    Style::default().fg(color),
                ),
            ];

            // 添加均线信息
            if let Some(v) = ma5_val {
                info_spans.push(Span::styled("MA5:", Style::default().fg(COLOR_MA5)));
                info_spans.push(Span::styled(
                    format!("{:.2} ", v),
                    Style::default().fg(COLOR_MA5),
                ));
            }
            if let Some(v) = ma10_val {
                info_spans.push(Span::styled("MA10:", Style::default().fg(COLOR_MA10)));
                info_spans.push(Span::styled(
                    format!("{:.2} ", v),
                    Style::default().fg(COLOR_MA10),
                ));
            }
            if let Some(v) = ma20_val {
                info_spans.push(Span::styled("MA20:", Style::default().fg(COLOR_MA20)));
                info_spans.push(Span::styled(
                    format!("{:.2} ", v),
                    Style::default().fg(COLOR_MA20),
                ));
            }

            let info_line = Line::from(info_spans);

            let overlay_area = Rect {
                x: chart_area.x,
                y: chart_area.y,
                width: chart_area.width,
                height: 1,
            };
            let overlay = Paragraph::new(info_line).style(Style::default().bg(Color::Black));
            f.render_widget(overlay, overlay_area);
        }
    }
}

/// 绘制自选股列表
fn draw_watchlist(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .watchlist
        .iter()
        .enumerate()
        .map(|(i, symbol)| {
            let quote = app.quotes.get(i).and_then(|q| q.as_ref());

            let (name, price, change_str, color) = if let Some(q) = quote {
                let change_pct = q.change_percent();
                let sign = if change_pct > 0.0 { "+" } else { "" };
                let color = if change_pct > 0.0 {
                    COLOR_UP
                } else if change_pct < 0.0 {
                    COLOR_DOWN
                } else {
                    COLOR_FLAT
                };
                (
                    q.name.clone(),
                    format!("{:.2}", q.current),
                    format!("{}{:.2}%", sign, change_pct),
                    color,
                )
            } else {
                (
                    "加载中...".to_string(),
                    "--".to_string(),
                    "--".to_string(),
                    Color::DarkGray,
                )
            };

            let prefix = if i == app.selected_index {
                "▶ "
            } else {
                "  "
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:<10} ", symbol), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<8} ", name), Style::default().fg(Color::White)),
                Span::styled(format!("{:>10} ", price), Style::default().fg(color)),
                Span::styled(format!("{:>8}", change_str), Style::default().fg(color)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" 自选股 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

/// 绘制底部状态栏
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.status_message, Style::default().fg(Color::DarkGray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.timeframe.label(), Style::default().fg(Color::Cyan)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::styled(" 快捷键", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(status, area);
}

/// 绘制添加股票的输入弹窗
fn draw_input_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 5, f.area());
    f.render_widget(Clear, area);

    let input = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(Color::Yellow)),
        Span::styled(
            &app.input_buffer,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("█", Style::default().fg(Color::Yellow)),
    ]))
    .block(
        Block::default()
            .title(" 添加股票 (输入代码如 sh600519) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(input, area);
}

/// 绘制快捷键帮助弹窗
fn draw_help_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let timeframes = [
        ("1", "5分钟", TimeFrame::Min5),
        ("2", "15分钟", TimeFrame::Min15),
        ("3", "30分钟", TimeFrame::Min30),
        ("4", "60分钟", TimeFrame::Min60),
        ("5", "日K", TimeFrame::Daily),
        ("6", "周K", TimeFrame::Weekly),
        ("7", "月K", TimeFrame::Monthly),
    ];

    // 构建周期行
    let mut tf_spans: Vec<Span> = vec![Span::styled("  ", Style::default())];
    for (key, label, tf) in &timeframes {
        let is_active = app.timeframe == *tf;
        tf_spans.push(Span::styled(
            format!(" {} ", key),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        let style = if is_active {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        tf_spans.push(Span::styled(format!("{} ", label), style));
        if is_active {
            tf_spans.push(Span::styled("◀ ", Style::default().fg(Color::Cyan)));
        }
    }

    let help_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ── 基本操作 ──",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  q       ", Style::default().fg(Color::Yellow)),
            Span::styled("退出程序", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  r       ", Style::default().fg(Color::Yellow)),
            Span::styled("刷新数据", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  f/Enter ", Style::default().fg(Color::Yellow)),
            Span::styled("切换全屏K线", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ── 自选股 ──",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ↑/k ↓/j ", Style::default().fg(Color::Yellow)),
            Span::styled("选择自选股", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  a       ", Style::default().fg(Color::Yellow)),
            Span::styled("添加股票", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  d       ", Style::default().fg(Color::Yellow)),
            Span::styled("删除股票", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ── K线操作 ──",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ←/h →/l ", Style::default().fg(Color::Yellow)),
            Span::styled("移动游标", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/Dn ", Style::default().fg(Color::Yellow)),
            Span::styled("滚动K线", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc     ", Style::default().fg(Color::Yellow)),
            Span::styled("取消游标 / 退出全屏", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ── K线周期 ──",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(tf_spans),
        Line::from(""),
        Line::from(vec![Span::styled(
            "        按 Esc / ? / q 关闭",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .title(" ⌨ 快捷键 ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(help, area);
}

/// 创建居中矩形
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
