//! AI解析結果のSQLiteキャッシュ（03_設計書.md §3.3、02_仕様書.md §4）。
//! キー生成・TTL判定はfile/projectキャッシュ両方で共通化する（CLAUDE.md 原則2.2.1）。

use crate::infra::db::SharedConn;
use crate::models::constants::tables;
use crate::models::{AppError, AppResult, CacheStats};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};

pub struct CacheRepo {
    conn: SharedConn,
}

impl CacheRepo {
    pub fn new(conn: SharedConn) -> Self {
        Self { conn }
    }

    /// キャッシュキー = SHA-256(hash + ":" + model_name + ":" + prompt_version)（03_設計書.md §3.3）
    pub fn cache_key(hash: &str, model_name: &str, prompt_version: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{hash}:{model_name}:{prompt_version}"));
        hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// ファイル単位キャッシュを照会する。TTL切れの場合はミス扱いで `None` を返す。
    pub fn get_file(&self, file_hash: &str, model_name: &str, prompt_version: &str) -> AppResult<Option<(String, u32)>> {
        let key = Self::cache_key(file_hash, model_name, prompt_version);
        let conn = self.conn.lock().unwrap();
        let result: rusqlite::Result<(String, u32, String)> = conn.query_row(
            &format!(
                "SELECT result_json, token_count, expires_at FROM {} WHERE cache_key = ?1",
                tables::FILE_ANALYSIS_CACHE
            ),
            rusqlite::params![key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((json, tokens, expires_at)) => {
                if is_expired(&expires_at) {
                    Ok(None)
                } else {
                    Ok(Some((json, tokens)))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::cache(format!("キャッシュ照会に失敗しました: {e}"))),
        }
    }

    pub fn save_file(
        &self,
        file_hash: &str,
        file_path: &str,
        model_name: &str,
        prompt_version: &str,
        result_json: &str,
        token_count: u32,
        ttl_days: u32,
    ) -> AppResult<()> {
        let key = Self::cache_key(file_hash, model_name, prompt_version);
        let expires_at = (Utc::now() + Duration::days(ttl_days as i64)).to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {} (cache_key, file_hash, file_path, model_name, prompt_version, result_json, token_count, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                tables::FILE_ANALYSIS_CACHE
            ),
            rusqlite::params![key, file_hash, file_path, model_name, prompt_version, result_json, token_count, expires_at],
        )
        .map_err(|e| AppError::cache(format!("キャッシュ保存に失敗しました: {e}")))?;
        Ok(())
    }

    pub fn get_project(&self, project_hash: &str, model_name: &str, prompt_version: &str) -> AppResult<Option<(String, u32)>> {
        let key = Self::cache_key(project_hash, model_name, prompt_version);
        let conn = self.conn.lock().unwrap();
        let result: rusqlite::Result<(String, u32, String)> = conn.query_row(
            &format!(
                "SELECT result_json, token_count, expires_at FROM {} WHERE cache_key = ?1",
                tables::PROJECT_ANALYSIS_CACHE
            ),
            rusqlite::params![key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((json, tokens, expires_at)) => {
                if is_expired(&expires_at) {
                    Ok(None)
                } else {
                    Ok(Some((json, tokens)))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::cache(format!("キャッシュ照会に失敗しました: {e}"))),
        }
    }

    pub fn save_project(
        &self,
        project_hash: &str,
        model_name: &str,
        prompt_version: &str,
        result_json: &str,
        token_count: u32,
        ttl_days: u32,
    ) -> AppResult<()> {
        let key = Self::cache_key(project_hash, model_name, prompt_version);
        let expires_at = (Utc::now() + Duration::days(ttl_days as i64)).to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {} (cache_key, project_hash, model_name, prompt_version, result_json, token_count, expires_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                tables::PROJECT_ANALYSIS_CACHE
            ),
            rusqlite::params![key, project_hash, model_name, prompt_version, result_json, token_count, expires_at],
        )
        .map_err(|e| AppError::cache(format!("キャッシュ保存に失敗しました: {e}")))?;
        Ok(())
    }

    /// プロンプトバージョン変更時、旧バージョンのキャッシュ行を削除する（03_設計書.md §3.3）。
    pub fn invalidate_old_prompt_versions(&self, current_prompt_version: &str) -> AppResult<u64> {
        let conn = self.conn.lock().unwrap();
        let mut total = 0u64;
        for table in [tables::FILE_ANALYSIS_CACHE, tables::PROJECT_ANALYSIS_CACHE] {
            let n = conn
                .execute(
                    &format!("DELETE FROM {table} WHERE prompt_version != ?1"),
                    rusqlite::params![current_prompt_version],
                )
                .map_err(|e| AppError::cache(format!("旧プロンプトキャッシュの削除に失敗しました: {e}")))?;
            total += n as u64;
        }
        Ok(total)
    }

    /// 起動時・定期実行での期限切れキャッシュ削除。
    pub fn cleanup_expired(&self) -> AppResult<u64> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        let mut total = 0u64;
        for table in [tables::FILE_ANALYSIS_CACHE, tables::PROJECT_ANALYSIS_CACHE] {
            let n = conn
                .execute(&format!("DELETE FROM {table} WHERE expires_at < ?1"), rusqlite::params![now])
                .map_err(|e| AppError::cache(format!("期限切れキャッシュの削除に失敗しました: {e}")))?;
            total += n as u64;
        }
        Ok(total)
    }

    /// サイズ上限超過時、created_at昇順で削除する（03_設計書.md §3.3）。
    pub fn enforce_size_limit(&self, max_size_mb: u64) -> AppResult<()> {
        loop {
            let stats = self.stats()?;
            if stats.size_mb <= max_size_mb as f64 {
                return Ok(());
            }
            let conn = self.conn.lock().unwrap();
            let deleted = conn
                .execute(
                    &format!(
                        "DELETE FROM {} WHERE cache_key = (SELECT cache_key FROM {} ORDER BY created_at ASC LIMIT 1)",
                        tables::FILE_ANALYSIS_CACHE,
                        tables::FILE_ANALYSIS_CACHE
                    ),
                    [],
                )
                .map_err(|e| AppError::cache(format!("サイズ上限超過時の削除に失敗しました: {e}")))?;
            if deleted == 0 {
                return Ok(()); // これ以上削除できるものがない
            }
        }
    }

    pub fn stats(&self) -> AppResult<CacheStats> {
        let conn = self.conn.lock().unwrap();
        let file_entries: u64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {}", tables::FILE_ANALYSIS_CACHE), [], |r| r.get(0))
            .map_err(|e| AppError::cache(format!("統計取得に失敗しました: {e}")))?;
        let project_entries: u64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {}", tables::PROJECT_ANALYSIS_CACHE), [], |r| r.get(0))
            .map_err(|e| AppError::cache(format!("統計取得に失敗しました: {e}")))?;

        let page_count: i64 = conn
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .map_err(|e| AppError::cache(format!("統計取得に失敗しました: {e}")))?;
        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |r| r.get(0))
            .map_err(|e| AppError::cache(format!("統計取得に失敗しました: {e}")))?;
        let size_mb = (page_count * page_size) as f64 / (1024.0 * 1024.0);

        Ok(CacheStats {
            file_entries,
            project_entries,
            size_mb,
        })
    }

    pub fn clear_all(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        for table in [tables::FILE_ANALYSIS_CACHE, tables::PROJECT_ANALYSIS_CACHE] {
            conn.execute(&format!("DELETE FROM {table}"), [])
                .map_err(|e| AppError::cache(format!("キャッシュ全削除に失敗しました: {e}")))?;
        }
        Ok(())
    }
}

fn is_expired(expires_at_rfc3339: &str) -> bool {
    match chrono::DateTime::parse_from_rfc3339(expires_at_rfc3339) {
        Ok(dt) => dt < Utc::now(),
        Err(_) => true, // 解釈できない場合は安全側に倒しキャッシュ無効として扱う
    }
}
