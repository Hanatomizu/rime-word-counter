//! 语言探测模块
//!
//! 界面文案由前端维护（`ui/src/i18n/strings.ts`），Rust 侧只负责
//! 探测系统语言，并通过 Tauri 命令告诉前端首次启动时用哪种语言，
//! 避免同一份文案在两端各写一遍。

use serde::Serialize;

/// 支持的语言
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum Language {
    /// 简体中文
    #[serde(rename = "zh-CN")]
    ZhCN,
    /// 繁体中文
    #[serde(rename = "zh-TW")]
    ZhTW,
    /// 英文
    #[serde(rename = "en")]
    En,
}

impl Language {
    /// BCP-47 语言代码，与前端 `strings.ts` 的 key 一致。
    pub fn code(self) -> &'static str {
        match self {
            Language::ZhCN => "zh-CN",
            Language::ZhTW => "zh-TW",
            Language::En => "en",
        }
    }
}

/// 把 Linux/macOS 的 locale 字符串归类到支持的语言。
///
/// 覆盖 `zh_CN.UTF-8`、`zh-Hans-CN`、`zh_TW`、`zh-Hant` 等常见写法。
fn classify(locale: &str) -> Language {
    let normalized = locale.to_ascii_lowercase().replace('_', "-");

    if normalized.starts_with("zh") {
        if normalized.contains("hant")
            || normalized.contains("tw")
            || normalized.contains("hk")
            || normalized.contains("mo")
        {
            return Language::ZhTW;
        }
        return Language::ZhCN;
    }

    // 其它语言一律回退到英文
    Language::En
}

/// 探测系统语言。
///
/// 依次检查 `LC_ALL`、`LC_MESSAGES`、`LANG`、`LANGUAGE`，
/// 全部缺失或无法识别时回退到英文。
pub fn detect_language() -> Language {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        if let Ok(value) = std::env::var(key) {
            if !value.trim().is_empty() {
                return classify(&value);
            }
        }
    }
    Language::En
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simplified() {
        assert_eq!(classify("zh_CN.UTF-8"), Language::ZhCN);
        assert_eq!(classify("zh-Hans-CN"), Language::ZhCN);
        assert_eq!(classify("zh_SG.utf8"), Language::ZhCN);
        assert_eq!(classify("zh"), Language::ZhCN);
    }

    #[test]
    fn test_classify_traditional() {
        assert_eq!(classify("zh_TW.UTF-8"), Language::ZhTW);
        assert_eq!(classify("zh_HK.UTF-8"), Language::ZhTW);
        assert_eq!(classify("zh-Hant-TW"), Language::ZhTW);
    }

    #[test]
    fn test_classify_other_languages() {
        assert_eq!(classify("en_US.UTF-8"), Language::En);
        assert_eq!(classify("ja_JP.UTF-8"), Language::En);
        assert_eq!(classify("C"), Language::En);
        assert_eq!(classify(""), Language::En);
    }

    #[test]
    fn test_language_codes_match_frontend_keys() {
        assert_eq!(Language::ZhCN.code(), "zh-CN");
        assert_eq!(Language::ZhTW.code(), "zh-TW");
        assert_eq!(Language::En.code(), "en");
    }
}
