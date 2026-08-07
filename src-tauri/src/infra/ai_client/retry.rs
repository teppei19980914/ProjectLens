//! NewtonXリトライ戦略（04_実装詳細.md §3.5）。ファイル解析・全体解析・接続テストで共用する
//! （CLAUDE.md 原則2.2.1）。
//!
//! 注意: ADKの `APIError` は429（レート制限）と5xx/ネットワークエラーを判別できないことが多い
//! （実装確認済み。04_実装詳細.md §3.5脚注/§10残課題#3）。そのためここでは両者を区別せず、
//! `retryAfterSecs` を初期待機、`retryAfterMaxSecs` を上限とした指数バックオフ+ジッターに統一する。

use crate::models::config::AiConfig;
use crate::models::{AppError, AppResult};
use rand::Rng;
use std::time::Duration;

pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_wait_secs: u64,
    pub max_wait_secs: u64,
}

impl From<&AiConfig> for RetryPolicy {
    fn from(cfg: &AiConfig) -> Self {
        Self {
            max_retries: cfg.max_retries,
            base_wait_secs: cfg.retry_after_secs,
            max_wait_secs: cfg.retry_after_max_secs,
        }
    }
}

impl RetryPolicy {
    fn backoff(&self, attempt: u32) -> Duration {
        let exp = self.base_wait_secs.saturating_mul(1u64 << attempt.saturating_sub(1).min(16));
        let capped = exp.min(self.max_wait_secs).max(1);
        let jitter = rand::thread_rng().gen_range(-0.2..=0.2);
        let secs = (capped as f64) * (1.0 + jitter);
        Duration::from_secs_f64(secs.max(0.1))
    }
}

/// `operation` を実行し、`retryable` なAiエラーの場合のみ指数バックオフ+ジッターで再試行する。
/// `recoverable=false`（認証切れ・チャットエラー・設定エラー）は即座に呼び出し元へ伝播する。
pub async fn with_retry<T, F, Fut>(policy: &RetryPolicy, mut operation: F) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    let mut attempt = 0u32;
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(AppError::Ai { retryable: true, .. }) if attempt < policy.max_retries => {
                attempt += 1;
                tokio::time::sleep(policy.backoff(attempt)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
