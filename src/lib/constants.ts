// アプリ全体で共有する文字列定数の一元定義（CLAUDE.md 原則2.1.3）。
// Rust側は src-tauri/src/models/constants.rs を単一の参照元とし、フロント側は本ファイルを
// 単一の参照元とする。値は必ず両者で一致させること。

/** Tauriイベント名（models/constants.rs::events と対応） */
export const Events = {
  PROGRESS: "analysis://progress",
  PHASE_COMPLETE: "analysis://phase-complete",
  FILE_COMPLETE: "analysis://file-complete",
  ERROR: "analysis://error",
  COMPLETE: "analysis://complete",
  CANCELLED: "analysis://cancelled",
} as const;

/** 解析フェーズ名（models/constants.rs::phases と対応） */
export const Phases = {
  SCAN: "scan",
  STATIC: "static",
  AI: "ai",
  DOC_GEN: "doc_gen",
} as const;
export type Phase = (typeof Phases)[keyof typeof Phases];

/** エラーカテゴリ（models/constants.rs::error_categories と対応） */
export const ErrorCategories = {
  SCAN: "scan",
  STATIC: "static",
  AI: "ai",
  CACHE: "cache",
  EXPORT: "export",
  CONFIG: "config",
} as const;
export type ErrorCategory = (typeof ErrorCategories)[keyof typeof ErrorCategories];

/** 出力ドキュメント種別（models/constants.rs::doc_types と対応） */
export const DocTypes = {
  SYSTEM_SPEC: "systemSpec",
  BASIC_DESIGN: "basicDesign",
  DETAIL_DESIGN: "detailDesign",
  DIRECTORY_STRUCTURE: "directoryStructure",
} as const;
export type DocType = (typeof DocTypes)[keyof typeof DocTypes];

/** エクスポート形式 */
export const ExportFormats = {
  MARKDOWN: "markdown",
  HTML: "html",
  JSON: "json",
} as const;
export type ExportFormat = (typeof ExportFormats)[keyof typeof ExportFormats];
