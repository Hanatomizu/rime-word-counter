//! 国际化模块
//!
//! 支持简体中文、繁体中文、英文三种语言。
//! 自动检测系统语言（LANG 环境变量），GUI 内可手动切换。

/// 支持的语言
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    /// 简体中文
    ZhCN,
    /// 繁体中文
    ZhTW,
    /// 英文
    En,
}

impl Language {
    /// 返回所有语言列表，用于 GUI 下拉菜单
    pub fn all() -> &'static [(Language, &'static str)] {
        &[
            (Language::ZhCN, "简体中文"),
            (Language::ZhTW, "繁體中文"),
            (Language::En, "English"),
        ]
    }
}

/// 从 `LANG` 环境变量检测系统语言。
/// 默认为英文（fallback）。
pub fn detect_language() -> Language {
    if let Ok(lang) = std::env::var("LANG") {
        if lang.starts_with("zh_CN") || lang.contains("zh-Hans") {
            return Language::ZhCN;
        }
        if lang.starts_with("zh_TW")
            || lang.starts_with("zh_HK")
            || lang.starts_with("zh_MO")
            || lang.contains("zh-Hant")
        {
            return Language::ZhTW;
        }
    }
    Language::En
}

/// 语言包结构体，存放所有 UI 文本
#[derive(Clone)]
#[allow(dead_code)]
pub struct Strings {
    // 窗口标题
    pub app_title: &'static str,
    // 总字数
    pub total_words: &'static str,
    // 总字数（带数值）
    pub total_words_fmt: &'static str,
    // 筛选面板
    pub filter_panel: &'static str,
    pub start_date: &'static str,
    pub end_date: &'static str,
    pub group_by: &'static str,
    pub daily: &'static str,
    pub monthly: &'static str,
    pub yearly: &'static str,
    // 图表区域
    pub chart_title: &'static str,
    pub chart_x_label: &'static str,
    pub chart_y_label: &'static str,
    // 统计信息
    pub stats_avg: &'static str,
    pub stats_max: &'static str,
    pub stats_min: &'static str,
    pub stats_days: &'static str,
    // 操作按钮
    pub reprocess: &'static str,
    pub reprocessing: &'static str,
    pub processing_log: &'static str,
    pub loading_data: &'static str,
    pub rendering_chart: &'static str,
    pub no_data: &'static str,
    // 语言
    pub language: &'static str,
    // 日期选择快捷
    pub last_7_days: &'static str,
    pub last_30_days: &'static str,
    pub last_year: &'static str,
    pub all_time: &'static str,
    // 错误
    pub error_prefix: &'static str,
}

impl Strings {
    /// 获取对应语言的字符串表
    pub fn for_language(lang: Language) -> Self {
        match lang {
            Language::ZhCN => Self::zh_cn(),
            Language::ZhTW => Self::zh_tw(),
            Language::En => Self::en(),
        }
    }

    fn zh_cn() -> Self {
        Strings {
            app_title: "Rime 字数统计",
            total_words: "总字数",
            total_words_fmt: "共 {} 字",
            filter_panel: "筛选条件",
            start_date: "开始日期",
            end_date: "结束日期",
            group_by: "分组依据",
            daily: "日",
            monthly: "月",
            yearly: "年",
            chart_title: "字数趋势",
            chart_x_label: "日期",
            chart_y_label: "字数",
            stats_avg: "平均 {} 字/日",
            stats_max: "最高 {} 字",
            stats_min: "最低 {} 字",
            stats_days: "共 {} 天数据",
            reprocess: "重新处理",
            reprocessing: "正在处理...",
            processing_log: "正在处理日志文件...",
            loading_data: "正在加载数据...",
            rendering_chart: "正在渲染图表...",
            no_data: "暂无数据，请先录入文字",
            language: "语言",
            last_7_days: "最近7天",
            last_30_days: "最近30天",
            last_year: "最近一年",
            all_time: "全部",
            error_prefix: "错误",
        }
    }

    fn zh_tw() -> Self {
        Strings {
            app_title: "Rime 字數統計",
            total_words: "總字數",
            total_words_fmt: "共 {} 字",
            filter_panel: "篩選條件",
            start_date: "開始日期",
            end_date: "結束日期",
            group_by: "分組依據",
            daily: "日",
            monthly: "月",
            yearly: "年",
            chart_title: "字數趨勢",
            chart_x_label: "日期",
            chart_y_label: "字數",
            stats_avg: "平均 {} 字/日",
            stats_max: "最高 {} 字",
            stats_min: "最低 {} 字",
            stats_days: "共 {} 天數據",
            reprocess: "重新處理",
            reprocessing: "正在處理...",
            processing_log: "正在處理日誌文件...",
            loading_data: "正在載入數據...",
            rendering_chart: "正在渲染圖表...",
            no_data: "暫無數據，請先錄入文字",
            language: "語言",
            last_7_days: "最近7天",
            last_30_days: "最近30天",
            last_year: "最近一年",
            all_time: "全部",
            error_prefix: "錯誤",
        }
    }

    fn en() -> Self {
        Strings {
            app_title: "Rime Word Counter",
            total_words: "Total Words",
            total_words_fmt: "{} words total",
            filter_panel: "Filters",
            start_date: "Start Date",
            end_date: "End Date",
            group_by: "Group By",
            daily: "Day",
            monthly: "Month",
            yearly: "Year",
            chart_title: "Word Count Trend",
            chart_x_label: "Date",
            chart_y_label: "Words",
            stats_avg: "Avg {} words/day",
            stats_max: "Max {} words",
            stats_min: "Min {} words",
            stats_days: "{} days of data",
            reprocess: "Reprocess",
            reprocessing: "Processing...",
            processing_log: "Processing log files...",
            loading_data: "Loading data...",
            rendering_chart: "Rendering chart...",
            no_data: "No data yet. Start typing with Rime!",
            language: "Language",
            last_7_days: "Last 7 Days",
            last_30_days: "Last 30 Days",
            last_year: "Last Year",
            all_time: "All Time",
            error_prefix: "Error",
        }
    }
}
