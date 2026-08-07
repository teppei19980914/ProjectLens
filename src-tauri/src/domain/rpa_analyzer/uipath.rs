//! UiPath解析（04_実装詳細.md §8.1）。
//! 対象: プロジェクト定義 project.json + ワークフロー *.xaml。

use super::RpaAnalyzer;
use crate::models::constants::rpa_tools;
use crate::models::{AppResult, RpaComponent, RpaStep};
use std::path::Path;

pub struct UiPathAnalyzer;

impl RpaAnalyzer for UiPathAnalyzer {
    fn tool_name(&self) -> &'static str {
        rpa_tools::UIPATH
    }

    fn detect(&self, relative_path: &Path, content: &str) -> bool {
        let file_name = relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name.eq_ignore_ascii_case("project.json") {
            // UiPath固有キーで確認する（誤検出防止）
            return content.contains("\"studioVersion\"")
                || content.contains("\"dependencies\"") && content.contains("\"projectId\"");
        }

        if file_name.ends_with(".xaml") {
            return content.contains("UiPath.Core") || content.contains("System.Activities");
        }

        false
    }

    fn parse(&self, relative_path: &Path, content: &str) -> AppResult<RpaComponent> {
        let file_name = relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if file_name.eq_ignore_ascii_case("project.json") {
            return Ok(RpaComponent {
                component_path: relative_path.to_string_lossy().replace('\\', "/"),
                tool: rpa_tools::UIPATH.into(),
                flow_name: "project.json".into(),
                flow_kind: "project".into(),
                triggers: Vec::new(),
                steps: Vec::new(),
                branch_or_loop_count: 0,
                external_connections: Vec::new(),
                dependencies: Vec::new(),
            });
        }

        let flow_name = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let activities: Vec<RpaStep> = ["If", "While", "ForEach", "TryCatch", "InvokeWorkflowFile"]
            .iter()
            .filter(|a| content.contains(&format!("<ui:{a}")) || content.contains(&format!("<{a}")))
            .map(|a| RpaStep {
                name: (*a).into(),
                kind: "activity".into(),
            })
            .collect();

        let branch_or_loop_count = activities
            .iter()
            .filter(|s| matches!(s.name.as_str(), "If" | "While" | "ForEach"))
            .count() as u32;

        let dependencies: Vec<String> = if content.contains("InvokeWorkflowFile") {
            vec!["InvokeWorkflowFile参照あり（詳細抽出は実サンプル検証後に対応）".into()]
        } else {
            Vec::new()
        };

        Ok(RpaComponent {
            component_path: relative_path.to_string_lossy().replace('\\', "/"),
            tool: rpa_tools::UIPATH.into(),
            flow_name,
            flow_kind: "workflow".into(),
            triggers: Vec::new(),
            steps: activities,
            branch_or_loop_count,
            external_connections: Vec::new(),
            dependencies,
        })
    }
}
