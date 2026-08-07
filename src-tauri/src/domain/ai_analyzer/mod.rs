//! AI解析エンジン（03_設計書.md §2.2 AiAnalyzer）。
//! プロンプト構築・キャッシュ照会・AIリクエスト（並列はOrchestrator側のセマフォで制御）・
//! リトライ・応答バリデーションを担う。

pub mod prompts;
pub mod validator;

use crate::infra::ai_client::{retry, AiClient};
use crate::infra::cache_repo::CacheRepo;
use crate::models::config::AiConfig;
use crate::models::constants::{PROMPT_VERSION, RPA_PROMPT_VERSION};
use crate::models::{
    AppError, AppResult, DependencyGraph, FileAnalysisResult, Issue, ProjectAnalysisResult, RpaComponent,
    StaticAnalysisResult,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AiAnalyzer {
    client: Arc<dyn AiClient>,
    cache: Arc<CacheRepo>,
    ai_config: AiConfig,
    cache_ttl_days: u32,
    chat_uid: RwLock<String>,
}

impl AiAnalyzer {
    pub fn new(
        client: Arc<dyn AiClient>,
        cache: Arc<CacheRepo>,
        ai_config: AiConfig,
        cache_ttl_days: u32,
        initial_chat_uid: String,
    ) -> Self {
        Self {
            client,
            cache,
            ai_config,
            cache_ttl_days,
            chat_uid: RwLock::new(initial_chat_uid),
        }
    }

    pub async fn current_chat_uid(&self) -> String {
        self.chat_uid.read().await.clone()
    }

    /// ファイル単位AI解析（キャッシュ優先。04_実装詳細.md §5.1）。
    pub async fn analyze_file(
        &self,
        file_hash: &str,
        static_result: &StaticAnalysisResult,
        file_content: &str,
    ) -> AppResult<FileAnalysisResult> {
        let model_name = &self.ai_config.newtonx.assistant_uid;

        if let Some((json, _tokens)) = self.cache.get_file(file_hash, model_name, PROMPT_VERSION)? {
            if let Ok(mut result) = serde_json::from_str::<FileAnalysisResult>(&json) {
                result.cache_hit = true;
                return Ok(result);
            }
        }

        let prompt = prompts::file_analysis_prompt(&prompts::FileAnalysisInput {
            file_path: &static_result.file_path,
            language: &static_result.language,
            loc: static_result.metrics.loc,
            complexity: static_result.metrics.cyclomatic_complexity,
            functions: &static_result.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
            classes: &static_result.classes.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
            file_content,
        });

        match self.send_and_validate(&prompt, |raw| validator::parse_file_result(&static_result.file_path, raw)).await {
            Ok(mut result) => {
                result.cache_hit = false;
                let token_count = estimate_tokens(&prompt) + estimate_tokens(file_content);
                let json = serde_json::to_string(&result).unwrap_or_default();
                self.cache
                    .save_file(file_hash, &static_result.file_path, model_name, PROMPT_VERSION, &json, token_count, self.cache_ttl_days)
                    .ok(); // キャッシュ保存失敗は解析継続を妨げない（回復可能エラー）
                Ok(result)
            }
            Err(_) => Ok(fallback_file_result(static_result)),
        }
    }

    /// RPAコンポーネント単位AI解析（04_実装詳細.md §8.4。RPA_PROMPT_VERSIONで独立キャッシュ管理）。
    pub async fn analyze_rpa_component(&self, component_hash: &str, component: &RpaComponent, raw_content: &str) -> AppResult<FileAnalysisResult> {
        let model_name = &self.ai_config.newtonx.assistant_uid;

        if let Some((json, _)) = self.cache.get_file(component_hash, model_name, RPA_PROMPT_VERSION)? {
            if let Ok(mut result) = serde_json::from_str::<FileAnalysisResult>(&json) {
                result.cache_hit = true;
                return Ok(result);
            }
        }

        let intermediate_json = serde_json::to_string(component).unwrap_or_default();
        let prompt = prompts::rpa_analysis_prompt(&component.component_path, &component.tool, &intermediate_json, raw_content);

        match self.send_and_validate(&prompt, |raw| validator::parse_file_result(&component.component_path, raw)).await {
            Ok(mut result) => {
                result.cache_hit = false;
                let token_count = estimate_tokens(&prompt);
                let json = serde_json::to_string(&result).unwrap_or_default();
                self.cache
                    .save_file(component_hash, &component.component_path, model_name, RPA_PROMPT_VERSION, &json, token_count, self.cache_ttl_days)
                    .ok();
                Ok(result)
            }
            Err(_) => Ok(fallback_rpa_result(component)),
        }
    }

    /// プロジェクト全体AI解析（04_実装詳細.md §5.2）。
    pub async fn analyze_project(
        &self,
        project_hash: &str,
        project_root: &str,
        file_count: usize,
        language_stats: &str,
        dependency_graph: &DependencyGraph,
        file_summaries: &str,
    ) -> AppResult<ProjectAnalysisResult> {
        let model_name = &self.ai_config.newtonx.assistant_uid;

        if let Some((json, _)) = self.cache.get_project(project_hash, model_name, PROMPT_VERSION)? {
            if let Ok(result) = serde_json::from_str::<ProjectAnalysisResult>(&json) {
                return Ok(result);
            }
        }

        let dependency_summary = format!(
            "ノード数={}, エッジ数={}",
            dependency_graph.nodes.len(),
            dependency_graph.edges.len()
        );
        let cycles = if dependency_graph.cycles.is_empty() {
            "なし".to_string()
        } else {
            format!("{}件", dependency_graph.cycles.len())
        };

        let prompt = prompts::project_analysis_prompt(&prompts::ProjectAnalysisInput {
            project_root,
            file_count,
            language_stats,
            dependency_summary: &dependency_summary,
            cycles: &cycles,
            file_summaries,
        });

        let result = self.send_and_validate(&prompt, validator::parse_project_result).await?;
        let token_count = estimate_tokens(&prompt);
        let json = serde_json::to_string(&result).unwrap_or_default();
        self.cache
            .save_project(project_hash, model_name, PROMPT_VERSION, &json, token_count, self.cache_ttl_days)
            .ok();
        Ok(result)
    }

    /// 送信→検証を最大 `max_retries` 回試行する（応答不正時のリトライ。04_実装詳細.md §3.5）。
    async fn send_and_validate<T>(
        &self,
        prompt: &str,
        parse: impl Fn(&str) -> Result<T, AppError>,
    ) -> AppResult<T> {
        let mut last_err: Option<AppError> = None;
        for _ in 0..=self.ai_config.max_retries {
            let raw = self.send_message(prompt).await?;
            match parse(&raw) {
                Ok(value) => return Ok(value),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::Ai {
            message: "AI応答の検証に繰り返し失敗しました".into(),
            recoverable: true,
            retryable: false,
        }))
    }

    /// 1回分のメッセージ送信（429/5xx/タイムアウトは指数バックオフ、サイレントNoneはチャット再作成）。
    async fn send_message(&self, prompt: &str) -> AppResult<String> {
        let policy = retry::RetryPolicy::from(&self.ai_config);

        for _ in 0..=self.ai_config.max_retries {
            let chat_uid = self.chat_uid.read().await.clone();
            let client = self.client.clone();
            let prompt_owned = prompt.to_string();
            let response = retry::with_retry(&policy, || {
                let client = client.clone();
                let chat_uid = chat_uid.clone();
                let prompt = prompt_owned.clone();
                async move { client.analyze(&chat_uid, &prompt).await }
            })
            .await?;

            match response {
                Some(text) => return Ok(text),
                None => {
                    // send_messageがNoneを返すサイレント失敗はリトライではなくチャットを作り直す
                    // （skills/newtonx-adk/reference_response_reliability.md、04_実装詳細.md §3.5）
                    self.recreate_session().await?;
                }
            }
        }

        Err(AppError::Ai {
            message: "AIからの応答が繰り返し得られませんでした".into(),
            recoverable: true,
            retryable: false,
        })
    }

    async fn recreate_session(&self) -> AppResult<()> {
        let mut guard = self.chat_uid.write().await;
        let _ = self.client.close_session(&guard).await;
        let new_uid = self.client.open_session(&session_title()).await?;
        *guard = new_uid;
        Ok(())
    }
}

fn session_title() -> String {
    format!("解析 {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))
}

/// 粗い近似トークン数（1トークン≒4文字）。ADK側の正確なトークン数取得手段が未確定のため暫定。
fn estimate_tokens(text: &str) -> u32 {
    (text.chars().count() / 4).max(1) as u32
}

/// AI解析が最終的に失敗した場合の静的解析結果のみのフォールバック（NFR-06）。
fn fallback_file_result(static_result: &StaticAnalysisResult) -> FileAnalysisResult {
    FileAnalysisResult {
        file_path: static_result.file_path.clone(),
        role_summary: "AI解析に失敗したため、静的解析結果のみで生成しています。".into(),
        public_apis: Vec::new(),
        design_patterns: Vec::new(),
        importance_score: 5,
        potential_issues: vec![Issue {
            severity: crate::models::Severity::Medium,
            description: "AI解析が失敗またはタイムアウトしました。".into(),
            suggestion: "再解析を試すか、ファイルサイズ・トークン上限設定を見直してください。".into(),
        }],
        cache_hit: false,
    }
}

fn fallback_rpa_result(component: &RpaComponent) -> FileAnalysisResult {
    FileAnalysisResult {
        file_path: component.component_path.clone(),
        role_summary: format!(
            "AI解析に失敗したため、構造解析結果のみで生成しています（フロー名: {}）。",
            component.flow_name
        ),
        public_apis: Vec::new(),
        design_patterns: Vec::new(),
        importance_score: 5,
        potential_issues: vec![Issue {
            severity: crate::models::Severity::Medium,
            description: "AI解析が失敗またはタイムアウトしました。".into(),
            suggestion: "再解析を試してください。".into(),
        }],
        cache_hit: false,
    }
}
