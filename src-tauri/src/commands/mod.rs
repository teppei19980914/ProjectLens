pub mod analysis;
pub mod apikey;
pub mod cache;
pub mod config;
pub mod export;
pub mod history;

use crate::models::AppError;
use tauri_plugin_dialog::DialogExt;

/// フォルダ選択ダイアログを表示し、選択結果を返す（analysis/export 両コマンドで共用、CLAUDE.md 原則2.2.1）。
/// `err` は取得失敗時のエラーをカテゴリ別に組み立てるコンストラクタ（例: `AppError::scan` / `AppError::export`）。
pub async fn pick_folder(
    app: &tauri::AppHandle,
    err: impl FnOnce(String) -> AppError,
) -> Result<Option<String>, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder);
    });
    let selected = rx
        .await
        .map_err(|_| err("フォルダ選択ダイアログの応答を取得できませんでした".to_string()))?;
    Ok(selected.map(|p| p.to_string()))
}
