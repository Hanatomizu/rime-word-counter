# CLAUDE.md

本仓库的完整上手文档见 **[AGENTS.md](./AGENTS.md)**（架构、模块职责、命令、设计系统、踩坑记录）。
本文件只保留最容易踩错的几条，避免两份文档漂移。

## 这是什么

Rime 输入法每日字数统计工具。Lua 脚本写 CSV 日志 → Rust 汇总进 SQLite → 桌面端 / 命令行展示。

## 两个前端，一个核心

| 目标 | 产物 | 入口 |
|---|---|---|
| 桌面端（默认产品形态） | `rime-word-counter-gui` | `src-tauri/`（Tauri 2 + React 19 + Vite） |
| 命令行（cron） | `rime-word-counter` | `src/main.rs`（`--process` / `--stats`） |

统计逻辑只有一份：根包的 lib（`src/{db,log_processor,stats,paths,i18n}.rs`），
两个前端都调它。**不要在 `src-tauri` 或前端里重新实现分组/汇总逻辑。**

## 命令

```bash
cargo test                       # 29 个单元测试（核心库 + CLI，不需要 webkit）
cargo run --release -- --process # 处理日志（cron）
cargo run --release -- --stats   # 终端看统计

npm install                      # 前端依赖（首次）
npm run tauri dev                # 桌面端开发（必须在仓库根目录执行）
npm run tauri build              # 打包桌面端
npm run typecheck                # 前端类型检查
```

## 三条硬约束

1. **Linux 编译桌面端需要 `webkit2gtk-4.1`**：`sudo pacman -S webkit2gtk-4.1` /
   `sudo apt install libwebkit2gtk-4.1-dev`。只跑 `cargo test` 和 CLI 则不需要。
2. **日志只能截断不能删除**：`log_processor` 用 `File::set_len(0)`，因为 Lua 侧可能
   正持有文件句柄，删除会让后续写入丢失。
3. **状态样式用 class，不要用 `aria-*` 属性选择器**：实测 React 19 重渲染时不会更新
   `aria-*` 属性，`[aria-invalid='true']` 会永远停在首帧状态。详见 AGENTS.md 第 6 节。

## 设计风格

Linear 风格深色 SaaS：近黑画布 `#010102`、单一强调色 `#5e6ad2`、1px hairline 分隔、
层级靠 surface 提亮而非阴影。改动前先看 `ui/src/styles/tokens.css`。
