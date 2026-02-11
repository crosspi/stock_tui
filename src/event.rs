use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// 应用事件
pub enum AppEvent {
    /// 键盘事件
    Key(KeyEvent),
    /// 定时 Tick（用于自动刷新数据）
    Tick,
    /// 终端大小改变
    Resize(u16, u16),
}

/// 事件处理器
pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    _tx: mpsc::Sender<AppEvent>,
}

impl EventHandler {
    /// 创建事件处理器，tick_rate 为自动刷新间隔
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();

        thread::spawn(move || {
            loop {
                // 等待事件，超时则发送 Tick
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        let app_event = match evt {
                            CrosstermEvent::Key(key) => AppEvent::Key(key),
                            CrosstermEvent::Resize(w, h) => AppEvent::Resize(w, h),
                            _ => continue,
                        };
                        if event_tx.send(app_event).is_err() {
                            break;
                        }
                    }
                } else {
                    // 超时，发送 Tick 触发数据刷新
                    if event_tx.send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// 接收下一个事件
    pub fn next(&self) -> Result<AppEvent> {
        let event = self.rx.recv()?;
        Ok(event)
    }
}
