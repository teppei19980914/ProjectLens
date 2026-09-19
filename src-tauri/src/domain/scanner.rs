//! プロジェクトフォルダの走査（03_設計書.md §2.2 Scanner、04_実装詳細.md §8.2）。
//! .gitignore尊重・除外パターン・上限判定・SHA-256ハッシュ生成 + RPA検出を行う。

use crate::domain::rpa_analyzer::{self, RpaAnalyzer};
use crate::domain::vba_analyzer::{self, VbaContainerAnalyzer};
use crate::models::config::ScanConfig;
use crate::models::{AppError, AppResult, ScanResult, ScannedFile};
use ignore::WalkBuilder;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub struct Scanner<'a> {
    config: &'a ScanConfig,
}

impl<'a> Scanner<'a> {
    pub fn new(config: &'a ScanConfig) -> Self {
        Self { config }
    }

    /// プロジェクトフォルダを再帰走査し、対象ファイル一覧とSHA-256ハッシュを返す。
    pub fn scan(&self, project_root: &Path, cancel: &CancellationToken) -> AppResult<ScanResult> {
        if !project_root.is_dir() {
            return Err(AppError::scan(format!(
                "指定されたフォルダが存在しません: {}",
                project_root.display()
            )));
        }

        let analyzers = rpa_analyzer::all_analyzers();
        let vba_analyzers = vba_analyzer::all_analyzers();
        let normal_extensions: Vec<String> = self
            .config
            .extensions
            .iter()
            .map(|e| e.trim_start_matches('.').to_ascii_lowercase())
            .collect();

        let mut walker = WalkBuilder::new(project_root);
        walker
            .hidden(false) // .gitignoreされていない限りhiddenも走査対象にする(除外は明示パターンに委ねる)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(true);

        // 追加除外パターン（config.scan.excludePatterns）をディレクトリ名一致で除外する
        let exclude_patterns = self.config.exclude_patterns.clone();

        let mut files = Vec::new();
        let mut skipped_for_limit = false;

        for entry in walker.build() {
            if cancel.is_cancelled() {
                return Err(AppError::scan("解析はキャンセルされました"));
            }

            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // 読み取り不能エントリはスキップして継続（NFR-03）
            };

            if !entry.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }

            let path = entry.path();
            let relative_path = path.strip_prefix(project_root).unwrap_or(path);

            if is_excluded(relative_path, &exclude_patterns) {
                continue;
            }

            if files.len() >= self.config.max_file_count as usize {
                skipped_for_limit = true;
                break;
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size_bytes = metadata.len();

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            let is_normal_code_ext = normal_extensions.iter().any(|e| e == &ext);

            // RPA/マクロ疑い判定は軽量に: 通常コード拡張子でない場合のみ、内容を確認して
            // 各アナライザに判定させる（通常コードファイルまで毎回読み込むとI/Oコストが増えるため）。
            // 非コードファイルの足切りラインはRPA/VBA両方の上限のうち大きい方を採用し、
            // 個別の上限は各アナライザ側の判定材料として別途扱う（既存RPA挙動は変えない）。
            let size_limit_kb = if is_normal_code_ext {
                self.config.max_file_size_kb
            } else {
                self.config.rpa.max_file_size_kb.max(self.config.vba.max_file_size_kb)
            };
            if size_bytes > size_limit_kb * 1024 {
                continue;
            }

            let rpa_tool = if self.config.rpa.enabled && !is_normal_code_ext {
                detect_rpa(relative_path, path, &analyzers)
            } else {
                None
            };

            let macro_tool = if self.config.vba.enabled && !is_normal_code_ext && rpa_tool.is_none() {
                detect_macro(relative_path, path, &vba_analyzers)
            } else {
                None
            };

            if !is_normal_code_ext && rpa_tool.is_none() && macro_tool.is_none() {
                // 通常コードでもRPAでもマクロでもないファイルは対象外
                let _ = file_name; // 将来: 追加の判定ロジックで使用予定
                continue;
            }

            let content_bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let hash = sha256_hex(&content_bytes);

            files.push(ScannedFile {
                path: relative_path.to_string_lossy().replace('\\', "/"),
                hash,
                size_bytes,
                rpa_tool,
                macro_tool,
            });
        }

        if skipped_for_limit {
            // 上限到達は解析継続可能な回復可能エラーとして扱うため、ここでは例外にせず
            // 呼び出し側（Orchestrator）がイベント通知することを想定し、結果はそのまま返す。
        }

        let total_count = files.len();
        Ok(ScanResult { files, total_count })
    }
}

fn is_excluded(relative_path: &Path, exclude_patterns: &[String]) -> bool {
    relative_path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        exclude_patterns.iter().any(|p| s == p.as_str())
    })
}

fn detect_rpa(
    relative_path: &Path,
    absolute_path: &Path,
    analyzers: &[Box<dyn RpaAnalyzer>],
) -> Option<String> {
    let content = std::fs::read_to_string(absolute_path).ok()?;
    rpa_analyzer::detect_tool(analyzers, relative_path, &content).map(|a| a.tool_name().to_string())
}

/// マクロ（VBA等、バイナリコンテナ）ファイルの検出。RPAと異なりテキストとして読めないため、
/// 各アナライザ実装（例: xlsmはzip中央ディレクトリの内容確認）に判定を委ねる。
fn detect_macro(
    relative_path: &Path,
    absolute_path: &Path,
    analyzers: &[Box<dyn VbaContainerAnalyzer>],
) -> Option<String> {
    vba_analyzer::detect_tool(analyzers, relative_path, absolute_path).map(|a| a.tool_name().to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}
