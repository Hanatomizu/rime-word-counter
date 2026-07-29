# Rime 字数统计工具 (rime-word-counter)

为 [Rime 输入法](https://rime.im/) 开发的每日字数统计与可视化工具。

## 架构

```
Lua 脚本（嵌入 Rime）         Rust 程序（定时运行）
┌─────────────────┐         ┌──────────────────────┐
│  Rime 输入法     │ 追加    │  rime-word-counter    │
│  word_counter    │───────→ │                      │
│  (processor)     │  CSV    │  ┌─ log_processor ──┐│
│                   │ 日志    │  │  读取 & 汇总      ││
│  每次上屏 →       │         │  │  (按日期分组)     ││
│  记录日期,字数    │         │  └──────┬───────────┘│
└─────────────────┘         │         ↓            │
                            │  ┌──────┴───────────┐│
                            │  │  SQLite 数据库    ││
                            │  │  daily_words 表   ││
                            │  └──────┬───────────┘│
                            │         ↓            │
                            │  ┌──────┴───────────┐│
                            │  │  visualizer      ││
                            │  │  plotters 图表   ││
                            │  └──────────────────┘│
                            └──────────────────────┘
```

### 工作流程

1. **Lua 脚本** 常驻 Rime 输入法，每次有文本上屏时（中英文输入确认后），
   自动记录 `YYYY-MM-DD,字数` 到 CSV 日志文件。
2. **Rust 程序** 读取 CSV 日志，按日期累加字数，写入 SQLite 数据库，
   然后清空日志文件。
3. **Rust 程序** 从 SQLite 读取最近 N 天的数据，生成 PNG 图表。

## 文件结构

```
rime-word-counter/
├── Cargo.toml               # Rust 项目配置
├── src/
│   ├── main.rs              # 入口，CLI 参数解析
│   ├── db.rs                # SQLite 操作封装
│   ├── log_processor.rs     # 日志处理、汇总、清空
│   └── visualizer.rs        # plotters 图表生成
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

3. 在使用的输入方案（如 `luna_pinyin.schema.yaml` 或 `default.yaml`）中
   `engine/processors` 列表末尾添加 `word_counter_processor`：
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
# 编译发布版本
cargo build --release

# 编译产物在 target/release/rime-word-counter
```

### 3. 运行

```bash
# 处理日志并生成图表（默认模式）
./target/release/rime-word-counter

# 仅处理日志（不生成图表）
./target/release/rime-word-counter --process

# 仅生成图表（从已有数据库读取）
./target/release/rime-word-counter --visualize

# 指定自定义路径
./target/release/rime-word-counter \
    --log-path ~/my_rime_word.log \
    --db-path ~/my_stats.db \
    --output ~/my_chart.png \
    --days 60

# 查看帮助
./target/release/rime-word-counter --help
```

### 4. 自动化运行（推荐）

将 Rust 程序加入定时任务，定期汇总数据：

**Linux (crontab) — 每 30 分钟运行一次：**
```bash
crontab -e
# 添加：
*/30 * * * * /path/to/rime-word-counter --process
0 22 * * * /path/to/rime-word-counter --visualize
```

**macOS (launchd) 或 Windows (任务计划程序)：** 类似配置。

## 命令行参数

| 参数 | 说明 | 默认值 |
|---|---|---|
| `--process` | 仅处理日志，不生成图表 | 默认同时执行 |
| `--visualize` | 仅生成图表，不处理日志 | 默认同时执行 |
| `--log-path <PATH>` | CSV 日志文件路径 | `~/.cache/rime-word-counter/rime_word.log` |
| `--db-path <PATH>` | SQLite 数据库路径 | `~/.cache/rime-word-counter/rime_stats.db` |
| `--output <PATH>` | 图表 PNG 输出路径 | `./word_stats.png` |
| `--days <N>` | 图表涵盖的天数 | `30` |
| `--help` | 显示帮助信息 | |

## 数据存储

- **日志文件** (`rime_word.log`): CSV 格式，每行 `YYYY-MM-DD,count`，无表头。
  处理完毕后会自动清空。
- **数据库** (`rime_stats.db`): SQLite 文件，表结构：
  ```sql
  CREATE TABLE daily_words (
      date       TEXT PRIMARY KEY,   -- YYYY-MM-DD
      word_count INTEGER NOT NULL DEFAULT 0
  );
  ```
  已存在的日期会自动累加。不会丢失历史数据。

## 图表示例

生成的图表包含：
- 📊 **柱状图**：每日字数（橙色柱体）
- 📈 **趋势线**：字数变化趋势（深红色折线）
- 🔵 **数据点**：每日本值标注
- 🏷️ **数值标签**：柱状图上方的具体字数

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
