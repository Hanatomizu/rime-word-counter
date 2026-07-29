# Rime 字数统计工具 (rime-word-counter)

为 [Rime 输入法](https://rime.im/) 开发的每日字数统计与可视化工具。提供 **图形界面**（默认）和 **CLI 处理模式**（用于定时任务）。

## 架构

```
Lua 脚本（嵌入 Rime）         Rust 程序
┌─────────────────┐         ┌──────────────────────────┐
│  Rime 输入法     │ 追加    │  rime-word-counter        │
│  word_counter    │───────→ │    ┌──────────────────┐  │
│  (processor)     │  CSV    │    │  log_processor   │  │
│                   │ 日志    │    │  读取 & 汇总     │  │
│  每次上屏 →       │         │    └──────┬───────────┘  │
│  记录日期,字数    │         │           ↓              │
└─────────────────┘         │    ┌──────────────┐      │
                            │    │  SQLite 数据库│      │
                            │    │ daily_words   │      │
                            │    └──────┬───────┘      │
                            │           ↓              │
                            │    ┌──────────────────┐  │
                            │    │  GUI (egui)      │  │
                            │    │  ├ 筛选面板      │  │
                            │    │  ├ 图表显示      │  │
                            │    │  └ 多语言支持    │  │
                            │    └──────────────────┘  │
                            └──────────────────────────┘
```

### 工作流程

1. **Lua 脚本** 常驻 Rime 输入法，每次上屏时记录 `YYYY-MM-DD,字数` 到 CSV 日志。
2. **GUI 启动时** 自动处理日志 → 按日期汇总 → 存入 SQLite → 显示图表。
3. 用户可在 GUI 中筛选日期范围、切换分组（日/月/年）、切换语言。

## 功能特性

- 🖥️ **图形界面** — 基于 egui 的原生窗口，无需浏览器
- 🌍 **多语言** — 简体中文、繁体中文、English（自动检测 + 手动切换）
- 📊 **交互图表** — 柱状图 + 折线叠加，数值标签标注
- 📅 **灵活筛选** — 自定义日期范围，快捷选择（最近7天/30天/一年/全部）
- 📈 **分组聚合** — 按日/月/年查看趋势
- 🔄 **自动处理** — 启动时自动处理日志，支持手动"重新处理"
- ⏰ **定时任务** — `--process` 模式可在 crontab 中运行

## 文件结构

```
rime-word-counter/
├── Cargo.toml               # Rust 项目配置
├── src/
│   ├── main.rs              # 入口，CLI 参数解析
│   ├── db.rs                # SQLite 操作封装
│   ├── log_processor.rs     # 日志处理、汇总、清空
│   ├── visualizer.rs        # plotters 图表渲染（内存输出）
│   ├── gui.rs               # egui 图形界面
│   └── i18n.rs              # 多语言（简中/繁中/英文）
├── lua/
│   └── word_counter.lua     # Rime 的 Lua 日志脚本
├── README.md                # 本文件
└── .gitignore
```

## 部署步骤

### 1. 部署 Lua 脚本到 Rime

1. 将 `lua/word_counter.lua` 复制到 Rime 用户目录下的 `lua/` 文件夹：
   - **Linux (fcitx5)**: `~/.local/share/fcitx5/rime/lua/`
   - **Linux (ibus)**: `~/.config/ibus/rime/lua/`
   - **macOS**: `~/Library/Rime/lua/`
   - **Windows**: `%APPDATA%\Rime\lua\`

2. 在 Rime 用户目录下创建/编辑 `rime.lua`，添加：
   ```lua
   word_counter_processor = require("word_counter")
   ```

3. 在使用的输入方案中 `engine/processors` 列表末尾添加 `word_counter_processor`：
   ```yaml
   schema:
     schema_id: luna_pinyin
   engine:
     processors:
       - ascii_composer
       - recognizer
       - key_binder
       - speller
       - punctuator
       - selector
       - navigator
       - express_editor
       - word_counter_processor    # ← 添加这一行
   ```

4. 重新部署 Rime（通常按 `Ctrl+Option+~` 或右键托盘图标选「重新部署」）。

### 2. 编译 Rust 程序

```bash
# 确保已安装 Rust（https://rustup.rs/）
cargo build --release

# 编译产物在 target/release/rime-word-counter
```

### 3. 运行

```bash
# 启动 GUI（默认模式，自动处理日志）
./target/release/rime-word-counter

# 仅处理日志（用于定时任务，不启动 GUI）
./target/release/rime-word-counter --process

# 指定自定义路径
./target/release/rime-word-counter \
    --log-path ~/my_rime_word.log \
    --db-path ~/my_stats.db

# 查看帮助
./target/release/rime-word-counter --help
```

### 4. 自动化运行

**Linux (crontab) — 每 30 分钟处理一次日志：**
```bash
crontab -e
# 添加：
*/30 * * * * /path/to/rime-word-counter --process
```

需要查看图表时，直接运行程序即可打开 GUI。

## 命令行参数

| 参数 | 说明 | 默认值 |
|---|---|---|
| `--process` | 仅处理日志（不启动 GUI） | |
| `--gui` | 强制启动 GUI（默认行为） | |
| `--log-path <PATH>` | CSV 日志文件路径 | `~/.cache/rime-word-counter/rime_word.log` |
| `--db-path <PATH>` | SQLite 数据库路径 | `~/.cache/rime-word-counter/rime_stats.db` |
| `--help` | 显示帮助信息 | |

## GUI 使用说明

### 布局
- **顶部栏**：应用标题 + 总字数 + 语言切换下拉菜单
- **左侧筛选面板**：日期范围（输入/快捷按钮）+ 分组依据（日/月/年）+ 重新处理按钮
- **主区域**：字数趋势图表 + 统计卡片（数据天数 / 平均 / 最高 / 最低）

### 语言切换
点击右上角语言下拉菜单，可在简体中文、繁体中文、English 之间切换，所有 UI 文本实时更新。

### 分组说明
- **按日**：显示每一天的字数（原始粒度）
- **按月**：聚合显示每月的总字数
- **按年**：聚合显示每年的总字数

## 多语言支持

程序自动检测系统语言（`LANG` 环境变量）：
- `zh_CN.*` / `zh-Hans` → 简体中文
- `zh_TW.*` / `zh_HK.*` / `zh-Hant` → 繁体中文
- 其他 → English

GUI 内可随时切换语言。

## 数据存储

- **日志文件** (`rime_word.log`): CSV 格式 `YYYY-MM-DD,count`，无表头。处理完毕后自动清空。
- **数据库** (`rime_stats.db`): SQLite 文件，表结构：
  ```sql
  CREATE TABLE daily_words (
      date       TEXT PRIMARY KEY,   -- YYYY-MM-DD
      word_count INTEGER NOT NULL DEFAULT 0
  );
  ```
  已存在的日期会自动累加，不会丢失历史数据。

## 开发

```bash
# 构建
cargo build

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

## 许可证

MIT
