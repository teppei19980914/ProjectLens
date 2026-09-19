//! VBAマクロ（Excel等）解析の共通trait（04_実装詳細.md §8.6）。
//! RPA解析（domain/rpa_analyzer）と同型の構造にし、コンテナ形式追加（.docm/.pptm等）は
//! 本traitの実装追加のみで対応できるようにする（CLAUDE.md 原則2.2.2 DRY）。

pub mod msovba;
pub mod xlsm;

use crate::models::{AppResult, VbaComponent};
use std::path::Path;

pub trait VbaContainerAnalyzer: Send + Sync {
    /// config.scan.vba.extensions と対応するツール識別子
    fn tool_name(&self) -> &'static str;

    /// 拡張子 + コンテナ内部構造（例: zip中央ディレクトリでの vbaProject.bin の存在）で
    /// VBAマクロを含むファイルかどうかを判定する（拡張子のみでの無差別判定は禁止）。
    fn detect(&self, relative_path: &Path, absolute_path: &Path) -> bool;

    /// 検出済みファイルから共通中間表現（VbaComponent）と、AI解析用に結合したVBAソース原文を抽出する。
    fn extract(&self, relative_path: &Path, absolute_path: &Path) -> AppResult<(VbaComponent, String)>;
}

/// 対応する全VBAコンテナアナライザを構築する。形式追加時はここに1行加えるだけでよい。
pub fn all_analyzers() -> Vec<Box<dyn VbaContainerAnalyzer>> {
    vec![Box::new(xlsm::XlsmAnalyzer)]
}

/// スキャン対象の1ファイルに対し、対応するVBAコンテナアナライザ（あれば）を返す。
pub fn detect_tool<'a>(
    analyzers: &'a [Box<dyn VbaContainerAnalyzer>],
    relative_path: &Path,
    absolute_path: &Path,
) -> Option<&'a dyn VbaContainerAnalyzer> {
    analyzers
        .iter()
        .map(|a| a.as_ref())
        .find(|a| a.detect(relative_path, absolute_path))
}
