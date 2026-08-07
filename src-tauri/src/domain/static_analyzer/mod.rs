//! 静的解析エンジン（03_設計書.md §2.2 StaticAnalyzer）。
//! tree-sitterによる言語判定・メトリクス算出・import/export解析・関数/クラス抽出を行う。
//! 言語ごとの差異は `language_spec::LanguageSpec` テーブルに閉じ込め、
//! 走査ロジック自体は言語非依存の1実装（本ファイル）に共通化する（CLAUDE.md 原則2.2.2 DRY）。

pub mod dependency_graph;
pub mod language_spec;

use crate::models::{AppError, AppResult, ClassInfo, CodeMetrics, FunctionInfo, StaticAnalysisResult};
use language_spec::LanguageSpec;
use tree_sitter::{Node, Parser};

pub struct StaticAnalyzer;

impl StaticAnalyzer {
    /// 拡張子から対応言語を判定できるか
    pub fn supports_extension(ext: &str) -> bool {
        language_spec::find_spec_for_extension(ext).is_some()
    }

    /// 1ファイルを解析する。
    pub fn analyze(file_path: &str, source: &str, extension: &str) -> AppResult<StaticAnalysisResult> {
        let spec = language_spec::find_spec_for_extension(extension)
            .ok_or_else(|| AppError::static_analysis(format!("未対応の拡張子です: {extension}")))?;

        let mut parser = Parser::new();
        let language = (spec.language_fn)();
        parser
            .set_language(&language)
            .map_err(|e| AppError::static_analysis(format!("パーサ初期化に失敗しました: {e}")))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| AppError::static_analysis("ソースコードの解析に失敗しました".to_string()))?;

        let root = tree.root_node();
        let mut walker = Walker::new(source, &spec);
        walker.visit(root, 0);

        let loc = source.lines().count() as u32;

        Ok(StaticAnalysisResult {
            file_path: file_path.to_string(),
            language: spec.name.to_string(),
            metrics: CodeMetrics {
                loc,
                cyclomatic_complexity: 1 + walker.decision_count,
                max_nest_depth: walker.max_depth,
            },
            imports: walker.imports,
            exports: walker.exports,
            functions: walker.functions,
            classes: walker.classes,
        })
    }
}

struct Walker<'a> {
    source: &'a str,
    spec: &'a LanguageSpec,
    decision_count: u32,
    max_depth: u32,
    imports: Vec<String>,
    exports: Vec<String>,
    functions: Vec<FunctionInfo>,
    classes: Vec<ClassInfo>,
}

impl<'a> Walker<'a> {
    fn new(source: &'a str, spec: &'a LanguageSpec) -> Self {
        Self {
            source,
            spec,
            decision_count: 0,
            max_depth: 0,
            imports: Vec::new(),
            exports: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
        }
    }

    fn node_text(&self, node: Node) -> String {
        node.utf8_text(self.source.as_bytes())
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    fn visit(&mut self, node: Node, depth: u32) {
        let kind = node.kind();

        if self.spec.decision_kinds.contains(&kind) {
            self.decision_count += 1;
        }

        let is_block = self.spec.block_kinds.contains(&kind);
        let next_depth = if is_block { depth + 1 } else { depth };
        if next_depth > self.max_depth {
            self.max_depth = next_depth;
        }

        if self.spec.function_kinds.contains(&kind) {
            let name = node
                .child_by_field_name("name")
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| "<anonymous>".to_string());
            self.functions.push(FunctionInfo {
                name,
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
            });
        }

        if self.spec.class_kinds.contains(&kind) {
            let name = node
                .child_by_field_name("name")
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| "<anonymous>".to_string());
            self.classes.push(ClassInfo {
                name,
                start_line: node.start_position().row as u32 + 1,
                end_line: node.end_position().row as u32 + 1,
            });
        }

        if self.spec.import_kinds.contains(&kind) {
            self.imports.push(self.node_text(node));
        }

        if self.spec.export_kinds.contains(&kind) {
            self.exports.push(self.node_text(node));
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit(child, next_depth);
        }
    }
}
