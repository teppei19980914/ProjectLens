//! エクスポートコマンド（04_実装詳細.md §7/§11）。直近の解析結果を元に再出力する。

use crate::commands::pick_folder;
use crate::domain::doc_generator;
use crate::models::{AppError, ExportResult};
use crate::state::AppState;

#[tauri::command]
pub async fn select_output_folder(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    pick_folder(&app, AppError::export).await
}

#[tauri::command]
pub async fn export_document(
    state: tauri::State<'_, AppState>,
    format: String,
    doc_types: Vec<String>,
    embed_mermaid: bool,
    output_dir: Option<String>,
) -> Result<ExportResult, AppError> {
    let guard = state.last_analysis.lock().await;
    let data = guard
        .as_ref()
        .ok_or_else(|| AppError::export("エクスポート対象の解析結果がありません。先に解析を実行してください。"))?;

    let resolved_dir = match output_dir {
        Some(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
        _ => doc_generator::resolve_output_dir(&data.project_path, ""),
    };

    doc_generator::generate(
        &resolved_dir,
        &doc_types,
        &format,
        embed_mermaid,
        &data.project_result,
        &data.static_results,
        &data.file_results,
        &data.rpa_components,
    )
}
