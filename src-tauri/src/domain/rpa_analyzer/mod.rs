//! RPA定義ファイル解析の共通trait（04_実装詳細.md §8）。
//! ツール追加は本traitの実装追加のみで対応できる構造とする（CLAUDE.md 原則2.2.2 DRY）。
//!
//! 注意: `parse()` の抽出ロジックは実サンプル未入手のため暫定実装であり、
//! 実サンプル入手後（04_実装詳細.md §10 残課題#6）に検証・調整が必要。

pub mod pad;
pub mod power_platform;
pub mod uipath;

use crate::models::{AppResult, RpaComponent};
use std::path::Path;

pub trait RpaAnalyzer: Send + Sync {
    /// config.scan.rpa.tools / detectionPatterns と対応するツール識別子
    fn tool_name(&self) -> &'static str;

    /// パス構造パターン + 内容マーカーで、このツールのRPA定義ファイルかどうかを判定する
    /// （拡張子のみでの無差別判定は禁止。04_実装詳細.md §8.1）
    fn detect(&self, relative_path: &Path, content: &str) -> bool;

    /// 検出済みファイルから共通中間表現（RpaComponent）を抽出する
    fn parse(&self, relative_path: &Path, content: &str) -> AppResult<RpaComponent>;
}

/// 対応する全RPAアナライザを構築する。ツール追加時はここに1行加えるだけでよい。
pub fn all_analyzers() -> Vec<Box<dyn RpaAnalyzer>> {
    vec![
        Box::new(power_platform::PowerPlatformAnalyzer),
        Box::new(pad::PadAnalyzer),
        Box::new(uipath::UiPathAnalyzer),
    ]
}

/// スキャン対象の1ファイルに対し、対応するRPAアナライザ（あれば）を返す。
pub fn detect_tool<'a>(
    analyzers: &'a [Box<dyn RpaAnalyzer>],
    relative_path: &Path,
    content: &str,
) -> Option<&'a dyn RpaAnalyzer> {
    analyzers
        .iter()
        .map(|a| a.as_ref())
        .find(|a| a.detect(relative_path, content))
}
