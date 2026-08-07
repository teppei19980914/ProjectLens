//! キャッシュコマンド（04_実装詳細.md §7）。

use crate::models::{AppError, CacheStats};
use crate::state::AppState;

#[tauri::command]
pub async fn get_cache_stats(state: tauri::State<'_, AppState>) -> Result<CacheStats, AppError> {
    state.cache_repo.stats()
}

#[tauri::command]
pub async fn clear_cache(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    state.cache_repo.clear_all()
}
