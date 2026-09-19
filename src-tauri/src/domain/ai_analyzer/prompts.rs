//! プロンプト本文（04_実装詳細.md §6、§8.4、§8.6）。プロンプト本文とバージョンは本ファイルに
//! 集約する（CLAUDE.md 原則2.1.3）。プロンプトを変更したら `constants::PROMPT_VERSION` /
//! `constants::RPA_PROMPT_VERSION` / `constants::VBA_PROMPT_VERSION` を必ず更新すること
//! （キャッシュキーと連動）。

const FILE_SCHEMA: &str = r#"{
  "roleSummary": "string: このファイルの責務の要約（日本語・200字以内）",
  "publicApis": [
    { "name": "string", "kind": "function|class|const|type", "description": "string" }
  ],
  "designPatterns": ["string"],
  "importanceScore": 5,
  "potentialIssues": [
    { "severity": "high|medium|low", "description": "string", "suggestion": "string" }
  ]
}"#;

const PROJECT_SCHEMA: &str = r#"{
  "systemSpec": {
    "purpose": "string: システムが解決する課題・存在意義（平易な日本語、非エンジニアが読める表現）",
    "mainFeatures": [
      { "name": "string: 機能名（平易な日本語）", "description": "string: 何ができるかの説明（平易な日本語）" }
    ],
    "userFlows": "string: 主要な利用シーン・業務の流れの説明（平易な日本語）"
  },
  "basicDesign": {
    "architecturePattern": "string",
    "modules": [
      { "name": "string", "responsibility": "string", "keyFiles": ["string"] }
    ],
    "techStack": ["string"],
    "dataFlow": "string: データの流れの説明（技術者向け）",
    "technicalDebts": [
      { "severity": "high|medium|low", "area": "string", "description": "string", "recommendation": "string" }
    ],
    "mermaidDiagrams": [
      { "type": "architecture|dependency|dataflow", "title": "string", "source": "string: mermaidソース" }
    ]
  }
}"#;

pub struct FileAnalysisInput<'a> {
    pub file_path: &'a str,
    pub language: &'a str,
    pub loc: u32,
    pub complexity: u32,
    pub functions: &'a [String],
    pub classes: &'a [String],
    pub file_content: &'a str,
}

/// ファイル解析プロンプト（04_実装詳細.md §6.1）。
pub fn file_analysis_prompt(input: &FileAnalysisInput) -> String {
    format!(
        "あなたはソフトウェアアーキテクチャの専門家です。以下のソースコードファイルを解析してください。\n\n\
# 制約\n\
- 応答は指定するJSON形式のみで返すこと。前置き・説明文・コードフェンスは一切含めないこと\n\
- すべての文字列値は日本語で記述すること\n\
- 推測で断定せず、コードから読み取れる事実に基づくこと\n\n\
# 出力JSON形式\n{schema}\n\n\
# ファイル情報\n\
- パス: {path}\n\
- 言語: {language}\n\
- 静的解析結果: 行数={loc}, 循環的複雑度={complexity}, 関数={functions}, クラス={classes}\n\n\
# ソースコード\n{content}\n",
        schema = FILE_SCHEMA,
        path = input.file_path,
        language = input.language,
        loc = input.loc,
        complexity = input.complexity,
        functions = input.functions.join(", "),
        classes = input.classes.join(", "),
        content = input.file_content,
    )
}

pub struct ProjectAnalysisInput<'a> {
    pub project_root: &'a str,
    pub file_count: usize,
    pub language_stats: &'a str,
    pub dependency_summary: &'a str,
    pub cycles: &'a str,
    pub file_summaries: &'a str,
}

/// プロジェクト全体解析プロンプト（04_実装詳細.md §6.2）。
pub fn project_analysis_prompt(input: &ProjectAnalysisInput) -> String {
    format!(
        "あなたはソフトウェアアーキテクチャの専門家です。以下のプロジェクト情報全体を俯瞰し、「システム仕様書」と「システム基本設計書」の元データをそれぞれ生成してください。\n\n\
# 制約\n\
- 応答は指定するJSON形式のみで返すこと。前置き・説明文・コードフェンスは一切含めないこと\n\
- すべての文字列値は日本語で記述すること\n\
- `systemSpec` 配下は非エンジニアが読んでも理解できる平易な日本語で記述し、専門用語（アーキテクチャパターン名・モジュール名等のコード上の固有名詞を除く）は避けること\n\
- `basicDesign` 配下は開発者向けとして、技術用語を正確に用いて記述すること\n\
- mermaidDiagrams.source は有効なMermaid記法であること\n\n\
# 出力JSON形式\n{schema}\n\n\
# プロジェクト情報\n\
- ルート: {root}\n\
- ファイル数: {file_count} / 言語構成: {language_stats}\n\
- 依存関係グラフ要約: {dependency_summary}（循環依存: {cycles}）\n\
- 各ファイルの解析要約:\n{file_summaries}\n",
        schema = PROJECT_SCHEMA,
        root = input.project_root,
        file_count = input.file_count,
        language_stats = input.language_stats,
        dependency_summary = input.dependency_summary,
        cycles = input.cycles,
        file_summaries = input.file_summaries,
    )
}

/// RPA定義ファイル解析プロンプト（04_実装詳細.md §8.4）。§5.1スキーマを流用し、
/// publicApisを「トリガー/主要アクション」、designPatternsを「フローパターン」として解釈させる。
pub fn rpa_analysis_prompt(component_path: &str, tool: &str, intermediate_repr_json: &str, raw_content: &str) -> String {
    format!(
        "あなたはRPA（Robotic Process Automation）の専門家です。以下のRPA定義ファイルを解析してください。\n\n\
# 制約\n\
- 応答は指定するJSON形式のみで返すこと。前置き・説明文・コードフェンスは一切含めないこと\n\
- すべての文字列値は日本語で記述すること\n\
- publicApisには「トリガー/主要アクション」を、designPatternsには「フローパターン（承認フロー/通知/データ同期/画面自動操作等）」を記述すること\n\n\
# 出力JSON形式\n{schema}\n\n\
# コンポーネント情報\n\
- パス: {path}\n\
- ツール: {tool}\n\
- 構造解析結果（共通中間表現）: {intermediate}\n\n\
# 定義ファイル原文\n{raw}\n",
        schema = FILE_SCHEMA,
        path = component_path,
        tool = tool,
        intermediate = intermediate_repr_json,
        raw = raw_content,
    )
}

/// VBAマクロ解析プロンプト（04_実装詳細.md §8.6）。§5.1スキーマを流用し、
/// publicApisを「公開Sub/Function（Public宣言のプロシージャ）」、
/// designPatternsを「マクロの処理パターン（自動化/データ加工/UI操作/外部アプリ連携等）」として解釈させる。
pub fn vba_analysis_prompt(component_path: &str, workbook_name: &str, modules_json: &str, combined_source: &str) -> String {
    format!(
        "あなたはVBA（Visual Basic for Applications）マクロの専門家です。以下のExcelワークブックに含まれるVBAマクロを解析してください。\n\n\
# 制約\n\
- 応答は指定するJSON形式のみで返すこと。前置き・説明文・コードフェンスは一切含めないこと\n\
- すべての文字列値は日本語で記述すること\n\
- publicApisには「公開Sub/Function（Public宣言のプロシージャ）」を、designPatternsには「マクロの処理パターン（自動化/データ加工/UI操作/外部アプリ連携等）」を記述すること\n\
- 複数モジュールで構成される場合は、モジュール間の呼び出し関係も踏まえて役割要約を記述すること\n\n\
# 出力JSON形式\n{schema}\n\n\
# ワークブック情報\n\
- パス: {path}\n\
- ワークブック名: {workbook}\n\
- モジュール構成（共通中間表現）: {modules}\n\n\
# VBAソース（全モジュール結合）\n{source}\n",
        schema = FILE_SCHEMA,
        path = component_path,
        workbook = workbook_name,
        modules = modules_json,
        source = combined_source,
    )
}
