//! Tauriアプリ状態（03_設計書.md §2.1 commands層が参照する共有状態）。

use crate::infra::ai_client::bridge::NewtonXBridge;
use crate::infra::ai_client::AiClient;
use crate::infra::cache_repo::CacheRepo;
use crate::infra::config_store::ConfigStore;
use crate::infra::history_repo::HistoryRepo;
use crate::models::{AppError, AppResult, FileAnalysisResult, ProjectAnalysisResult, RpaComponent, ScannedFile, StaticAnalysisResult, VbaComponent};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// 直近の解析結果（エクスポート画面からの再出力用に保持する。プロセス終了で失われる簡易実装）。
pub struct LastAnalysisData {
    pub project_result: ProjectAnalysisResult,
    pub static_results: Vec<StaticAnalysisResult>,
    pub file_results: Vec<FileAnalysisResult>,
    pub rpa_components: Vec<RpaComponent>,
    pub vba_components: Vec<VbaComponent>,
    pub scanned_files: Vec<ScannedFile>,
    pub project_path: PathBuf,
}

pub struct AppState {
    pub config_store: ConfigStore,
    pub cache_repo: Arc<CacheRepo>,
    pub history_repo: Arc<HistoryRepo>,
    pub sidecar_script_path: PathBuf,
    pub sidecar_config_path: PathBuf,
    pub active_cancel_token: Mutex<Option<CancellationToken>>,
    pub last_analysis: Mutex<Option<LastAnalysisData>>,
}

impl AppState {
    /// 現在の設定でNewtonXサイドカーを起動する。
    /// 開発時は毎回プロセスを起動する簡易実装（残課題: 常駐化はサイドカー配布方式確定後に検討）。
    pub async fn spawn_ai_client(&self) -> AppResult<Arc<dyn AiClient>> {
        let config = self.config_store.load()?;
        let bridge = NewtonXBridge::spawn(
            &config.ai.python_exe,
            &self.sidecar_script_path,
            &self.sidecar_config_path,
            config.ai.newtonx.assistant_uid,
            config.ai.timeout_secs,
        )
        .await
            .map_err(|e| AppError::Ai {
                message: format!("NewtonXサイドカーの起動に失敗しました: {e}"),
                recoverable: false,
                retryable: false,
            })?;
        Ok(Arc::new(bridge))
    }
}
