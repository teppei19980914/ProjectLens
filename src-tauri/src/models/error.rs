//! アプリ共通エラー型（03_設計書.md §7.1）。
//! エラー→通知方法のマッピングは本型 + フロント側1箇所（src/lib/errorMapping.ts）に集約する（CLAUDE.md 原則2.2.1）。

use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "category", rename_all = "lowercase")]
pub enum AppError {
    #[error("scan error: {message}")]
    Scan { message: String, recoverable: bool },

    #[error("static analysis error: {message}")]
    Static { message: String, recoverable: bool },

    #[error("ai error: {message}")]
    Ai {
        message: String,
        recoverable: bool,
        retryable: bool,
    },

    #[error("cache error: {message}")]
    Cache { message: String, recoverable: bool },

    #[error("export error: {message}")]
    Export { message: String, recoverable: bool },

    #[error("config error: {message}")]
    Config { message: String, recoverable: bool },
}

impl AppError {
    pub fn scan(message: impl Into<String>) -> Self {
        Self::Scan {
            message: message.into(),
            recoverable: true,
        }
    }

    pub fn static_analysis(message: impl Into<String>) -> Self {
        Self::Static {
            message: message.into(),
            recoverable: true,
        }
    }

    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
            recoverable: true,
        }
    }

    pub fn export(message: impl Into<String>) -> Self {
        Self::Export {
            message: message.into(),
            recoverable: false,
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            recoverable: false,
        }
    }
}

// Tauri commandの戻り値(Result<T, E>)にAppErrorを直接使うためSerializeのみで足りるが、
// 呼び出し側の利便性のため From<rusqlite::Error> 等はinfra層で個別に実装する（責務分離）。

pub type AppResult<T> = std::result::Result<T, AppError>;
