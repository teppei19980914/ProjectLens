//! 解析結果の共通型定義（03_設計書.md §4、04_実装詳細.md §5.1/§5.2）。

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------
// ファイル単位AI解析結果
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAnalysisResult {
    pub file_path: String,
    /// 役割要約
    pub role_summary: String,
    /// 公開API
    pub public_apis: Vec<PublicApi>,
    /// 設計パターン
    pub design_patterns: Vec<String>,
    /// 1-10にクランプ
    pub importance_score: u8,
    /// 潜在的問題
    pub potential_issues: Vec<Issue>,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicApi {
    pub name: String,
    pub kind: String, // "function" | "class" | "const" | "type"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub severity: Severity,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    High,
    Medium,
    Low,
}

// ---------------------------------------------------------------------
// プロジェクト全体AI解析結果（システム仕様書・基本設計書の元データ。04_実装詳細.md §5.2/§11）
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAnalysisResult {
    pub system_spec: SystemSpec,
    pub basic_design: BasicDesign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSpec {
    /// システムの目的（平易な日本語）
    pub purpose: String,
    /// 主要機能（平易な日本語）
    pub main_features: Vec<FeatureSummary>,
    /// 主要な利用シーン・業務の流れ（平易な日本語）
    pub user_flows: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSummary {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicDesign {
    pub architecture_pattern: String,
    pub modules: Vec<ModuleSummary>,
    pub tech_stack: Vec<String>,
    pub data_flow: String,
    pub technical_debts: Vec<TechnicalDebt>,
    pub mermaid_diagrams: Vec<MermaidDiagram>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSummary {
    pub name: String,
    pub responsibility: String,
    pub key_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalDebt {
    pub severity: Severity,
    pub area: String,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MermaidDiagram {
    pub r#type: String, // "architecture" | "dependency" | "dataflow"
    pub title: String,
    pub source: String,
}

// ---------------------------------------------------------------------
// 静的解析結果
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticAnalysisResult {
    pub file_path: String,
    pub language: String,
    pub metrics: CodeMetrics,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeMetrics {
    pub loc: u32,
    pub cyclomatic_complexity: u32,
    pub max_nest_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassInfo {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
}

// ---------------------------------------------------------------------
// 依存グラフ
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    /// 循環依存
    pub cycles: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyNode {
    pub id: String,
    pub fan_in: u32,
    pub fan_out: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
}

// ---------------------------------------------------------------------
// 進捗イベント
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    pub phase: String, // "scan" | "static" | "ai" | "doc_gen"
    pub processed: u64,
    pub total: u64,
    pub current_file: Option<String>,
}

// ---------------------------------------------------------------------
// RPA構造解析の共通中間表現（04_実装詳細.md §8.3）
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpaComponent {
    pub component_path: String,
    pub tool: String, // "powerplatform" | "pad" | "uipath"
    pub flow_name: String,
    pub flow_kind: String,
    pub triggers: Vec<String>,
    pub steps: Vec<RpaStep>,
    pub branch_or_loop_count: u32,
    pub external_connections: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpaStep {
    pub name: String,
    pub kind: String,
}

// ---------------------------------------------------------------------
// スキャン結果
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedFile {
    pub path: String,
    pub hash: String,
    pub size_bytes: u64,
    /// RPAとして検出された場合、そのツール種別（04_実装詳細.md §8.1）
    pub rpa_tool: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
    pub total_count: usize,
}

// ---------------------------------------------------------------------
// キャッシュ・履歴（03_設計書.md §3、04_実装詳細.md §7）
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    pub file_entries: u64,
    pub project_entries: u64,
    pub size_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisHistoryEntry {
    pub id: i64,
    pub project_path: String,
    /// "completed" | "cancelled" | "error"
    pub status: String,
    pub total_files: u64,
    pub analyzed_files: u64,
    pub cache_hits: u64,
    pub total_tokens: u64,
    pub duration_ms: u64,
    pub error_summary: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

// ---------------------------------------------------------------------
// ドキュメント出力（04_実装詳細.md §11）
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFile {
    pub doc_type: String, // "systemSpec" | "basicDesign" | "detailDesign"
    pub path: String,
    pub source_file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub output_dir: String,
    pub files: Vec<ExportedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResultSnapshot {
    pub project_result: ProjectAnalysisResult,
    pub file_results: Vec<FileAnalysisResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisSummary {
    pub total_files: u64,
    pub analyzed_files: u64,
    pub cache_hits: u64,
    pub total_tokens: u64,
    pub duration_ms: u64,
    pub status: String,
}
