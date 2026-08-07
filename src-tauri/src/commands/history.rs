//! 解析履歴コマンド（04_実装詳細.md §7）。

use crate::models::{AnalysisHistoryEntry, AppError};
use crate::state::AppState;

#[tauri::command]
pub async fn get_analysis_history(state: tauri::State<'_, AppState>, limit: Option<u32>) -> Result<Vec<AnalysisHistoryEntry>, AppError> {
    state.history_repo.list(limit.unwrap_or(50))
}

#[tauri::command]
pub async fn delete_analysis_history(state: tauri::State<'_, AppState>, id: i64) -> Result<(), AppError> {
    state.history_repo.delete(id)
}
