# Rime 字数统计工具 (rime-word-counter)

为 [Rime 输入法](https://rime.im/) 开发的每日字数统计与可视化工具。提供 **桌面应用**（默认）、
**命令行处理模式**（用于定时任务）和 **终端统计摘要**。

## 架构

```
 Lua 脚本（嵌入 Rime）              Rust 核心库（src/）               前端
┌──────────────────┐            ┌───────────────────────┐      ┌──────────────────┐
│  word_counter    │  追加      │  log_processor        │      │ rime-word-counter│
│  每次上屏记录     │───────────►│  读取 → 按日累加       │      │  命令行 (cron)   │
│  YYYY-MM-DD,字数 │  CSV 日志  │         ↓             │      │  --process       │
└──────────────────┘            │  SQLite daily_words   │      │  --stats         │
                                │         ↓             │      └──────────────────┘
                                │  stats 聚合统计        │      ┌──────────────────┐
                                │  （唯一数据出口）       │─────►│ rime-word-counter│
                                └───────────────────────┘      │ -gui  Tauri 桌面端│
                                                               │  React + Vite    │
                                                               └──────────────────┘
```

**两个前端共用同一个核心库**，统计口径完全一致。

### 工作流程

1. **Lua 脚本** 常驻 Rime，每次上屏时记录 `YYYY-MM-DD,字数` 到 CSV 日志。
2. **桌面端启动时** 自动处理日志 → 按日期汇总 → 存入 SQLite → 渲染趋势图。
3. 界面里可筛选日期范围、切换分组（日/月/年）、切换语言。

## 功能特性

- 🖥️ **桌面应用** — Tauri 2 + React 19，Linear 风格深色界面
- 🌍 **多语言** — 简体中文、繁体中文、English（自动检测 + 手动切换）
- 📊 **趋势图表** — 手写 SVG 柱状 + 折线叠加，hover 查看单日明细
- 📅 **灵活筛选** — 自定义日期范围，快捷选择（最近 7 天 / 30 天 / 一年 / 全部）
- 📈 **分组聚合** — 按日 / 月 / 年查看
- 🔄 **一键重新处理** — 读取日志并即时反馈处理了多少行
- ⏰ **定时任务** — `--process` 模式可在 crontab 中运行
- 📟 **终端摘要** — `--stats` 不启动图形界面也能看统计
- 🧩 **浏览器演示模式** — `npm run dev` 直接开前端调试 UI（本地假数据）

## 文件结构

```
rime-word-counter/
├── Cargo.toml                工作区根：核心库 + CLI
├── src/                      核心库 + 命令行
│   ├── main.rs               CLI 入口（--process / --stats）
│   ├── lib.rs                核心库导出
│   ├── db.rs                 SQLite 封装
│   ├── log_processor.rs      日志读取、汇总、截断
│   ├── stats.rs              仪表盘数据（趋势点 + 概览统计）
│   ├── paths.rs              默认路径解析
│   └── i18n.rs               系统语言探测
├── src-tauri/                Tauri 桌面端（Rust 侧）
├── ui/                       React + Vite + TypeScript 界面
├── lua/word_counter.lua      Rime 的 Lua 日志脚本
├── scripts/generate_icon.py  生成应用图标源图
├── AGENTS.md                 开发/协作上手文档（权威）
├── CLAUDE.md                 速查要点
└── README.md                 本文件
```

## 部署步骤

### 1. 部署 Lua 脚本到 Rime

1. 将 `lua/word_counter.lua` 复制到 Rime 用户目录下的 `lua/` 文件夹：
   - **Linux (fcitx5)**: `~/.local/share/fcitx5/rime/lua/`
   - **Linux (ibus)**: `~/.config/ibus/rime/lua/`
   - **macOS**: `~/Library/Rime/lua/`
   - **Windows**: `%APPDATA%\Rime\lua\`

2. 在你的输入方案配置文件下（比如雾凇拼音 `rime_ice.custom.yaml`）中加入：

```yaml
patch:
  engine/processors/+:
    - lua_processor@*word_counter

```

3. 重新部署 Rime（通常按 `Ctrl+Option+~` 或右键托盘图标选「重新部署」）。

### 2. 准备环境

```bash
# Rust（https://rustup.rs/）与 Node.js 20+
# Linux 编译桌面端还需要 WebKitGTK：
sudo pacman -S webkit2gtk-4.1                    # Arch
sudo apt install libwebkit2gtk-4.1-dev           # Debian / Ubuntu
```

### 3. 构建与运行

```bash
npm install              # 安装前端依赖（首次）

# 桌面应用
npm run tauri dev        # 开发模式
npm run tauri build      # 打包安装包（target/release/bundle/）

# 命令行（不需要 webkit2gtk）
cargo build --release
./target/release/rime-word-counter --process     # 处理日志（cron）
./target/release/rime-word-counter --stats       # 终端看统计
```

### 4. 自动化运行

**Linux (crontab) — 每 30 分钟处理一次日志：**

```bash
crontab -e
# 添加：
*/30 * * * * /path/to/rime-word-counter --process
```

需要看图表时，启动桌面应用即可。

## 命令行参数

| 参数 | 说明 |
|---|---|
| `--process` | 处理日志文件（读取 → 汇总 → 清空），适用于定时任务 |
| `--stats` | 在终端打印统计摘要 |
| `--log-path <PATH>` | CSV 日志文件路径（默认 `~/.cache/rime-word-counter/rime_word.log`） |
| `--db-path <PATH>` | SQLite 数据库路径（默认 `~/.cache/rime-word-counter/rime_stats.db`） |
| `--help` | 显示帮助信息 |

桌面端 `rime-word-counter-gui` 同样支持 `--log-path` / `--db-path`。

## 界面说明

- **左侧栏**：时间范围（输入 + 快捷按钮）、数据源路径、重新处理按钮
- **顶部**：当前范围、覆盖天数与数据点数量、总字数、语言切换
- **概览卡片**：总字数 / 范围内字数 / 日均 / 覆盖天数
- **趋势图**：柱状 + 折线，右上角切换日 / 月 / 年分组，鼠标悬停查看明细
- **明细记录**：按时间倒序，含占比条，可展开全部

### 语言切换

点击右上角语言下拉菜单，可在简体中文、繁体中文、English 之间切换，界面文本实时更新。
首次启动的语言由系统 `LANG` / `LC_ALL` 决定。

## 数据存储

- **日志文件** (`rime_word.log`)：CSV 格式 `YYYY-MM-DD,count`，无表头。
  处理时按日期累加写入数据库，然后**截断**（不删除，避免 Lua 侧句柄失效）。
- **数据库** (`rime_stats.db`)：SQLite，表结构：

  ```sql
  CREATE TABLE daily_words (
      date       TEXT PRIMARY KEY,   -- YYYY-MM-DD
      word_count INTEGER NOT NULL DEFAULT 0
  );
  ```

  同一天多次导入会累加，不会覆盖历史数据。

## 开发

```bash
cargo test            # 29 个单元测试
npm run typecheck     # 前端类型检查
npm run dev           # 浏览器里调 UI（演示数据）
```

详细架构、模块职责、设计令牌与踩坑记录见 **[AGENTS.md](./AGENTS.md)**。

## 许可证

MIT

字体：[Inter](https://github.com/rsms/inter)（SIL OFL-1.1，通过 `@fontsource-variable/inter` 分发）。
