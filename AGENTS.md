# AGENTS.md

面向本仓库工作的 AI 助手 / 协作者的快速上手文档。
`CLAUDE.md` 是精简速查版，本文件是权威文档。

---

## 1. 这是什么

统计 [Rime 输入法](https://rime.im/) 每日打字字数的工具。Rime 里的 Lua 脚本把每次上屏的
字数追加到 CSV 日志，Rust 程序把日志汇总进 SQLite，再由桌面端 / 命令行展示。

```
Rime + lua/word_counter.lua
        │  追加 "YYYY-MM-DD,字数"
        ▼
~/.cache/rime-word-counter/rime_word.log
        │  log_processor::process_logs()  读取 → 按日累加 → 清空日志
        ▼
~/.cache/rime-word-counter/rime_stats.db  (表 daily_words)
        │  stats::compute_dashboard()     筛选 → 分组 → 概览统计
        ├──────────────► rime-word-counter        命令行（--process / --stats，cron 用）
        └──────────────► rime-word-counter-gui    Tauri 桌面端（默认产品形态）
```

**两个前端共用同一个核心库**（根包的 lib），统计口径完全一致，不存在两套算法。

---

## 2. 仓库结构

```
rime-word-counter/
├── Cargo.toml              工作区根：核心库 + CLI 包，members 含 src-tauri
├── src/                    ── 核心库 rime_word_counter + CLI 二进制
│   ├── lib.rs              导出 db / i18n / log_processor / paths / stats
│   ├── main.rs             CLI 入口：--process、--stats
│   ├── db.rs               SQLite 封装（建表、upsert、分组查询）
│   ├── log_processor.rs    CSV 日志 → 按日汇总 → 写库 → 截断日志
│   ├── stats.rs            仪表盘数据（趋势点 + 概览统计），前后端唯一数据出口
│   ├── paths.rs            缓存目录 / 默认路径 / 按需建父目录
│   └── i18n.rs             系统语言探测（界面文案不在这里，见 ui/src/i18n）
├── src-tauri/              ── Tauri 桌面端（包名 rime-word-counter-gui）
│   ├── src/lib.rs          3 个 Tauri 命令 + 应用状态
│   ├── src/main.rs         桌面端入口
│   ├── tauri.conf.json     窗口 / CSP / 打包配置
│   ├── capabilities/       权限（只开 core:default + 设置窗口标题）
│   └── icons/              应用图标（源图 source.png 由脚本生成）
├── ui/                     ── React + Vite + TypeScript 前端
│   ├── src/App.tsx         应用外壳与全部状态
│   ├── src/types.ts        与 Rust DTO 一一对应（改 Rust 结构体要同步改这里）
│   ├── src/lib/api.ts      invoke 封装 + 浏览器演示模式（无 Tauri 时的假数据）
│   ├── src/lib/{date,format}.ts
│   ├── src/i18n/strings.ts 三语言文案（简中为基准表，漏翻会编译报错）
│   ├── src/components/     Sidebar / StatCards / TrendChart / RecordsTable / Select / Logo
│   └── src/styles/         tokens.css（设计令牌）+ app.css（组件样式）
├── lua/word_counter.lua    Rime 侧 Lua 脚本
├── scripts/generate_icon.py 生成图标源图（改了图标才需要跑）
└── package.json            工作区脚本入口（dev / build / tauri）
```

---

## 3. 环境依赖

| 依赖 | 说明 |
|---|---|
| Rust ≥ 1.77（edition 2021） | 核心库与桌面端 |
| Node.js ≥ 20 + npm | 前端与 Tauri CLI |
| **`webkit2gtk-4.1` + `libjavascriptcoregtk-4.1`** | **Linux 编译 Tauri 必需**，见下 |
| `libsoup-3.0`、`gtk+-3.0` | webkit2gtk 的依赖 |

### Linux 前置（最容易卡住的一步）

Tauri 在 Linux 上通过 `wry` → `webkit2gtk-4.1` 渲染，缺了它 `cargo build` 会在
`webkit2gtk-sys` 的 build script 处报 pkg-config 找不到库。

```bash
# Arch
sudo pacman -S webkit2gtk-4.1

# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel libsoup3-devel
```

**没有 root 权限时的临时方案**（仅供本地验证，勿写进仓库）：把发行版包解到用户目录，
改写 `.pc` 里的 `prefix`，然后编译时指定：

```bash
PKG_CONFIG_PATH="$HOME/.local/tauri-deps/prefix/usr/lib/pkgconfig" \
  cargo build --manifest-path src-tauri/Cargo.toml
```

**只想跑测试 / 用 cron？不需要 webkit。** 根包不依赖 Tauri：

```bash
cargo test     # 只编译核心库 + CLI
cargo build    # 同上（workspace 的 default-members = ["."]）
```

---

## 4. 常用命令

```bash
# ── 核心库 / CLI（无需 webkit）──
cargo test                     # 全部单元测试
cargo test -- --nocapture      # 显示 println 输出
cargo test test_dashboard_by_month   # 跑单个测试
cargo build --release
cargo run --release -- --process     # 处理日志（cron 用）
cargo run --release -- --stats       # 终端打印统计摘要
cargo run --release -- --help

# ── 桌面端 ──
npm install                    # 首次：安装根 + ui 的前端依赖
npm run tauri dev              # 开发模式（Vite HMR + Tauri 窗口）
npm run tauri build            # 打包安装包（deb/rpm/AppImage/msi/dmg…）
npm run dev                    # 只起前端，浏览器打开 http://localhost:5173（演示数据）
npm run build                  # 只构建前端到 ui/dist
npm run typecheck              # tsc --noEmit

# ── 图标 ──
python3 scripts/generate_icon.py && npx tauri icon src-tauri/icons/source.png
```

> `npm run tauri dev/build` 必须在**仓库根目录**执行：`tauri.conf.json` 里的
> `beforeDevCommand` / `beforeBuildCommand` 是 `npm run dev` / `npm run build`，
> 依赖根目录的 `package.json`。

---

## 5. 模块职责与关键不变量

### `db.rs`
- 表结构只有一张：`daily_words(date TEXT PRIMARY KEY, word_count INTEGER NOT NULL DEFAULT 0)`
- **upsert 必须累加而不是覆盖**：`ON CONFLICT(date) DO UPDATE SET word_count = word_count + ?2`。
  同一天多次导入要相加，这是历史数据不丢的关键。
- `init_db()` 会自动创建数据库文件的父目录，因此可以传任意自定义路径。
- `GroupBy::from_code()` 把前端传来的 `day/month/year` 转成枚举，未知值回退按日。

### `log_processor.rs`
- **日志用 `File::set_len(0)` 截断，不删除文件**：Lua 侧可能正持有该文件句柄，
  删除会导致后续写入落到已删除的 inode 上，日志凭空消失。
- 日志不存在 → 返回空 `ProcessReport`，不报错（首次运行属于正常情况）。
- 解析失败的行只记 warning 并跳过，不中断整批处理。
- 返回 `ProcessReport { lines_read, parse_errors, dates_updated }`，桌面端用它拼提示语。

### `stats.rs`
- 唯一的数据出口：`compute_dashboard(conn, start, end, group_by) -> Dashboard`。
- `average` 是**日均**（`total_range / 有记录的天数`），不是「按分组数平均」；
  切到按月分组时日均不应该跟着变。
- `days` 是范围内**有记录的天数**，`buckets` 是分组后的点数，两者不同。
- 空范围返回全 0 而不是报错；日期非法才报错（前端要能区分「没数据」和「输入没写完」）。
- DTO 全部 `#[serde(rename_all = "camelCase")]`，与 `ui/src/types.ts` 一一对应。

### `paths.rs`
- 默认路径：`dirs::cache_dir()/rime-word-counter/{rime_word.log,rime_stats.db}`。
  Lua 脚本里写死了同样的路径，改这里必须同步改 `lua/word_counter.lua`。

### `i18n.rs`（Rust 侧）
- 只负责探测系统语言并交给前端，`Language::code()` 返回 `zh-CN` / `zh-TW` / `en`。
- **界面文案不在这里**，在 `ui/src/i18n/strings.ts`；Rust 侧不再维护一份 UI 文案。

### `src-tauri/src/lib.rs`
- 三个命令：`get_bootstrap`（语言 + 路径 + 版本）、`get_dashboard`（查数据）、
  `reprocess`（处理日志并返回新数据）。
- 路径放在 `AppState` 里，支持 `--log-path` / `--db-path` 覆盖；解析失败回退默认值，
  不阻断启动。
- 错误统一 `Result<T, String>`，把 anyhow 的上下文链 `{:#}` 展开给前端展示。

### `ui/src/App.tsx`
- 全部状态集中在这里，数据流是 `request（范围 + 分组）→ getDashboard → dashboard`。
- 日期输入 240ms 防抖，且**只有两个日期都合法才发请求**。
- `hydratedRef` 保证后端返回的真实日期只回填输入框一次，之后不再覆盖用户输入。
- 「重新处理」时如果原先没有数据或选的是「全部」，会把范围扩到新的完整区间，
  否则刚导入的历史数据会落在筛选范围外看不见。

### `ui/src/lib/api.ts`
- 通过 `window.__TAURI_INTERNALS__` 判断是否在 Tauri 里；**不在就返回本地生成的演示数据**，
  这样 `npm run dev` 用普通浏览器也能调 UI。演示模式会在界面上显示「演示数据」徽章。

---

## 6. 前端设计系统（Linear 风格深色 SaaS）

改 UI 前先读 `ui/src/styles/tokens.css`，三条硬规则：

1. **画布是近黑 `#010102`，不要用纯黑**；层级靠 `surface-1 → surface-4` 逐级提亮的炭黑面板。
2. **全站只有一个强调色 `--primary: #5e6ad2`**（薰衣草蓝），只出现在品牌标记、焦点环、
   主按钮和图表数据上，不做装饰性点缀。语义色 `--success` / `--danger` 仅用于状态。
3. **分隔一律 1px hairline（`--hairline`），不要大面积阴影**；圆角不超过 12px。

排版：正文 13px / 字重 400；标题 600 + 负字距（`-0.02em` ~ `-0.03em`）；
数字一律 `--font-mono` + `font-variant-numeric: tabular-nums`。
字体优先 Inter（`@fontsource-variable/inter` 随包分发），中文回退到系统 Noto Sans CJK / 微软雅黑。

图表是**手写 SVG**（`ui/src/components/TrendChart.tsx`），没有引图表库：
网格用 hairline、数据用唯一强调色、坐标文字用 `--ink-tertiary`，容器宽度用
`ResizeObserver` 跟随（不使用 viewBox 缩放，避免坐标文字被拉伸）。

### ⚠️ 踩坑记录：不要用 `aria-*` 属性驱动样式

本项目实测 **React 19 在重渲染时不会更新 `aria-*` 属性**：
首帧 `aria-invalid={true}` 之后即使变成 `false`，DOM 上仍然是 `"true"`；
`aria-pressed` 从 `undefined` 变成 `true` 也不会写进去。用
`.input[aria-invalid='true']` 这类选择器会导致样式永远停在首帧状态。

因此**状态样式一律用 class**：

```tsx
className={`chip${activePreset === id ? ' chip--active' : ''}`}
aria-pressed={activePreset === id}   // 保留给读屏软件，不参与样式
```

CSS 里可以同时写 `.chip[aria-pressed='true'], .chip--active { … }`，
但**生效的必须是 class 那一份**。

---

## 7. 测试

```bash
cargo test
```

29 个单元测试，覆盖：

| 模块 | 覆盖内容 |
|---|---|
| `db` | upsert 新建 / 累加、按日/月/年分组、总量、日期范围 |
| `log_processor` | 单行解析（含空格、非法日期、缺逗号、非数字）、集成流程（写库 + 截断日志）、缺文件、解析失败计数 |
| `stats` | 三种分组、日均口径、空范围为 0、非法日期报错、起止倒置报错 |
| `paths` | 默认路径落在缓存目录、`:memory:` 与裸文件名跳过、递归建目录 |
| `i18n` | locale 归类（简中/繁中/其它）、语言代码与前端 key 一致 |

前端目前没有测试，靠 `npm run typecheck`（严格模式，开了 `noUnusedLocals`）和
`npm run dev` 的演示数据肉眼验证。

**改统计逻辑时请同时更新 `stats.rs` 的测试**，它是口径的唯一真源。

---

## 8. 发布

发布全部由 **`.github/workflows/release.yml`**（手写维护，不再由 cargo-dist 生成）驱动，
推送形如 `v1.1.0` 的 tag 即可触发，CLI 与桌面端会汇总进**同一个 GitHub Release**。

| Job | 产物 |
|---|---|
| `versioning` | 解析 tag，并校验 5 处版本号是否一致（不一致直接失败） |
| `cli` | 6 个平台的 `rime-word-counter-<triple>.tar.xz` / `.zip` |
| `desktop` | Linux `.deb` + `.AppImage`（x86_64 / aarch64）、macOS `.dmg`、Windows `.msi` + NSIS `.exe` |
| `publish` | 合并全部产物 + `SHA256SUMS.txt`，创建 Release |

- CLI 用 `cargo build --profile dist`，桌面端用 `npm run tauri build -- --bundles …`，
  两者共用根包的统计逻辑。
- **版本号有 5 处需要同步**：根 `Cargo.toml`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、
  `package.json`、`ui/package.json`（当前 `1.1.0`）。`ui/src/lib/api.ts` 里的演示版本文案
  和 `Cargo.lock` 也一并跟着改。tag 与这些不一致时 `versioning` job 会直接失败。
- **不要用预发布后缀**（如 `1.1.0-r2`）：MSI/NSIS 对含 `-` 的版本号敏感，历史上踩过坑，
  现在 `1.1.0` 起统一用纯 `major.minor.patch`。
- 桌面端产物**未签名/未公证**：macOS 需 `xattr -dr com.apple.quarantine`，
  Windows 需在 SmartScreen 里放行。签名与公证尚未接入。
- Linux 桌面端在 `ubuntu-24.04` 上构建，**要求用户机器 glibc ≥ 2.39**；
  想让老发行版可用需改用更老的构建镜像并自行编译 webkit2gtk。
- `dist-workspace.toml` 保留但**不再驱动 CI**（见该文件顶部注释），不要跑 `dist init`。

### ⚠️ 改这个 workflow 时的两个坑（都实际踩过）

1. **桌面端产物路径带目标三元组**：CI 里带 `--target <triple>` 构建，产物落在
   `target/<triple>/release/bundle/`，**不是** `target/release/bundle/`（后者只在
   不带 `--target` 的原地构建时出现）。job 里的 `path` 一律用 `**/release/bundle/…`
   通配来兼容两种布局，别再写死。
2. **run 块按平台选工具**：`desktop` / `cli` 两个 job 跑在三个平台上，块内不能出现
   - `/dev/null`（Windows PowerShell 会解析成 `D:\dev\null` 而报错）
   - `sha256sum`（macOS 与 Windows Git Bash 都没有，那是 GNU coreutils）

   校验和统一用 `node`（三个平台都有）；只有 `publish` / `versioning`（固定
   `ubuntu-24.04`）才可以用 GNU 工具。只用 bash 语法而**不实际跨平台跑过**的写法，
   很容易在 macOS/Windows 上才炸。

---

## 9. 排查清单

| 现象 | 原因 / 处理 |
|---|---|
| `cargo build` 报 `webkit2gtk-4.1 not found` | 未装系统依赖，见第 3 节；只想跑 CLI/测试就用 `cargo test` |
| 启动时报 `Failed to spawn child process "/usr/lib/webkit2gtk-4.1/WebKitNetworkProcess"` | WebKit 的辅助进程路径是**编译期写死**的（`PKGLIBEXECDIR`），2.52 起不再读取 `WEBKIT_EXEC_PATH`。用非系统路径的 webkit 一定会撞上，必须真正装上发行版包 |
| `frontendDist` 路径不存在 | 先 `npm run build` 生成 `ui/dist`，`tauri build` 需要它存在 |
| 界面没有数据但数据库有记录 | 检查日期范围输入框；点「全部」快捷按钮 |
| 中文显示成方框 | 系统缺 CJK 字体，装 `noto-fonts-cjk` / `wqy-microhei` |
| 日志处理完但字数没变 | Lua 写入路径与 `--log-path` 是否一致；`--stats` 看数据库真实内容 |
| 图表/按钮选中态不更新 | 见第 6 节的 `aria-*` 踩坑，改用 class |
| `npm run tauri dev` 起不来 | 必须在仓库根目录执行；端口 5173 被占用会直接失败（`strictPort`） |
