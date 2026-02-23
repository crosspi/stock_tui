# Stock TUI (Rust Learning Project)

[中文说明](#chinese-readme) | [English](#english-readme)

<a name="english-readme"></a>
## English Description

**Stock TUI** is a terminal-based application for tracking stock market data, specifically designed for China A-shares. It serves as a practical project for learning Rust programming, demonstrating how to build a robust TUI application with network capabilities.

This project leverages the **Sina Finance API** to fetch real-time quotes and historical K-line data.

### Features

*   **Real-time Quotes**: View the latest prices, changes, and volume for stocks in your watchlist.
*   **K-Line Charts**: Visualize stock performance with candlestick charts.
    *   Supports multiple timeframes: 5min, 15min, 30min, 60min, Daily, Weekly, Monthly.
    *   Technical Indicators: Displays Moving Averages (MA5, MA10, MA20, MA30).
*   **Watchlist Management**: Add and remove stocks (e.g., `sh600519`) easily.
*   **Interactive TUI**: Navigate using keyboard shortcuts, support for resizing, and fullscreen charts.
*   **Cross-Platform**: Runs on Linux, macOS, and Windows.

### Tech Stack

*   **Language**: [Rust](https://www.rust-lang.org/)
*   **TUI Framework**: [Ratatui](https://github.com/ratatui-org/ratatui)
*   **Terminal Backend**: [Crossterm](https://github.com/crossterm-rs/crossterm)
*   **HTTP Client**: [Reqwest](https://github.com/seanmonstar/reqwest) (Blocking)
*   **Serialization**: [Serde](https://serde.rs/) & [Serde JSON](https://github.com/serde-rs/json)
*   **Encoding**: [encoding_rs](https://github.com/hsivonen/encoding_rs) (for handling GBK data from Sina API)

### Installation

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/crosspi/stock-tui.git
cd stock-tui
cargo run --release
```

### Usage / Keybindings

| Key | Action |
| :--- | :--- |
| `q` / `Ctrl+c` | Quit application |
| `Esc` | Cancel cursor / Exit fullscreen / Quit |
| `f` | Toggle chart fullscreen mode |
| `Enter` | Select stock / Confirm add |
| `Up` / `k` | Select previous stock |
| `Down` / `j` | Select next stock |
| `Left` / `h` | Move K-line cursor left |
| `Right` / `l` | Move K-line cursor right |
| `PageUp` | Scroll K-line chart left |
| `PageDown` | Scroll K-line chart right |
| `a` | Add stock (input mode) |
| `d` | Delete selected stock |
| `r` | Refresh data |
| `1` - `7` | Switch timeframe (5m, 15m, 30m, 60m, Daily, Weekly, Monthly) |
| `?` | Show Help screen |

### License

This project is licensed under the [MIT License](LICENSE).

---

<a name="chinese-readme"></a>
## 中文说明 (Chinese Description)

**Stock TUI** 是一个基于终端的股票行情查看工具，专为查看中国 A 股市场数据设计。这也是一个用于学习 Rust 编程的实战项目，展示了如何构建一个包含网络请求功能的 TUI 应用。

本项目封装了 **新浪财经 (Sina Finance) API**，用于获取实时行情和历史 K 线数据。

### 功能特性

*   **实时行情**：查看自选股的最新价格、涨跌幅和成交量。
*   **K 线图表**：以蜡烛图形式可视化股票走势。
    *   支持多种周期：5分钟、15分钟、30分钟、60分钟、日K、周K、月K。
    *   技术指标：显示移动平均线 (MA5, MA10, MA20, MA30)。
*   **自选股管理**：轻松添加和删除股票（例如输入 `sh600519`）。
*   **交互式界面**：全键盘操作，支持窗口缩放和全屏图表模式。
*   **跨平台**：可在 Linux, macOS, 和 Windows 上运行。

### 技术栈

*   **编程语言**: [Rust](https://www.rust-lang.org/)
*   **TUI 框架**: [Ratatui](https://github.com/ratatui-org/ratatui)
*   **终端后端**: [Crossterm](https://github.com/crossterm-rs/crossterm)
*   **HTTP 客户端**: [Reqwest](https://github.com/seanmonstar/reqwest) (同步模式)
*   **序列化**: [Serde](https://serde.rs/) & [Serde JSON](https://github.com/serde-rs/json)
*   **字符编码**: [encoding_rs](https://github.com/hsivonen/encoding_rs) (处理新浪 API 的 GBK 编码数据)

### 安装与运行

确保你已经安装了 Rust 和 Cargo 环境。

```bash
git clone https://github.com/crosspi/stock-tui.git
cd stock-tui
cargo run --release
```

### 快捷键说明

| 按键 | 功能 |
| :--- | :--- |
| `q` / `Ctrl+c` | 退出程序 |
| `Esc` | 取消游标 / 退出全屏 / 退出程序 |
| `f` | 切换图表全屏模式 |
| `Enter` | 确认选择 / 确认添加股票 |
| `Up` / `k` | 选择上一个股票 |
| `Down` / `j` | 选择下一个股票 |
| `Left` / `h` | K线图游标左移 |
| `Right` / `l` | K线图游标右移 |
| `PageUp` | K线图向左滚动 |
| `PageDown` | K线图向右滚动 |
| `a` | 添加股票 (进入输入模式) |
| `d` | 删除选中股票 |
| `r` | 手动刷新数据 |
| `1` - `7` | 切换周期 (5分, 15分, 30分, 60分, 日K, 周K, 月K) |
| `?` | 显示帮助页面 |

### 许可证

本项目采用 [MIT License](LICENSE) 许可证。
