//! SQLite接続・スキーマ初期化（03_設計書.md §3.2 DDL）。
//! マイグレーションは `db_metadata.schema_version` で管理する。

use crate::models::constants::{db_metadata_keys, tables, CURRENT_SCHEMA_VERSION};
use crate::models::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// rusqlite::Connection は Sync ではないため、複数コマンド・非同期タスク間で共有するには
/// Arc<Mutex<..>> でラップする（Orchestrator/各Repositoryで共用）。
pub type SharedConn = Arc<Mutex<Connection>>;

pub fn open_shared(db_path: impl AsRef<Path>) -> AppResult<SharedConn> {
    Ok(Arc::new(Mutex::new(open(db_path)?)))
}

pub fn open(db_path: impl AsRef<Path>) -> AppResult<Connection> {
    if let Some(parent) = db_path.as_ref().parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::cache(format!("DBディレクトリの作成に失敗しました: {e}")))?;
    }

    let conn = Connection::open(db_path.as_ref())
        .map_err(|e| AppError::cache(format!("DBのオープンに失敗しました: {e}")))?;

    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(&format!(
        "
        CREATE TABLE IF NOT EXISTS {file_cache} (
            cache_key       TEXT PRIMARY KEY,
            file_hash       TEXT NOT NULL,
            file_path       TEXT NOT NULL,
            model_name      TEXT NOT NULL,
            prompt_version  TEXT NOT NULL,
            result_json     TEXT NOT NULL,
            token_count     INTEGER NOT NULL DEFAULT 0,
            created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at      DATETIME NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_fac_expires ON {file_cache}(expires_at);
        CREATE INDEX IF NOT EXISTS idx_fac_prompt  ON {file_cache}(prompt_version);

        CREATE TABLE IF NOT EXISTS {project_cache} (
            cache_key       TEXT PRIMARY KEY,
            project_hash    TEXT NOT NULL,
            model_name      TEXT NOT NULL,
            prompt_version  TEXT NOT NULL,
            result_json     TEXT NOT NULL,
            token_count     INTEGER NOT NULL DEFAULT 0,
            created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at      DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS {history} (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            project_path    TEXT NOT NULL,
            status          TEXT NOT NULL,
            total_files     INTEGER NOT NULL DEFAULT 0,
            analyzed_files  INTEGER NOT NULL DEFAULT 0,
            cache_hits      INTEGER NOT NULL DEFAULT 0,
            total_tokens    INTEGER NOT NULL DEFAULT 0,
            duration_ms     INTEGER NOT NULL DEFAULT 0,
            error_summary   TEXT,
            started_at      DATETIME NOT NULL,
            finished_at     DATETIME
        );

        CREATE TABLE IF NOT EXISTS {metadata} (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
        file_cache = tables::FILE_ANALYSIS_CACHE,
        project_cache = tables::PROJECT_ANALYSIS_CACHE,
        history = tables::ANALYSIS_HISTORY,
        metadata = tables::DB_METADATA,
    ))
    .map_err(|e| AppError::cache(format!("スキーマ初期化に失敗しました: {e}")))?;

    conn.execute(
        &format!(
            "INSERT OR IGNORE INTO {} (key, value) VALUES (?1, ?2)",
            tables::DB_METADATA
        ),
        rusqlite::params![db_metadata_keys::SCHEMA_VERSION, CURRENT_SCHEMA_VERSION],
    )
    .map_err(|e| AppError::cache(format!("schema_versionの初期化に失敗しました: {e}")))?;

    Ok(())
}
