//! OS資格情報ストア（Windows Credential Manager）経由のPAT保存/取得/削除。
//! NewtonXはPersonal Access Token（PAT）方式のため、config.jsonには平文保存しない
//! （04_実装詳細.md §3.4、NFR-07）。

use crate::models::constants::credential_store as cred_consts;
use crate::models::{AppError, AppResult};
use keyring::Entry;

pub struct CredentialStore;

impl CredentialStore {
    fn entry() -> AppResult<Entry> {
        Entry::new(cred_consts::SERVICE_NAME, cred_consts::NEWTONX_PAT_ENTRY)
            .map_err(|e| AppError::config(format!("資格情報ストアへのアクセスに失敗しました: {e}")))
    }

    /// PATを保存する。
    pub fn save_pat(pat: &str) -> AppResult<()> {
        Self::entry()?
            .set_password(pat)
            .map_err(|e| AppError::config(format!("PATの保存に失敗しました: {e}")))
    }

    /// PATを取得する。未設定の場合は `Ok(None)` を返す。
    pub fn get_pat() -> AppResult<Option<String>> {
        match Self::entry()?.get_password() {
            Ok(pat) => Ok(Some(pat)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::config(format!("PATの取得に失敗しました: {e}"))),
        }
    }

    /// PATを削除する。未設定の場合は何もしない。
    pub fn clear_pat() -> AppResult<()> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::config(format!("PATの削除に失敗しました: {e}"))),
        }
    }
}
