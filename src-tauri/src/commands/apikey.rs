//! NewtonX認証・接続コマンド（PAT方式。04_実装詳細.md §3.4/§7）。

use crate::infra::ai_client::AssistantInfo;
use crate::infra::credential_store::CredentialStore;
use crate::models::AppError;
use crate::state::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
}

#[derive(Serialize)]
pub struct SaveCredentialsResponse {
    pub success: bool,
}

#[derive(Serialize)]
pub struct TestConnectionResponse {
    pub ok: bool,
    pub message: String,
}

#[tauri::command]
pub async fn newtonx_auth_status(state: tauri::State<'_, AppState>) -> Result<AuthStatusResponse, AppError> {
    let client = state.spawn_ai_client().await?;
    let authenticated = client.auth_status().await?;
    Ok(AuthStatusResponse { authenticated })
}

/// PATをOS資格情報ストアへ保存し、サイドカー側にも反映する（04_実装詳細.md §3.4）。
#[tauri::command]
pub async fn save_newtonx_credentials(
    state: tauri::State<'_, AppState>,
    host: String,
    personal_access_token: String,
) -> Result<SaveCredentialsResponse, AppError> {
    CredentialStore::save_pat(&personal_access_token)?;

    let mut config = state.config_store.load()?;
    config.ai.newtonx.host = host.clone();
    state.config_store.save(&config)?;

    let client = state.spawn_ai_client().await?;
    client.save_credentials(&host, &personal_access_token).await?;
    Ok(SaveCredentialsResponse { success: true })
}

/// PATをOS資格情報ストア・サイドカー双方から削除する（04_実装詳細.md §3.4）。
#[tauri::command]
pub async fn clear_newtonx_credentials(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let client = state.spawn_ai_client().await?;
    client.clear_credentials().await?;
    CredentialStore::clear_pat()?;
    Ok(())
}

#[tauri::command]
pub async fn newtonx_list_assistants(state: tauri::State<'_, AppState>) -> Result<Vec<AssistantInfo>, AppError> {
    let client = state.spawn_ai_client().await?;
    client.list_assistants().await
}

#[tauri::command]
pub async fn test_ai_connection(state: tauri::State<'_, AppState>) -> Result<TestConnectionResponse, AppError> {
    let client = state.spawn_ai_client().await?;
    match client.test_connection().await {
        Ok(message) => Ok(TestConnectionResponse { ok: true, message }),
        Err(e) => Ok(TestConnectionResponse {
            ok: false,
            message: e.to_string(),
        }),
    }
}
