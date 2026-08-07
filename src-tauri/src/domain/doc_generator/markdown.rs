//! Markdown中間表現の組み立て（03_設計書.md §2.2 DocGenerator）。
//! MD/HTML/JSONは本ファイルが組み立てるMarkdownを起点にレンダラのみ差し替える
//! （CLAUDE.md 原則2.2.2「解析結果→中間表現→各レンダラ」）。

use crate::models::{BasicDesign, FileAnalysisResult, RpaComponent, StaticAnalysisResult, SystemSpec};

pub fn system_spec(spec: &SystemSpec) -> String {
    let mut out = String::new();
    out.push_str("# システム仕様書\n\n");
    out.push_str("## 目的\n\n");
    out.push_str(&spec.purpose);
    out.push_str("\n\n## 主要機能\n\n");
    for f in &spec.main_features {
        out.push_str(&format!("### {}\n\n{}\n\n", f.name, f.description));
    }
    out.push_str("## 主要な利用シーン・業務の流れ\n\n");
    out.push_str(&spec.user_flows);
    out.push('\n');
    out
}

pub fn basic_design(design: &BasicDesign, embed_mermaid: bool) -> String {
    let mut out = String::new();
    out.push_str("# システム基本設計書\n\n");
    out.push_str("## アーキテクチャパターン\n\n");
    out.push_str(&design.architecture_pattern);
    out.push_str("\n\n## モジュール構成\n\n");
    for m in &design.modules {
        out.push_str(&format!(
            "### {}\n\n{}\n\n主要ファイル: {}\n\n",
            m.name,
            m.responsibility,
            m.key_files.join(", ")
        ));
    }
    out.push_str("## 技術スタック\n\n");
    for t in &design.tech_stack {
        out.push_str(&format!("- {t}\n"));
    }
    out.push_str("\n## データフロー\n\n");
    out.push_str(&design.data_flow);
    out.push_str("\n\n## 技術的負債\n\n");
    if design.technical_debts.is_empty() {
        out.push_str("検出された技術的負債はありません。\n\n");
    } else {
        out.push_str("| 重大度 | 領域 | 内容 | 推奨対応 |\n|---|---|---|---|\n");
        for d in &design.technical_debts {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                severity_ja(&d.severity),
                d.area,
                d.description,
                d.recommendation
            ));
        }
        out.push('\n');
    }

    if embed_mermaid && !design.mermaid_diagrams.is_empty() {
        out.push_str("## 図\n\n");
        for diagram in &design.mermaid_diagrams {
            out.push_str(&format!("### {}\n\n```mermaid\n{}\n```\n\n", diagram.title, diagram.source));
        }
    }

    out
}

pub fn detail_design_for_file(file_result: &FileAnalysisResult, static_result: Option<&StaticAnalysisResult>) -> String {
    let mut out = String::new();
    out.push_str(&format!("# 詳細設計書: {}\n\n", file_result.file_path));
    out.push_str("## 役割要約\n\n");
    out.push_str(&file_result.role_summary);
    out.push_str("\n\n## 公開API\n\n");
    if file_result.public_apis.is_empty() {
        out.push_str("なし\n\n");
    } else {
        out.push_str("| 名称 | 種別 | 説明 |\n|---|---|---|\n");
        for api in &file_result.public_apis {
            out.push_str(&format!("| {} | {} | {} |\n", api.name, api.kind, api.description));
        }
        out.push('\n');
    }

    out.push_str("## 設計パターン\n\n");
    if file_result.design_patterns.is_empty() {
        out.push_str("検出なし\n\n");
    } else {
        for p in &file_result.design_patterns {
            out.push_str(&format!("- {p}\n"));
        }
        out.push('\n');
    }

    out.push_str(&format!("## 重要度スコア\n\n{}/10\n\n", file_result.importance_score));

    out.push_str("## 潜在的問題\n\n");
    if file_result.potential_issues.is_empty() {
        out.push_str("検出なし\n\n");
    } else {
        out.push_str("| 重大度 | 内容 | 改善提案 |\n|---|---|---|\n");
        for issue in &file_result.potential_issues {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                severity_ja(&issue.severity),
                issue.description,
                issue.suggestion
            ));
        }
        out.push('\n');
    }

    if let Some(sr) = static_result {
        out.push_str("## 静的解析メトリクス\n\n");
        out.push_str(&format!(
            "- 行数: {}\n- 循環的複雑度: {}\n- 最大ネスト深度: {}\n- 関数: {}\n- クラス: {}\n\n",
            sr.metrics.loc,
            sr.metrics.cyclomatic_complexity,
            sr.metrics.max_nest_depth,
            sr.functions.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", "),
            sr.classes.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(", "),
        ));
    }

    out
}

pub fn detail_design_for_rpa(file_result: &FileAnalysisResult, component: &RpaComponent) -> String {
    let mut out = String::new();
    out.push_str(&format!("# 詳細設計書（RPA）: {}\n\n", component.component_path));
    out.push_str(&format!("- ツール: {}\n- フロー名: {}\n- 種別: {}\n\n", component.tool, component.flow_name, component.flow_kind));
    out.push_str("## 役割要約\n\n");
    out.push_str(&file_result.role_summary);
    out.push_str("\n\n## トリガー/主要アクション\n\n");
    if file_result.public_apis.is_empty() {
        out.push_str("なし\n\n");
    } else {
        for api in &file_result.public_apis {
            out.push_str(&format!("- {}: {}\n", api.name, api.description));
        }
        out.push('\n');
    }
    out.push_str("## フローパターン\n\n");
    for p in &file_result.design_patterns {
        out.push_str(&format!("- {p}\n"));
    }
    out.push_str(&format!(
        "\n## 構造\n\n- 分岐/ループ数: {}\n- 外部接続: {}\n- 依存関係: {}\n\n",
        component.branch_or_loop_count,
        component.external_connections.join(", "),
        component.dependencies.join(", ")
    ));
    out.push_str("## 潜在的問題\n\n");
    for issue in &file_result.potential_issues {
        out.push_str(&format!("- [{}] {}（提案: {}）\n", severity_ja(&issue.severity), issue.description, issue.suggestion));
    }
    out
}

fn severity_ja(s: &crate::models::Severity) -> &'static str {
    match s {
        crate::models::Severity::High => "高",
        crate::models::Severity::Medium => "中",
        crate::models::Severity::Low => "低",
    }
}
