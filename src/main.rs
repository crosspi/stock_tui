mod api;
mod app;
mod config;
mod event;
mod models;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;
use event::{AppEvent, EventHandler};
use models::{InputMode, TimeFrame, ViewMode};

fn main() -> Result<()> {
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用
    let mut app = App::new();

    // 创建事件处理器（每5秒自动刷新）
    let events = EventHandler::new(Duration::from_secs(5));

    // 主循环
    loop {
        // 获取终端宽度用于游标边界计算
        let term_width = terminal.size()?.width as usize;

        // 渲染
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // 处理事件
        match events.next()? {
            AppEvent::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.input_mode {
                    InputMode::Normal => {
                        // 计算当前K线图的可见数量
                        let visible = app.visible_kline_count(term_width);

                        match key.code {
                            // 退出
                            KeyCode::Char('q') => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.should_quit = true;
                            }
                            // Esc: 全屏模式退出全屏 / 有游标则取消游标 / 否则退出程序
                            KeyCode::Esc => {
                                if app.kline_cursor.is_some() {
                                    app.kline_cursor = None;
                                } else if app.view_mode == ViewMode::FullscreenChart {
                                    app.toggle_fullscreen();
                                } else {
                                    app.should_quit = true;
                                }
                            }
                            // 全屏切换
                            KeyCode::Char('f') => {
                                app.toggle_fullscreen();
                            }
                            KeyCode::Enter => {
                                app.on_enter();
                            }
                            // 自选股上下选择（仅在非全屏时）
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.view_mode == ViewMode::Normal {
                                    app.select_prev();
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.view_mode == ViewMode::Normal {
                                    app.select_next();
                                }
                            }
                            // 左右：移动K线游标
                            KeyCode::Left | KeyCode::Char('h') => {
                                app.cursor_left(visible);
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.cursor_right(visible);
                            }
                            // Page Up/Down: 滚动K线图
                            KeyCode::PageUp => app.scroll_kline_left(),
                            KeyCode::PageDown => app.scroll_kline_right(),
                            // 添加/删除股票
                            KeyCode::Char('a') => app.start_add_stock(),
                            KeyCode::Char('d') => {
                                if app.view_mode == ViewMode::Normal {
                                    app.delete_selected();
                                }
                            }
                            // 手动刷新
                            KeyCode::Char('r') => {
                                app.status_message = "正在刷新...".to_string();
                                app.refresh_all();
                            }
                            // 周期切换 1-7
                            KeyCode::Char('1') => app.set_timeframe(TimeFrame::Min5),
                            KeyCode::Char('2') => app.set_timeframe(TimeFrame::Min15),
                            KeyCode::Char('3') => app.set_timeframe(TimeFrame::Min30),
                            KeyCode::Char('4') => app.set_timeframe(TimeFrame::Min60),
                            KeyCode::Char('5') => app.set_timeframe(TimeFrame::Daily),
                            KeyCode::Char('6') => app.set_timeframe(TimeFrame::Weekly),
                            KeyCode::Char('7') => app.set_timeframe(TimeFrame::Monthly),
                            // 帮助页面
                            KeyCode::Char('?') => {
                                app.input_mode = InputMode::HelpScreen;
                            }
                            _ => {}
                        }
                    }
                    InputMode::AddStock => match key.code {
                        KeyCode::Enter => app.confirm_add_stock(),
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    InputMode::HelpScreen => match key.code {
                        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                            app.input_mode = InputMode::Normal;
                        }
                        // 在帮助页面也可以直接切换周期
                        KeyCode::Char('1') => {
                            app.set_timeframe(TimeFrame::Min5);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('2') => {
                            app.set_timeframe(TimeFrame::Min15);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('3') => {
                            app.set_timeframe(TimeFrame::Min30);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('4') => {
                            app.set_timeframe(TimeFrame::Min60);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('5') => {
                            app.set_timeframe(TimeFrame::Daily);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('6') => {
                            app.set_timeframe(TimeFrame::Weekly);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('7') => {
                            app.set_timeframe(TimeFrame::Monthly);
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
            AppEvent::Tick => {
                // 自动刷新行情
                app.refresh_quotes();
            }
            AppEvent::Resize(_, _) => {
                // 终端大小变化会自动重绘
            }
        }

        if app.should_quit {
            break;
        }
    }

    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
