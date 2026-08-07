//! 設定コマンド（04_実装詳細.md §7）。

use crate::models::{AppConfig, AppError};
use crate::state::AppState;

#[tauri::command]
pub async fn load_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, AppError> {
    state.config_store.load()
}

#[tauri::command]
pub async fn save_config(state: tauri::State<'_, AppState>, config: AppConfig) -> Result<(), AppError> {
    state.config_store.save(&config)
}

#[tauri::command]
pub async fn reset_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, AppError> {
    state.config_store.reset()
}
