//! 图表渲染模块
//!
//! 使用 plotters 库将字数数据渲染为折线+柱状叠加图。
//! 输出为 RGBA 像素缓冲区，供 GUI 或其它模块使用。

use anyhow::{Context, Result};
use plotters::backend::RGBPixel;
use plotters::prelude::*;

/// 色调：Rime 风格的暖橙色系
const BAR_COLOR: RGBColor = RGBColor(0xE8, 0x6C, 0x00); // 橙色柱体
const LINE_COLOR: RGBColor = RGBColor(0xC0, 0x39, 0x2B); // 深红色折线
const ACCENT_COLOR: RGBColor = RGBColor(0xF3, 0x9C, 0x12); // 金色数据点

/// 支持中文的候选字体列表（按优先级排序）。
const CJK_FONT_CANDIDATES: &[&str] = &[
    "DengXian",
    "AR PL UMing CN",
    "AR PL UKai CN",
    "Noto Sans CJK SC",
    "Noto Serif CJK SC",
    "WenQuanYi Micro Hei",
    "Microsoft YaHei",
    "SimHei",
    "FangSong",
];

/// 探测系统中可用的中文字体，返回第一个能正常加载的字体家族名。
pub fn find_cjk_font() -> &'static str {
    for name in CJK_FONT_CANDIDATES {
        let font: FontDesc = (*name, 12).into_font();
        if font.layout_box("测试").is_ok() {
            return name;
        }
    }
    "sans-serif"
}

/// 将字数数据渲染为 RGBA 像素缓冲区。
///
/// * `data` — 已按分组聚合的数据，格式 `[(label, count)]`
/// * `font_name` — 字体家族名（由 `find_cjk_font()` 获取）
/// * `width` — 输出图像宽度（像素）
/// * `height` — 输出图像高度（像素）
/// * `x_label` — X 轴标签文字
/// * `y_label` — Y 轴标签文字
///
/// 返回 RGBA 格式的像素数据（每个像素 4 字节：R, G, B, A）。
pub fn render_chart_to_rgba(
    data: &[(String, i64)],
    font_name: &str,
    width: u32,
    height: u32,
    x_label: &str,
    y_label: &str,
) -> Result<Vec<u8>> {
    if data.is_empty() {
        anyhow::bail!("没有数据可供渲染");
    }

    // 分配 RGB 缓冲区（plotters 默认使用 RGB888）
    let buf_size = (width * height * 3) as usize;
    let mut rgb_buf = vec![0u8; buf_size];

    // 颜色预先定义（需比 chart 活得更久）
    let bar_color = BAR_COLOR.mix(0.7);
    let line_style = LINE_COLOR.stroke_width(2);

    // 将渲染过程包裹在块中，确保 root 在 rgb_buf 读取前被 drop
    {
        // 创建内存后端
        let root =
            BitMapBackend::<RGBPixel>::with_buffer_and_format(&mut rgb_buf, (width, height))
                .map_err(|e| anyhow::anyhow!("创建位图后端失败: {e}"))?
                .into_drawing_area();
        root.fill(&WHITE).context("填充背景失败")?;

        let max_count = data.iter().map(|(_, c)| *c).max().unwrap_or(1) as f64;
        let y_max = (max_count * 1.2).ceil().max(10.0);

        let labels: Vec<String> = data.iter().map(|(d, _)| d.clone()).collect();
        let count = data.len();
        let x_min = -0.5f64;
        let x_max = (count - 1) as f64 + 0.5;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                &format!("{} — {} {}条", y_label, x_label, count),
                (font_name, 22).into_font(),
            )
            .margin(15)
            .x_label_area_size(45)
            .y_label_area_size(55)
            .build_cartesian_2d(x_min..x_max, 0.0..y_max)
            .context("创建图表失败")?;

        // 网格
        chart
            .configure_mesh()
            .x_labels(data.len().min(31))
            .x_label_formatter(&|&v| {
                let i = v.round() as usize;
                if i < labels.len() {
                    labels[i].clone()
                } else {
                    String::new()
                }
            })
            .y_label_formatter(&|v| format!("{}", *v as i64))
            .x_desc(x_label)
            .y_desc(y_label)
            .axis_desc_style((font_name, 16))
            .label_style((font_name, 13))
            .light_line_style(&RGBColor(0xE0, 0xE0, 0xE0))
            .bold_line_style(&WHITE.mix(0.0))
            .draw()?;

        // 柱状图
        let bar_width = 0.3f64;
        chart
            .draw_series(data.iter().enumerate().map(|(i, (_date, count))| {
                let x = i as f64;
                Rectangle::new(
                    [(x - bar_width, 0.0), (x + bar_width, *count as f64)],
                    bar_color.filled(),
                )
            }))?
            .label("Bar")
            .legend(|(x, y)| {
                Rectangle::new([(x, y - 5), (x + 10, y + 5)], bar_color.filled())
            });

        // 折线
        chart
            .draw_series(LineSeries::new(
                data.iter()
                    .enumerate()
                    .map(|(i, (_, count))| (i as f64, *count as f64)),
                line_style.clone(),
            ))?
            .label("Line")
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], LINE_COLOR.stroke_width(2))
            });

        // 数据点
        chart
            .draw_series(data.iter().enumerate().map(|(i, (_, count))| {
                Circle::new((i as f64, *count as f64), 4, ACCENT_COLOR.filled())
            }))
            .context("绘制数据点失败")?;

        // 数值标签
        let label_font = (font_name, 11).into_font().color(&BLACK.mix(0.6));
        for (i, (_, count)) in data.iter().enumerate() {
            if *count > 0 {
                chart
                    .draw_series(std::iter::once(Text::new(
                        format!("{}", count),
                        (i as f64, *count as f64 + y_max * 0.03),
                        label_font.clone(),
                    )))
                    .context("绘制数值标签失败")?;
            }
        }

        // 图例
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK.mix(0.2))
            .label_font((font_name, 13))
            .position(SeriesLabelPosition::UpperLeft)
            .draw()
            .context("绘制图例失败")?;

        root.present().context("无法完成图表渲染")?;
    } // root, chart 在此被 drop，释放对 rgb_buf 的可变借用

    // 将 RGB 转换为 RGBA
    let pixel_count = (width * height) as usize;
    let mut rgba_buf = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        rgba_buf[i * 4] = rgb_buf[i * 3];
        rgba_buf[i * 4 + 1] = rgb_buf[i * 3 + 1];
        rgba_buf[i * 4 + 2] = rgb_buf[i * 3 + 2];
        rgba_buf[i * 4 + 3] = 255;
    }

    Ok(rgba_buf)
}
