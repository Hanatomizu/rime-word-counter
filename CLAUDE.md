# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build
cargo build --release

# Test (14 unit tests across db + log_processor)
cargo test
cargo test -- --nocapture   # show stdout/stderr

# Run a single test
cargo test test_upsert_accumulate
cargo test test_parse_valid_line

# Run CLI log processing (for cron)
cargo run --release -- --process

# Run GUI (default)
cargo run --release

# Run with custom paths
cargo run --release -- --log-path ~/custom.log --db-path ~/custom.db
```

## Architecture

The project tracks Rime IME typing stats. A Lua script logs commits to a CSV file; a Rust app processes logs into SQLite and displays stats via a native GUI.

```
Lua (in Rime) → CSV log → log_processor → SQLite → GUI (egui)
                                              ↓
                                        plotters (chart)
```

### Modules

- **`main.rs`** — Entry point. Parses CLI args (clap), dispatches to `--process` (log-only) or default GUI mode. Exports `default_log_path()`, `default_db_path()`, `get_cache_dir()` used by other modules.

- **`log_processor.rs`** — Reads CSV (`YYYY-MM-DD,count`), groups by date, upserts to SQLite via `db`, truncates the log file. Single public fn: `process_logs(log_path, db_path)`.

- **`db.rs`** — SQLite wrapper. Table `daily_words(date TEXT PK, word_count INTEGER)`. `GroupBy` enum (Day/Month/Year) with `query_grouped()` using SQL `substr()`. Also `init_db()`, `upsert_word_count()`, `query_all()`, `query_total_words()`, `query_date_range()`.

- **`visualizer.rs`** — plotters renders a combined bar+line chart to an RGBA pixel buffer. Key fn: `render_chart_to_rgba(data, font_name, width, height, x_label, y_label) -> Vec<u8>`. Font detection via `find_cjk_font()` probing fontconfig.

- **`gui.rs`** — egui (eframe) native window. Three panels: top bar (title, total count, lang/theme toggles), left sidebar (date range inputs, quick buttons, group_by radio, reprocess button), central (chart image + stat cards). Loads CJK font into egui's `FontDefinitions` at startup via `setup_cjk_font()`. Supports dark/light mode toggle.

- **`i18n.rs`** — Three-language string table (zh_CN, zh_TW, en). Auto-detects from `LANG` env var. `Language` enum + `Strings` struct with all UI text.

### Key Design Decisions

- **Log truncation**: `File::set_len(0)` after processing — never deletes the file, safe for concurrent Lua writes
- **DB upsert**: `INSERT ... ON CONFLICT DO UPDATE SET word_count = word_count + EXCLUDED.word_count` — atomic accumulation
- **Chart rendering**: plotters → RGB buffer → manual RGBA conversion → egui texture. No file I/O for charts in GUI mode.
- **Font for plotters**: probed via fontconfig (`find_cjk_font()`)
- **Font for egui**: loaded from known CJK font file paths at startup (`setup_cjk_font()`) — two independent font systems
- **Data flow**: all data loaded from SQLite at startup into `Vec<(String, i64)>`, filtering/grouping done in-memory

### Dependencies

| Crate | Purpose |
|---|---|
| clap | CLI argument parsing |
| rusqlite (bundled) | SQLite, no system dep |
| chrono | Date parsing/formatting |
| plotters | Chart rendering to buffer |
| eframe/egui | Native GUI framework |
| dirs | Platform cache directory |
| anyhow | Error handling |

## Tests

14 tests (`db.rs` + `log_processor.rs`). The `db` tests use `Connection::open_in_memory()`. Integration test `test_process_logs_integration` creates temp log and db files, processes them, verifies DB state and file truncation, then cleans up.
