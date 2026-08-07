//! Power Automate for Desktop（PAD）解析（04_実装詳細.md §8.1）。
//! 対象: Robinスクリプト形式のフローファイル（*.robin、または *.pad.txt）。

use super::RpaAnalyzer;
use crate::models::constants::rpa_tools;
use crate::models::{AppResult, RpaComponent, RpaStep};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

/// Robin構文のアクションドット記法（例: `Display.ShowMessage`）を検出する内容マーカー
static ACTION_DOT_NOTATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[A-Z][A-Za-z0-9]*\.[A-Z][A-Za-z0-9]*").unwrap());

pub struct PadAnalyzer;

impl RpaAnalyzer for PadAnalyzer {
    fn tool_name(&self) -> &'static str {
        rpa_tools::PAD
    }

    fn detect(&self, relative_path: &Path, content: &str) -> bool {
        let file_name = relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let is_candidate_ext = file_name.ends_with(".robin") || file_name.ends_with(".pad.txt");
        if !is_candidate_ext {
            return false;
        }
        // 拡張子だけでなく、Robin構文のアクションドット記法まで確認する（誤検出防止）
        ACTION_DOT_NOTATION.is_match(content)
    }

    fn parse(&self, relative_path: &Path, content: &str) -> AppResult<RpaComponent> {
        let flow_name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let actions: Vec<RpaStep> = ACTION_DOT_NOTATION
            .find_iter(content)
            .map(|m| RpaStep {
                name: m.as_str().to_string(),
                kind: "action".into(),
            })
            .collect();

        let branch_or_loop_count = content.matches("IF ").count()
            + content.matches("LOOP").count()
            + content.matches("ON ERROR").count();

        Ok(RpaComponent {
            component_path: relative_path.to_string_lossy().replace('\\', "/"),
            tool: rpa_tools::PAD.into(),
            flow_name,
            flow_kind: "desktop_flow".into(),
            triggers: Vec::new(),
            steps: actions,
            branch_or_loop_count: branch_or_loop_count as u32,
            external_connections: Vec::new(),
            dependencies: Vec::new(),
        })
    }
}
