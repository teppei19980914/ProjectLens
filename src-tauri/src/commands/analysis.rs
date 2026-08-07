//! 解析コマンド（04_実装詳細.md §7）。

use crate::domain::orchestrator::Orchestrator;
use crate::models::{AnalysisResultSnapshot, AnalysisSummary, AppError};
use crate::state::AppState;
use tauri_plugin_dialog::DialogExt;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn select_project_folder(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder);
    });
    let selected = rx
        .await
        .map_err(|_| AppError::scan("フォルダ選択ダイアログの応答を取得できませんでした"))?;
    Ok(selected.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn start_full_analysis(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_path: String,
) -> Result<AnalysisSummary, AppError> {
    let cancel = CancellationToken::new();
    {
        let mut guard = state.active_cancel_token.lock().await;
        *guard = Some(cancel.clone());
    }

    let ai_client = state.spawn_ai_client().await?;
    let config = state.config_store.load()?;

    let orchestrator = Orchestrator {
        ai_client,
        cache_repo: state.cache_repo.clone(),
        history_repo: state.history_repo.clone(),
    };

    let result = orchestrator
        .run_full_analysis(app, std::path::Path::new(&project_path), &config, cancel)
        .await;

    {
        let mut guard = state.active_cancel_token.lock().await;
        *guard = None;
    }

    match result {
        Ok(output) => {
            let mut guard = state.last_analysis.lock().await;
            *guard = Some(crate::state::LastAnalysisData {
                project_result: output.project_result,
                static_results: output.static_results,
                file_results: output.file_results,
                rpa_components: output.rpa_components,
                project_path: std::path::PathBuf::from(&project_path),
            });
            Ok(output.summary)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn get_last_analysis_result(state: tauri::State<'_, AppState>) -> Result<Option<AnalysisResultSnapshot>, AppError> {
    let guard = state.last_analysis.lock().await;
    Ok(guard.as_ref().map(|data| AnalysisResultSnapshot {
        project_result: data.project_result.clone(),
        file_results: data.file_results.clone(),
    }))
}

#[tauri::command]
pub async fn cancel_analysis(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let guard = state.active_cancel_token.lock().await;
    if let Some(token) = guard.as_ref() {
        token.cancel();
    }
    Ok(())
}
