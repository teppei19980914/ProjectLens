//! config.json の読込/保存/リセット（03_設計書.md §2.2、CLAUDE.md 原則2.1.2）。
//! デフォルト値は `models::config::AppConfig` の `Default` 実装1箇所のみを参照する。

use crate::models::{AppConfig, AppError, AppResult};
use std::path::{Path, PathBuf};

pub struct ConfigStore {
    file_path: PathBuf,
}

impl ConfigStore {
    /// `config_dir` はOS標準のアプリ設定ディレクトリ（例: `%APPDATA%/ProjectLens/`）。
    pub fn new(config_dir: impl AsRef<Path>) -> Self {
        Self {
            file_path: config_dir.as_ref().join("config.json"),
        }
    }

    /// 設定を読み込む。ファイルが存在しない場合は既定値を書き込んで返す。
    pub fn load(&self) -> AppResult<AppConfig> {
        if !self.file_path.exists() {
            let default = AppConfig::default();
            self.save(&default)?;
            return Ok(default);
        }

        let raw = std::fs::read_to_string(&self.file_path)
            .map_err(|e| AppError::config(format!("config.jsonの読込に失敗しました: {e}")))?;

        serde_json::from_str(&raw)
            .map_err(|e| AppError::config(format!("config.jsonの形式が不正です: {e}")))
    }

    /// 設定を保存する。
    pub fn save(&self, config: &AppConfig) -> AppResult<()> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::config(format!("設定ディレクトリの作成に失敗しました: {e}")))?;
        }

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::config(format!("設定のシリアライズに失敗しました: {e}")))?;

        std::fs::write(&self.file_path, json)
            .map_err(|e| AppError::config(format!("config.jsonの書き込みに失敗しました: {e}")))
    }

    /// 既定値にリセットして保存し、その値を返す。
    pub fn reset(&self) -> AppResult<AppConfig> {
        let default = AppConfig::default();
        self.save(&default)?;
        Ok(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_creates_default_when_missing() {
        let dir = tempfile_dir();
        let store = ConfigStore::new(&dir);
        let cfg = store.load().unwrap();
        assert_eq!(cfg.ai.concurrency, 5);
        assert!(dir.join("config.json").exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reset_restores_default_after_mutation() {
        let dir = tempfile_dir();
        let store = ConfigStore::new(&dir);
        let mut cfg = store.load().unwrap();
        cfg.ai.concurrency = 1;
        store.save(&cfg).unwrap();
        assert_eq!(store.load().unwrap().ai.concurrency, 1);

        let reset = store.reset().unwrap();
        assert_eq!(reset.ai.concurrency, 5);
        assert_eq!(store.load().unwrap().ai.concurrency, 5);
        std::fs::remove_dir_all(&dir).ok();
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("projectlens_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
