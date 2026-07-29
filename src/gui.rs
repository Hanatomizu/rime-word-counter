//! GUI 界面模块
//!
//! 使用 egui (eframe) 实现字数统计的图形界面。
//! 包含筛选面板、图表显示、多语言切换等功能。

use std::collections::HashMap;

use anyhow::Result;
use chrono::NaiveDate;

use crate::db::{self, GroupBy};
use crate::i18n::{self, Language, Strings};
use crate::log_processor;
use crate::visualizer;

/// 图表渲染的固定尺寸
const CHART_RENDER_WIDTH: u32 = 1200;
const CHART_RENDER_HEIGHT: u32 = 500;

/// GUI 应用主状态
pub struct GuiApp {
    db_path: String,
    lang: Language,

    // 全部数据（按日，从数据库加载）
    all_data: Vec<(String, i64)>,
    total_words: i64,

    // 筛选状态
    start_date: String,
    end_date: String,
    group_by: GroupBy,

    // 筛选后的数据
    filtered_data: Vec<(String, i64)>,

    // 统计数据
    avg_words: f64,
    max_words: i64,
    min_words: i64,
    display_days: usize,

    // 图表纹理
    chart_texture: Option<egui::TextureHandle>,
    chart_needs_update: bool,
    chart_version: u64,

    // 界面状态
    is_processing: bool,
    status_message: String,
    error_message: Option<String>,

    // 日期输入缓冲区
    start_date_buf: String,
    end_date_buf: String,

    // 检测变化
    prev_group_by: GroupBy,
}

impl GuiApp {
    /// 创建并初始化 GUI 应用。
    pub fn new(db_path: &str) -> Self {
        let lang = i18n::detect_language();

        let mut app = GuiApp {
            db_path: db_path.to_string(),
            lang,
            all_data: Vec::new(),
            total_words: 0,
            start_date: String::new(),
            end_date: String::new(),
            group_by: GroupBy::Day,
            filtered_data: Vec::new(),
            avg_words: 0.0,
            max_words: 0,
            min_words: 0,
            display_days: 0,
            chart_texture: None,
            chart_needs_update: true,
            chart_version: 0,
            is_processing: true,
            status_message: String::new(),
            error_message: None,
            start_date_buf: String::new(),
            end_date_buf: String::new(),
            prev_group_by: GroupBy::Day,
        };

        match app.load_data() {
            Ok(_) => app.is_processing = false,
            Err(e) => {
                app.error_message = Some(e.to_string());
                app.is_processing = false;
            }
        }

        app
    }

    /// 加载数据：处理日志 → 读取数据库 → 初始化筛选
    fn load_data(&mut self) -> Result<()> {
        let s = Strings::for_language(self.lang);

        self.status_message = s.processing_log.to_string();
        log_processor::process_logs(&crate::default_log_path(), &self.db_path)?;

        let conn = db::init_db(&self.db_path)?;

        self.status_message = s.loading_data.to_string();
        self.all_data = db::query_all(&conn)?;
        self.total_words = db::query_total_words(&conn)?;

        let (db_start, db_end) = db::query_date_range(&conn)?;
        self.start_date = db_start;
        self.end_date = db_end;
        self.start_date_buf = self.start_date.clone();
        self.end_date_buf = self.end_date.clone();

        self.apply_filter();
        Ok(())
    }

    /// 重新处理日志并刷新数据
    fn reprocess(&mut self) {
        self.is_processing = true;
        self.error_message = None;
        if let Err(e) = self.load_data() {
            self.error_message = Some(e.to_string());
        }
        self.is_processing = false;
        self.chart_needs_update = true;
    }

    /// 根据当前筛选条件过滤并聚合数据
    fn apply_filter(&mut self) {
        let start_ok = NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d").is_ok();
        let end_ok = NaiveDate::parse_from_str(&self.end_date, "%Y-%m-%d").is_ok();

        if !start_ok || !end_ok || self.all_data.is_empty() {
            self.filtered_data.clear();
            self.avg_words = 0.0;
            self.max_words = 0;
            self.min_words = 0;
            self.display_days = 0;
            self.chart_needs_update = true;
            return;
        }

        let mut in_range: Vec<&(String, i64)> = self
            .all_data
            .iter()
            .filter(|(date, _)| {
                date.as_str() >= self.start_date.as_str()
                    && date.as_str() <= self.end_date.as_str()
            })
            .collect();

        in_range.sort_by(|a, b| a.0.cmp(&b.0));

        self.filtered_data = match self.group_by {
            GroupBy::Day => in_range.iter().map(|(d, c)| (d.clone(), *c)).collect(),
            GroupBy::Month => {
                let mut map: HashMap<String, i64> = HashMap::new();
                for (date, count) in in_range {
                    let key = date[..7].to_string();
                    *map.entry(key).or_insert(0) += count;
                }
                let mut result: Vec<_> = map.into_iter().collect();
                result.sort_by(|a, b| a.0.cmp(&b.0));
                result
            }
            GroupBy::Year => {
                let mut map: HashMap<String, i64> = HashMap::new();
                for (date, count) in in_range {
                    let key = date[..4].to_string();
                    *map.entry(key).or_insert(0) += count;
                }
                let mut result: Vec<_> = map.into_iter().collect();
                result.sort_by(|a, b| a.0.cmp(&b.0));
                result
            }
        };

        let counts: Vec<i64> = self.filtered_data.iter().map(|(_, c)| *c).collect();
        self.display_days = self.filtered_data.len();
        self.avg_words = if !counts.is_empty() {
            counts.iter().sum::<i64>() as f64 / counts.len() as f64
        } else {
            0.0
        };
        self.max_words = counts.iter().max().copied().unwrap_or(0);
        self.min_words = counts.iter().min().copied().unwrap_or(0);

        self.chart_needs_update = true;
    }

    /// 渲染图表到纹理
    fn update_chart_texture(&mut self, ctx: &egui::Context) {
        if !self.chart_needs_update || self.filtered_data.is_empty() {
            return;
        }

        let s = Strings::for_language(self.lang);
        self.status_message = s.rendering_chart.to_string();

        let font_name = visualizer::find_cjk_font();
        let x_label = match self.group_by {
            GroupBy::Day => &s.chart_x_label,
            GroupBy::Month => &s.monthly,
            GroupBy::Year => &s.yearly,
        };
        let y_label = &s.chart_y_label;

        let result = visualizer::render_chart_to_rgba(
            &self.filtered_data,
            font_name,
            CHART_RENDER_WIDTH,
            CHART_RENDER_HEIGHT,
            x_label,
            y_label,
        );

        match result {
            Ok(rgba_data) => {
                let size = [CHART_RENDER_WIDTH as usize, CHART_RENDER_HEIGHT as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba_data);
                self.chart_texture = Some(ctx.load_texture(
                    &format!("chart_v{}", self.chart_version),
                    color_image,
                    egui::TextureOptions::LINEAR,
                ));
                self.chart_version += 1;
                self.chart_needs_update = false;
                self.status_message = String::new();
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
    }

    /// 快速设置日期范围
    fn set_date_range_days(&mut self, days: i64) {
        let end = chrono::Local::now().date_naive();
        let start = end - chrono::Duration::days(days - 1);
        self.start_date = start.format("%Y-%m-%d").to_string();
        self.end_date = end.format("%Y-%m-%d").to_string();
        self.start_date_buf = self.start_date.clone();
        self.end_date_buf = self.end_date.clone();
        self.apply_filter();
    }

    fn set_date_range_all(&mut self) {
        if let Some((first, _)) = self.all_data.first() {
            self.start_date = first.clone();
            self.start_date_buf = self.start_date.clone();
        }
        if let Some((last, _)) = self.all_data.last() {
            self.end_date = last.clone();
            self.end_date_buf = self.end_date.clone();
        }
        self.apply_filter();
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let strings = Strings::for_language(self.lang);

        // ---- 加载中 ----
        if self.is_processing {
            let msg = self.status_message.clone();
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.4);
                    ui.spinner();
                    ui.label(msg);
                });
            });
            ctx.request_repaint();
            return;
        }

        // ---- 顶部栏 ----
        egui::TopBottomPanel::top("top_bar")
            .min_height(48.0)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                inner_margin: egui::Margin::symmetric(16, 10),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new(strings.app_title)
                            .color(egui::Color32::from_rgb(0xFF, 0xFF, 0xFF))
                            .size(18.0),
                    );

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            let current_lang_label = Language::all()
                                .iter()
                                .find(|(l, _)| *l == self.lang)
                                .map(|(_, label)| *label)
                                .unwrap_or("");

                            egui::ComboBox::from_id_salt("lang_selector")
                                .selected_text(current_lang_label)
                                .width(90.0)
                                .show_ui(ui, |ui| {
                                    for (lang, label) in Language::all() {
                                        if ui.selectable_label(*lang == self.lang, *label).clicked() {
                                            self.lang = *lang;
                                        }
                                    }
                                });

                            let total_text = strings
                                .total_words_fmt
                                .replace("{}", &format_count(self.total_words));
                            ui.label(
                                egui::RichText::new(total_text)
                                    .color(egui::Color32::from_rgb(0xE8, 0x6C, 0x00))
                                    .size(16.0)
                                    .strong(),
                            );

                            ui.label(
                                egui::RichText::new(strings.total_words)
                                    .color(egui::Color32::from_gray(0xAA))
                                    .size(13.0),
                            );
                        },
                    );
                });
            });

        // ---- 侧边栏 ----
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(220.0)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                inner_margin: egui::Margin::symmetric(16, 16),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 筛选标题
                    ui.label(
                        egui::RichText::new(strings.filter_panel)
                            .size(14.0)
                            .strong()
                            .color(egui::Color32::from_rgb(0x55, 0x55, 0x55)),
                    );
                    ui.separator();
                    ui.add_space(8.0);

                    // 开始日期
                    ui.label(
                        egui::RichText::new(strings.start_date)
                            .size(12.0)
                            .color(egui::Color32::from_gray(0x88)),
                    );
                    let sr = ui.add_sized(
                        [ui.available_width(), 24.0],
                        egui::TextEdit::singleline(&mut self.start_date_buf)
                            .hint_text("YYYY-MM-DD")
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                    if sr.changed() {
                        self.start_date = self.start_date_buf.clone();
                        self.apply_filter();
                    }
                    ui.add_space(8.0);

                    // 结束日期
                    ui.label(
                        egui::RichText::new(strings.end_date)
                            .size(12.0)
                            .color(egui::Color32::from_gray(0x88)),
                    );
                    let er = ui.add_sized(
                        [ui.available_width(), 24.0],
                        egui::TextEdit::singleline(&mut self.end_date_buf)
                            .hint_text("YYYY-MM-DD")
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                    if er.changed() {
                        self.end_date = self.end_date_buf.clone();
                        self.apply_filter();
                    }
                    ui.add_space(8.0);

                    // 快捷按钮
                    ui.label(
                        egui::RichText::new("Quick:")
                            .size(11.0)
                            .color(egui::Color32::from_gray(0x99)),
                    );
                    ui.horizontal_wrapped(|ui| {
                        let bs = egui::vec2((ui.available_width() - 4.0) / 2.0, 26.0);
                        if ui.add_sized(bs, egui::Button::new(strings.last_7_days)).clicked() {
                            self.set_date_range_days(7);
                        }
                        if ui.add_sized(bs, egui::Button::new(strings.last_30_days)).clicked() {
                            self.set_date_range_days(30);
                        }
                        if ui.add_sized(bs, egui::Button::new(strings.last_year)).clicked() {
                            self.set_date_range_days(365);
                        }
                        if ui.add_sized(bs, egui::Button::new(strings.all_time)).clicked() {
                            self.set_date_range_all();
                        }
                    });
                    ui.add_space(16.0);

                    // 分组依据
                    ui.label(
                        egui::RichText::new(strings.group_by)
                            .size(12.0)
                            .color(egui::Color32::from_gray(0x88)),
                    );
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.group_by, GroupBy::Day, strings.daily);
                        ui.radio_value(&mut self.group_by, GroupBy::Month, strings.monthly);
                        ui.radio_value(&mut self.group_by, GroupBy::Year, strings.yearly);
                    });
                    if self.prev_group_by != self.group_by {
                        self.prev_group_by = self.group_by;
                        self.apply_filter();
                    }

                    ui.add_space(24.0);

                    if ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            egui::Button::new(
                                egui::RichText::new(strings.reprocess)
                                    .color(egui::Color32::from_rgb(0xE8, 0x6C, 0x00)),
                            )
                            .fill(egui::Color32::from_rgb(0xFF, 0xF0, 0xE0)),
                        )
                        .clicked()
                    {
                        self.reprocess();
                    }
                });
            });

        // ---- 主内容区域 ----
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(0xFA, 0xFA, 0xFA),
                inner_margin: egui::Margin::symmetric(20, 16),
                ..Default::default()
            })
            .show(ctx, |ui| {
                if self.all_data.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.35);
                        ui.label(
                            egui::RichText::new(strings.no_data)
                                .size(16.0)
                                .color(egui::Color32::from_gray(0xAA)),
                        );
                    });
                    return;
                }

                // 更新图表
                self.update_chart_texture(ctx);

                if let Some(texture) = &self.chart_texture {
                    let avail = ui.available_size();
                    let aspect = CHART_RENDER_WIDTH as f32 / CHART_RENDER_HEIGHT as f32;
                    let img_width = avail.x.min(1200.0);
                    let img_height = (img_width / aspect).min(avail.y * 0.65);

                    ui.add(
                        egui::Image::new(texture)
                            .max_width(img_width)
                            .max_height(img_height),
                    );
                }

                ui.add_space(12.0);

                // 统计信息
                if !self.filtered_data.is_empty() {
                    let day_label = match self.group_by {
                        GroupBy::Day => strings.daily,
                        GroupBy::Month => strings.monthly,
                        GroupBy::Year => strings.yearly,
                    };

                    ui.horizontal(|ui| {
                        let stat_style = |text: &str| {
                            egui::RichText::new(text)
                                .size(13.0)
                                .color(egui::Color32::from_rgb(0x55, 0x55, 0x55))
                        };

                        let days_text = strings
                            .stats_days
                            .replace("{}", &self.display_days.to_string());
                        frame_card(ui, |ui| {
                            ui.label(stat_style(&format!("{} {}", days_text, day_label)));
                        });

                        let avg_text =
                            strings.stats_avg.replace("{}", &format_count(self.avg_words as i64));
                        frame_card(ui, |ui| {
                            ui.label(stat_style(&avg_text));
                        });

                        let max_text =
                            strings.stats_max.replace("{}", &format_count(self.max_words));
                        frame_card(ui, |ui| {
                            ui.label(stat_style(&max_text));
                        });

                        let min_text =
                            strings.stats_min.replace("{}", &format_count(self.min_words));
                        frame_card(ui, |ui| {
                            ui.label(stat_style(&min_text));
                        });
                    });
                }

                if let Some(err) = &self.error_message {
                    ui.add_space(8.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(0xC0, 0x39, 0x2B),
                        format!("{}: {}", strings.error_prefix, err),
                    );
                }

                if !self.status_message.is_empty() {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_gray(0x88), &self.status_message);
                }
            });
    }
}

/// 格式化数字（例：12345 → "12,345"）
fn format_count(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// 绘制卡片背景 frame
fn frame_card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame {
        fill: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        corner_radius: egui::CornerRadius::same(6),
        shadow: egui::epaint::Shadow {
            offset: [0, 1].into(),
            blur: 4,
            spread: 0,
            color: egui::Color32::from_black_alpha(20),
        },
        inner_margin: egui::Margin::symmetric(10, 6),
        ..Default::default()
    }
    .show(ui, add_contents);
}

/// 启动 GUI 界面
pub fn run_gui(db_path: &str) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(String::new())
            .with_inner_size([1100.0, 680.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    let db_path_owned = db_path.to_string();
    eframe::run_native(
        "Rime Word Counter",
        options,
        Box::new(move |_cc| Ok(Box::new(GuiApp::new(&db_path_owned)))),
    )
    .map_err(|e| anyhow::anyhow!("GUI 错误: {e}"))?;

    Ok(())
}
