pub mod commands;
pub mod domain;
pub mod infra;
pub mod models;
pub mod state;

use infra::cache_repo::CacheRepo;
use infra::history_repo::HistoryRepo;
use state::AppState;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle();
            let config_dir = app_handle
                .path()
                .app_config_dir()
                .expect("app_config_dir の取得に失敗しました");
            let data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("app_data_dir の取得に失敗しました");
            std::fs::create_dir_all(&config_dir).ok();
            std::fs::create_dir_all(&data_dir).ok();

            let config_store = infra::config_store::ConfigStore::new(&config_dir);
            let db_conn = infra::db::open_shared(data_dir.join("projectlens.sqlite3"))
                .expect("SQLiteの初期化に失敗しました");
            let cache_repo = std::sync::Arc::new(CacheRepo::new(db_conn.clone()));
            let history_repo = std::sync::Arc::new(HistoryRepo::new(db_conn));

            // 開発時: リポジトリ同梱の sidecar/newtonx_bridge.py を直接起動する。
            // 配布時: PyInstaller化したサイドカーバイナリに置き換える（04_実装詳細.md §10 #4）。
            let sidecar_script_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("sidecar")
                .join("newtonx_bridge.py");
            let sidecar_config_path = data_dir.join("newtonx_adk_config.json");

            app.manage(AppState {
                config_store,
                cache_repo,
                history_repo,
                sidecar_script_path,
                sidecar_config_path,
                python_exe: "python".to_string(),
                active_cancel_token: Mutex::new(None),
                last_analysis: Mutex::new(None),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analysis::select_project_folder,
            commands::analysis::start_full_analysis,
            commands::analysis::get_last_analysis_result,
            commands::analysis::cancel_analysis,
            commands::config::load_config,
            commands::config::save_config,
            commands::config::reset_config,
            commands::cache::get_cache_stats,
            commands::cache::clear_cache,
            commands::history::get_analysis_history,
            commands::history::delete_analysis_history,
            commands::apikey::newtonx_auth_status,
            commands::apikey::save_newtonx_credentials,
            commands::apikey::clear_newtonx_credentials,
            commands::apikey::newtonx_list_assistants,
            commands::apikey::test_ai_connection,
            commands::export::export_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
