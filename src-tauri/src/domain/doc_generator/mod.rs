//! ドキュメント生成（03_設計書.md §2.2 DocGenerator、04_実装詳細.md §11）。
//! システム仕様書・システム基本設計書・詳細設計書（ファイル/RPAコンポーネント単位）の
//! 3成果物を、共通の中間表現（markdown.rs）からMD/HTML/JSONへレンダリングする。

pub mod markdown;

use crate::models::config::ExportConfig;
use crate::models::constants::{doc_types, export_paths};
use crate::models::{AppError, AppResult, ExportResult, ExportedFile, FileAnalysisResult, ProjectAnalysisResult, RpaComponent, StaticAnalysisResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn resolve_output_dir(project_path: &Path, configured: &str) -> PathBuf {
    if configured.is_empty() {
        project_path.join(export_paths::DEFAULT_OUTPUT_DIR_NAME)
    } else {
        PathBuf::from(configured)
    }
}

/// config.export の既定設定に従い3成果物を生成する（Orchestrator Phase4から呼ばれる）。
pub fn generate_all(
    output_dir: &Path,
    project_result: &ProjectAnalysisResult,
    static_results: &[StaticAnalysisResult],
    file_results: &[FileAnalysisResult],
    rpa_components: &[RpaComponent],
    export_config: &ExportConfig,
) -> AppResult<ExportResult> {
    generate(
        output_dir,
        &export_config.default_doc_types,
        &export_config.default_format,
        export_config.embed_mermaid,
        project_result,
        static_results,
        file_results,
        rpa_components,
    )
}

/// 明示的な成果物種別・形式を指定して生成する（エクスポート画面からの再出力用）。
#[allow(clippy::too_many_arguments)]
pub fn generate(
    output_dir: &Path,
    doc_type_list: &[String],
    format: &str,
    embed_mermaid: bool,
    project_result: &ProjectAnalysisResult,
    static_results: &[StaticAnalysisResult],
    file_results: &[FileAnalysisResult],
    rpa_components: &[RpaComponent],
) -> AppResult<ExportResult> {
    std::fs::create_dir_all(output_dir).map_err(|e| AppError::export(format!("出力先の作成に失敗しました: {e}")))?;

    let ext = format_extension(format)?;
    let mut files = Vec::new();

    if doc_type_list.iter().any(|t| t == doc_types::SYSTEM_SPEC) {
        let md = markdown::system_spec(&project_result.system_spec);
        let path = output_dir.join(format!("{}.{}", export_paths::SYSTEM_SPEC_FILENAME, ext));
        write_rendered(&path, &md, format, &project_result.system_spec)?;
        files.push(ExportedFile {
            doc_type: doc_types::SYSTEM_SPEC.into(),
            path: path.to_string_lossy().to_string(),
            source_file_path: None,
        });
    }

    if doc_type_list.iter().any(|t| t == doc_types::BASIC_DESIGN) {
        let md = markdown::basic_design(&project_result.basic_design, embed_mermaid);
        let path = output_dir.join(format!("{}.{}", export_paths::BASIC_DESIGN_FILENAME, ext));
        write_rendered(&path, &md, format, &project_result.basic_design)?;
        files.push(ExportedFile {
            doc_type: doc_types::BASIC_DESIGN.into(),
            path: path.to_string_lossy().to_string(),
            source_file_path: None,
        });
    }

    if doc_type_list.iter().any(|t| t == doc_types::DETAIL_DESIGN) {
        let detail_dir = output_dir.join(export_paths::DETAIL_DESIGN_SUBDIR);
        std::fs::create_dir_all(&detail_dir).map_err(|e| AppError::export(format!("詳細設計書出力先の作成に失敗しました: {e}")))?;

        let static_by_path: HashMap<&str, &StaticAnalysisResult> =
            static_results.iter().map(|r| (r.file_path.as_str(), r)).collect();
        let rpa_by_path: HashMap<&str, &RpaComponent> =
            rpa_components.iter().map(|c| (c.component_path.as_str(), c)).collect();

        for file_result in file_results {
            let file_name = sanitize_filename(&file_result.file_path);
            let path = detail_dir.join(format!("{file_name}.{ext}"));

            if let Some(component) = rpa_by_path.get(file_result.file_path.as_str()) {
                let md = markdown::detail_design_for_rpa(file_result, component);
                write_rendered(&path, &md, format, &(file_result, component))?;
            } else {
                let static_result = static_by_path.get(file_result.file_path.as_str()).copied();
                let md = markdown::detail_design_for_file(file_result, static_result);
                write_rendered(&path, &md, format, &(file_result, static_result))?;
            }

            files.push(ExportedFile {
                doc_type: doc_types::DETAIL_DESIGN.into(),
                path: path.to_string_lossy().to_string(),
                source_file_path: Some(file_result.file_path.clone()),
            });
        }
    }

    Ok(ExportResult {
        output_dir: output_dir.to_string_lossy().to_string(),
        files,
    })
}

fn format_extension(format: &str) -> AppResult<&'static str> {
    match format {
        "markdown" => Ok("md"),
        "html" => Ok("html"),
        "json" => Ok("json"),
        other => Err(AppError::export(format!("未対応の出力形式です: {other}"))),
    }
}

/// 中間表現（Markdown）を起点に、形式に応じてレンダリングして書き出す。
/// JSON形式のみ、元データ（`json_source`）をそのままシリアライズする（Markdownより情報が正確なため）。
fn write_rendered(path: &Path, markdown_source: &str, format: &str, json_source: &impl serde::Serialize) -> AppResult<()> {
    let content = match format {
        "markdown" => markdown_source.to_string(),
        "html" => render_html(markdown_source),
        "json" => serde_json::to_string_pretty(json_source).map_err(|e| AppError::export(format!("JSON変換に失敗しました: {e}")))?,
        other => return Err(AppError::export(format!("未対応の出力形式です: {other}"))),
    };
    std::fs::write(path, content).map_err(|e| AppError::export(format!("ファイル書き込みに失敗しました: {e}")))
}

fn render_html(markdown_source: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let parser = Parser::new_ext(markdown_source, Options::ENABLE_TABLES);
    let mut body = String::new();
    html::push_html(&mut body, parser);

    format!(
        "<!doctype html>\n<html lang=\"ja\"><head><meta charset=\"utf-8\">\n\
<script src=\"https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js\"></script>\n\
<script>window.addEventListener('DOMContentLoaded',()=>{{if(window.mermaid){{mermaid.initialize({{startOnLoad:true}});}}}});</script>\n\
<style>body{{font-family:sans-serif;max-width:960px;margin:2rem auto;padding:0 1rem;}}table{{border-collapse:collapse;}}td,th{{border:1px solid #ccc;padding:4px 8px;}}</style>\n\
</head><body>\n{body}\n</body></html>\n"
    )
}

/// 詳細設計書のファイル名衝突を避けるため、相対パスを `_` 区切りに変換する（04_実装詳細.md §11.2）。
fn sanitize_filename(relative_path: &str) -> String {
    relative_path.replace(['/', '\\'], "_")
}
