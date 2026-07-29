//! 图表生成模块
//!
//! 使用 plotters 库读取 SQLite 数据，生成每日字数的折线+柱状叠加图，
//! 输出为 PNG 图片。

use anyhow::{Context, Result};
use plotters::prelude::*;

use crate::db;

/// 色调：Rime 风格的暖橙色系
const BAR_COLOR: RGBColor = RGBColor(0xE8, 0x6C, 0x00); // 橙色柱体
const LINE_COLOR: RGBColor = RGBColor(0xC0, 0x39, 0x2B); // 深红色折线
const ACCENT_COLOR: RGBColor = RGBColor(0xF3, 0x9C, 0x12); // 金色数据点

/// 生成字数统计图表，保存为 PNG 文件。
///
/// * `db_path` — SQLite 数据库路径
/// * `output_path` — 输出的 PNG 图片路径
/// * `days` — 图表涵盖的天数（最近 N 天）
pub fn generate_chart(db_path: &str, output_path: &str, days: i64) -> Result<()> {
    let conn = db::init_db(db_path)?;
    let data = db::query_recent_days(&conn, days)?;

    if data.is_empty() {
        anyhow::bail!(
            "数据库中没有字数数据，无法生成图表。请先运行 --process 处理日志文件。"
        );
    }

    println!(
        "[INFO] 图表将展示 {} 天的数据（{} → {}）",
        data.len(),
        data.first().unwrap().0,
        data.last().unwrap().0
    );

    // 确定图表尺寸
    let count = data.len();
    let chart_width = 1200.max(count * 60); // 自适应宽度，每个数据点至少 60px
    let chart_height = 600;

    let root = BitMapBackend::new(output_path, (chart_width as u32, chart_height as u32))
        .into_drawing_area();
    root.fill(&WHITE)
        .with_context(|| "无法填充图表背景")?;

    // 计算 Y 轴范围
    let max_count = data.iter().map(|(_, c)| *c).max().unwrap_or(1) as f64;
    let y_max = (max_count * 1.2).ceil().max(10.0); // 顶部留 20% 空间

    // 颜色定义（须在 chart 之前，确保生命周期足够长）
    let bar_color = BAR_COLOR.mix(0.7);

    // X 轴标签使用日期字符串
    let labels: Vec<String> = data.iter().map(|(d, _)| d.clone()).collect();
    // 使用 f64 范围，把每个数据点映射到整数位置 [0, 1, 2, ..., count-1]
    // 这样柱状图的宽度可以精确控制
    let x_min = -0.5f64;
    let x_max = (count - 1) as f64 + 0.5;

    let mut chart = ChartBuilder::on(&root)
        .caption("每日输入字数统计", ("sans-serif", 32).into_font())
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, 0.0..y_max)
        .context("创建图表失败")?;

    // 绘制网格
    chart
        .configure_mesh()
        .x_labels(data.len().min(31)) // 最多显示 31 个日期标签
        .x_label_formatter(&|&v| {
            let i = v.round() as usize;
            if i < labels.len() {
                // 简化日期显示：MM-DD
                let parts: Vec<&str> = labels[i].split('-').collect();
                if parts.len() == 3 {
                    format!("{}-{}", parts[1], parts[2])
                } else {
                    labels[i].clone()
                }
            } else {
                String::new()
            }
        })
        .y_label_formatter(&|v| format!("{}", *v as i64))
        .x_desc("日期")
        .y_desc("字数")
        .axis_desc_style(("sans-serif", 18))
        .label_style(("sans-serif", 14))
        .light_line_style(&RGBColor(0xE0, 0xE0, 0xE0))
        .bold_line_style(&WHITE.mix(0.0))
        .draw()?;

    // —— 绘制柱状图 ——
    // 使用 Rectangle 手动绘制柱状，宽度为 0.6
    let bar_width = 0.3f64;
    chart
        .draw_series(data.iter().enumerate().map(|(i, (_date, count))| {
            let x = i as f64;
            Rectangle::new(
                [(x - bar_width, 0.0), (x + bar_width, *count as f64)],
                bar_color.filled(),
            )
        }))
        .context("绘制柱状图失败")?
        .label("每日字数")
        .legend(|(x, y)| {
            Rectangle::new([(x, y - 5), (x + 10, y + 5)], bar_color.filled())
        });

    // —— 绘制折线趋势 ——
    let line_style = LINE_COLOR.stroke_width(2);
    chart
        .draw_series(LineSeries::new(
            data.iter().enumerate().map(|(i, (_date, count))| {
                (i as f64, *count as f64)
            }),
            line_style,
        ))
        .context("绘制折线失败")?
        .label("趋势线")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                LINE_COLOR.stroke_width(2),
            )
        });

    // —— 绘制数据点（小圆点）——
    chart
        .draw_series(data.iter().enumerate().map(|(i, (_date, count))| {
            Circle::new((i as f64, *count as f64), 4, ACCENT_COLOR.filled())
        }))
        .context("绘制数据点失败")?;

    // —— 在每个柱子上方标注具体数值 ——
    let label_font = ("sans-serif", 12).into_font().color(&BLACK.mix(0.6));
    for (i, (_date, count)) in data.iter().enumerate() {
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

    // —— 绘制图例 ——
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK.mix(0.2))
        .label_font(("sans-serif", 14))
        .position(SeriesLabelPosition::UpperLeft)
        .draw()
        .context("绘制图例失败")?;

    root.present().context("无法保存图表文件")?;
    println!("[INFO] 图表已保存至: {output_path}");

    Ok(())
}
