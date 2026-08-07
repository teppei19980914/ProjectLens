//! アプリ全体で共有する文字列定数の一元定義（CLAUDE.md 原則2.1.3）。
//! イベント名・フェーズ名・エラーカテゴリ・SQLテーブル名・既定パス・RPCメソッド名を集約する。
//! Rust側はこのファイル、フロント側は `src/lib/constants.ts` を単一の参照元とする。

/// Tauriイベント名（02_仕様書.md §5.2）
pub mod events {
    pub const PROGRESS: &str = "analysis://progress";
    pub const PHASE_COMPLETE: &str = "analysis://phase-complete";
    pub const FILE_COMPLETE: &str = "analysis://file-complete";
    pub const ERROR: &str = "analysis://error";
    pub const COMPLETE: &str = "analysis://complete";
    pub const CANCELLED: &str = "analysis://cancelled";
}

/// 解析フェーズ名（02_仕様書.md §2.1）
pub mod phases {
    pub const SCAN: &str = "scan";
    pub const STATIC: &str = "static";
    pub const AI: &str = "ai";
    pub const DOC_GEN: &str = "doc_gen";
}

/// エラーカテゴリ（03_設計書.md §7.1 AppErrorのtagと対応）
pub mod error_categories {
    pub const SCAN: &str = "scan";
    pub const STATIC: &str = "static";
    pub const AI: &str = "ai";
    pub const CACHE: &str = "cache";
    pub const EXPORT: &str = "export";
    pub const CONFIG: &str = "config";
}

/// SQLiteテーブル名（03_設計書.md §3.2）
pub mod tables {
    pub const FILE_ANALYSIS_CACHE: &str = "file_analysis_cache";
    pub const PROJECT_ANALYSIS_CACHE: &str = "project_analysis_cache";
    pub const ANALYSIS_HISTORY: &str = "analysis_history";
    pub const DB_METADATA: &str = "db_metadata";
}

/// db_metadata の既定キー
pub mod db_metadata_keys {
    pub const SCHEMA_VERSION: &str = "schema_version";
}

/// 現行スキーマバージョン（db_metadata.schema_version の既定値。マイグレーション時にここを更新する）
pub const CURRENT_SCHEMA_VERSION: &str = "1";

/// プロンプトバージョン（04_実装詳細.md §6。プロンプト本文を変更したら必ず更新する。キャッシュキーと連動）
pub const PROMPT_VERSION: &str = "1.1.0";
/// RPA専用プロンプトのバージョン（04_実装詳細.md §8.4。コード用プロンプト変更でRPAキャッシュを無効化しないため独立管理）
pub const RPA_PROMPT_VERSION: &str = "1.0.0";

/// 出力ドキュメント種別（04_実装詳細.md §11）
pub mod doc_types {
    pub const SYSTEM_SPEC: &str = "systemSpec";
    pub const BASIC_DESIGN: &str = "basicDesign";
    pub const DETAIL_DESIGN: &str = "detailDesign";
}

/// 既定出力先（プロジェクトフォルダ直下のサブディレクトリ名。04_実装詳細.md §4/§11.2）
pub mod export_paths {
    pub const DEFAULT_OUTPUT_DIR_NAME: &str = "projectlens-docs";
    pub const SYSTEM_SPEC_FILENAME: &str = "system-spec";
    pub const BASIC_DESIGN_FILENAME: &str = "basic-design";
    pub const DETAIL_DESIGN_SUBDIR: &str = "detail";
}

/// RPAツール種別（04_実装詳細.md §8.1、config.scan.rpa.tools と対応）
pub mod rpa_tools {
    pub const POWER_PLATFORM: &str = "powerplatform";
    pub const PAD: &str = "pad";
    pub const UIPATH: &str = "uipath";
}

/// NewtonXサイドカー RPCメソッド名（04_実装詳細.md §3.2）
pub mod newtonx_rpc {
    pub const AUTH_STATUS: &str = "auth.status";
    pub const AUTH_SAVE_CREDENTIALS: &str = "auth.save_credentials";
    pub const AUTH_CLEAR_CREDENTIALS: &str = "auth.clear_credentials";
    pub const AI_TEST: &str = "ai.test";
    pub const AI_ANALYZE: &str = "ai.analyze";
    pub const SESSION_OPEN: &str = "session.open";
    pub const SESSION_CLOSE: &str = "session.close";
    pub const ASSISTANTS_LIST: &str = "assistants.list";
}

/// NewtonX解析用チャットの専用フォルダ名（04_実装詳細.md §3.3）
pub const NEWTONX_CHAT_FOLDER_NAME: &str = "ProjectLens";

/// OS資格情報ストアのサービス識別子（credential_store.rs で使用）
pub mod credential_store {
    pub const SERVICE_NAME: &str = "ProjectLens";
    pub const NEWTONX_PAT_ENTRY: &str = "newtonx_pat";
}
