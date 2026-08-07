//! アプリ設定型（config.json）。デフォルト値は本ファイルの `Default` 実装1箇所にのみ定義する
//! （CLAUDE.md 原則2.1.2）。リセット機能もこの `Default::default()` を参照すること。
//! 値は04_実装詳細.md §4「config.json デフォルト値（性能重視・確定版）」に準拠する。

use crate::models::constants;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub scan: ScanConfig,
    pub ai: AiConfig,
    pub cache: CacheConfig,
    pub export: ExportConfig,
    pub ui: UiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            scan: ScanConfig::default(),
            ai: AiConfig::default(),
            cache: CacheConfig::default(),
            export: ExportConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    pub exclude_patterns: Vec<String>,
    pub max_file_size_kb: u64,
    pub max_file_count: u64,
    pub extensions: Vec<String>,
    pub rpa: RpaScanConfig,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            exclude_patterns: vec![
                "node_modules".into(),
                "target".into(),
                "dist".into(),
                "build".into(),
                ".git".into(),
                "vendor".into(),
                "__pycache__".into(),
            ],
            max_file_size_kb: 1024,
            max_file_count: 5000,
            extensions: vec![
                ".ts".into(),
                ".tsx".into(),
                ".js".into(),
                ".jsx".into(),
                ".rs".into(),
                ".py".into(),
                ".java".into(),
                ".go".into(),
                ".cs".into(),
                ".c".into(),
                ".cpp".into(),
                ".h".into(),
                ".hpp".into(),
            ],
            rpa: RpaScanConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpaScanConfig {
    pub enabled: bool,
    pub tools: Vec<String>,
    pub detection_patterns: RpaDetectionPatterns,
    pub max_file_size_kb: u64,
}

impl Default for RpaScanConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tools: vec![
                constants::rpa_tools::POWER_PLATFORM.into(),
                constants::rpa_tools::PAD.into(),
                constants::rpa_tools::UIPATH.into(),
            ],
            detection_patterns: RpaDetectionPatterns::default(),
            max_file_size_kb: 4096,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpaDetectionPatterns {
    pub powerplatform: Vec<String>,
    pub pad: Vec<String>,
    pub uipath: Vec<String>,
}

impl Default for RpaDetectionPatterns {
    fn default() -> Self {
        Self {
            powerplatform: vec![
                "solution.xml".into(),
                "customizations.xml".into(),
                "Workflows/*.json".into(),
                "*.msapp".into(),
                "CanvasApps/**/*.json".into(),
            ],
            pad: vec!["*.robin".into(), "*.pad.txt".into()],
            uipath: vec!["project.json".into(), "*.xaml".into()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub provider: String,
    pub newtonx: NewtonxConfig,
    pub concurrency: u32,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub retry_after_secs: u64,
    pub retry_after_max_secs: u64,
    pub detail_level: String,
    pub max_tokens_per_file: u32,
    pub output_language: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "newtonx".into(),
            newtonx: NewtonxConfig::default(),
            concurrency: 5,
            timeout_secs: 90,
            max_retries: 3,
            retry_after_secs: 60,
            retry_after_max_secs: 300,
            detail_level: "standard".into(),
            max_tokens_per_file: 8000,
            output_language: "ja".into(),
        }
    }
}

/// NewtonX固有設定。PAT本体はここに含めない（OS資格情報ストアに保存。04_実装詳細.md §3.4）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewtonxConfig {
    pub host: String,
    pub assistant_uid: String,
}

impl Default for NewtonxConfig {
    fn default() -> Self {
        Self {
            host: "seraku.newton-x.net".into(),
            // GPT-5.4(高性能)。選定理由は 04_実装詳細.md §3.3 参照。
            assistant_uid: "d18ad1c0-c7e6-4651-9ff2-4fe86af1a73b".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_days: u32,
    pub max_size_mb: u64,
    pub auto_cleanup: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_days: 30,
            max_size_mb: 500,
            auto_cleanup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    pub default_format: String,
    pub default_doc_types: Vec<String>,
    pub output_dir: String,
    pub embed_mermaid: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: "markdown".into(),
            default_doc_types: vec![
                constants::doc_types::SYSTEM_SPEC.into(),
                constants::doc_types::BASIC_DESIGN.into(),
                constants::doc_types::DETAIL_DESIGN.into(),
            ],
            output_dir: String::new(),
            embed_mermaid: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
    pub recent_projects_count: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: "ja".into(),
            recent_projects_count: 10,
        }
    }
}
