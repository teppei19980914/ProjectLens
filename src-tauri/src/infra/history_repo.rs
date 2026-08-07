//! 解析履歴のSQLite永続化（03_設計書.md §3.2 analysis_history）。

use crate::infra::db::SharedConn;
use crate::models::constants::tables;
use crate::models::{AnalysisHistoryEntry, AppError, AppResult};

pub struct HistoryRepo {
    conn: SharedConn,
}

pub struct NewHistoryEntry {
    pub project_path: String,
    pub status: String,
    pub total_files: u64,
    pub analyzed_files: u64,
    pub cache_hits: u64,
    pub total_tokens: u64,
    pub duration_ms: u64,
    pub error_summary: Option<String>,
    pub started_at: String,
    pub finished_at: String,
}

impl HistoryRepo {
    pub fn new(conn: SharedConn) -> Self {
        Self { conn }
    }

    pub fn record(&self, entry: NewHistoryEntry) -> AppResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            &format!(
                "INSERT INTO {} (project_path, status, total_files, analyzed_files, cache_hits, total_tokens, duration_ms, error_summary, started_at, finished_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                tables::ANALYSIS_HISTORY
            ),
            rusqlite::params![
                entry.project_path,
                entry.status,
                entry.total_files,
                entry.analyzed_files,
                entry.cache_hits,
                entry.total_tokens,
                entry.duration_ms,
                entry.error_summary,
                entry.started_at,
                entry.finished_at,
            ],
        )
        .map_err(|e| AppError::cache(format!("解析履歴の記録に失敗しました: {e}")))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list(&self, limit: u32) -> AppResult<Vec<AnalysisHistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT id, project_path, status, total_files, analyzed_files, cache_hits, total_tokens, duration_ms, error_summary, started_at, finished_at
                 FROM {} ORDER BY started_at DESC LIMIT ?1",
                tables::ANALYSIS_HISTORY
            ))
            .map_err(|e| AppError::cache(format!("解析履歴の取得に失敗しました: {e}")))?;

        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok(AnalysisHistoryEntry {
                    id: row.get(0)?,
                    project_path: row.get(1)?,
                    status: row.get(2)?,
                    total_files: row.get(3)?,
                    analyzed_files: row.get(4)?,
                    cache_hits: row.get(5)?,
                    total_tokens: row.get(6)?,
                    duration_ms: row.get(7)?,
                    error_summary: row.get(8)?,
                    started_at: row.get(9)?,
                    finished_at: row.get(10)?,
                })
            })
            .map_err(|e| AppError::cache(format!("解析履歴の取得に失敗しました: {e}")))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| AppError::cache(format!("解析履歴の取得に失敗しました: {e}")))
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            &format!("DELETE FROM {} WHERE id = ?1", tables::ANALYSIS_HISTORY),
            rusqlite::params![id],
        )
        .map_err(|e| AppError::cache(format!("解析履歴の削除に失敗しました: {e}")))?;
        Ok(())
    }
}
