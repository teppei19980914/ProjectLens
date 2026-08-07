//! Power Platform（Power Automate クラウドフロー / Power Apps）解析（04_実装詳細.md §8.1）。
//! 対象: アンパック済みソリューションエクスポート内の solution.xml / customizations.xml /
//! Workflows/*.json / CanvasApps/**/*.json。

use super::RpaAnalyzer;
use crate::models::constants::rpa_tools;
use crate::models::{AppResult, RpaComponent, RpaStep};
use std::path::Path;

pub struct PowerPlatformAnalyzer;

impl RpaAnalyzer for PowerPlatformAnalyzer {
    fn tool_name(&self) -> &'static str {
        rpa_tools::POWER_PLATFORM
    }

    fn detect(&self, relative_path: &Path, content: &str) -> bool {
        let path_str = relative_path.to_string_lossy().replace('\\', "/");
        let file_name = relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name.eq_ignore_ascii_case("solution.xml")
            || file_name.eq_ignore_ascii_case("customizations.xml")
        {
            return true;
        }

        let is_workflow_json = path_str.contains("Workflows/") && file_name.ends_with(".json");
        let is_canvas_json = path_str.contains("CanvasApps/") && file_name.ends_with(".json");
        if is_workflow_json || is_canvas_json {
            // 拡張子だけでなく内容マーカー（$schema / definition キー）まで確認する（誤検出防止）
            return content.contains("\"$schema\"") || content.contains("\"definition\"");
        }

        false
    }

    fn parse(&self, relative_path: &Path, content: &str) -> AppResult<RpaComponent> {
        let flow_name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let flow_kind = if content.contains("\"triggers\"") {
            "automated_or_scheduled"
        } else {
            "unknown"
        };

        let step_count = content.matches("\"actions\"").count();
        let branch_or_loop_count =
            content.matches("\"Condition\"").count() + content.matches("apply_to_each").count();

        let external_connections: Vec<String> = ["SharePoint", "Dataverse", "Outlook", "Teams"]
            .iter()
            .filter(|c| content.contains(**c))
            .map(|c| c.to_string())
            .collect();

        Ok(RpaComponent {
            component_path: relative_path.to_string_lossy().replace('\\', "/"),
            tool: rpa_tools::POWER_PLATFORM.into(),
            flow_name,
            flow_kind: flow_kind.into(),
            triggers: Vec::new(),
            steps: vec![RpaStep {
                name: "actions".into(),
                kind: format!("count={step_count}"),
            }],
            branch_or_loop_count: branch_or_loop_count as u32,
            external_connections,
            dependencies: Vec::new(),
        })
    }
}
