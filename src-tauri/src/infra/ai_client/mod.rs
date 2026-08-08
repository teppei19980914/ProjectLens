//! AIプロバイダ抽象化（03_設計書.md §1.2「プロバイダ抽象化」）。
//! 現状の実装はNewtonXのみだが、呼び出し側（AiAnalyzer）はこのtraitのみに依存する。

pub mod bridge;
pub mod retry;

use crate::models::AppResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantInfo {
    pub uid: String,
    pub name: String,
}

/// 接続テスト（ai.test）の結果。`ok` はRPC往復の成否ではなく、サイドカーが判定した
/// アプリケーションレベルの成否（04_実装詳細.md §3.2 `ai.test` → `get_model_status()`）を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub ok: bool,
    pub message: String,
}

#[async_trait]
pub trait AiClient: Send + Sync {
    /// 認証状態を確認する。
    async fn auth_status(&self) -> AppResult<bool>;

    /// Host/PATを反映する（04_実装詳細.md §3.4 auth.save_credentials）。
    async fn save_credentials(&self, host: &str, personal_access_token: &str) -> AppResult<()>;

    /// PATを消去する（04_実装詳細.md §3.4 auth.clear_credentials）。
    async fn clear_credentials(&self) -> AppResult<()>;

    /// 接続テスト（get_model_status）。
    async fn test_connection(&self) -> AppResult<ConnectionTestResult>;

    /// 選択可能なアシスタント一覧を取得する。
    async fn list_assistants(&self) -> AppResult<Vec<AssistantInfo>>;

    /// 解析セッション用チャットを開く。chat_uidを返す（04_実装詳細.md §3.3 session.open）。
    async fn open_session(&self, title: &str) -> AppResult<String>;

    /// 解析セッションを閉じる（04_実装詳細.md §3.3 session.close）。
    async fn close_session(&self, chat_uid: &str) -> AppResult<()>;

    /// 解析リクエストを送信し、応答テキストを返す（04_実装詳細.md §3.3 ai.analyze）。
    /// `web_search=False` / `knowledge_search=False` は実装側（bridge）で必ず明示指定する。
    async fn analyze(&self, chat_uid: &str, prompt: &str) -> AppResult<Option<String>>;
}
