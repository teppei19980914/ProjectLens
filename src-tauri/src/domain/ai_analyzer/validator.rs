//! AI応答バリデーションパイプライン（03_設計書.md §5.2、04_実装詳細.md §5.1/§5.2）。
//! 前処理→パース→検証→補完。ファイル/プロジェクトのスキーマ差はここに集約し重複させない
//! （CLAUDE.md 原則2.2.1）。

use crate::models::{AppError, FileAnalysisResult, ProjectAnalysisResult};
use serde_json::Value;

fn invalid_response(context: &str, detail: impl std::fmt::Display) -> AppError {
    AppError::Ai {
        message: format!("AI応答の検証に失敗しました（{context}）: {detail}"),
        recoverable: true,
        retryable: true, // 応答不正はリトライ対象（04_実装詳細.md §3.5）
    }
}

/// 前置き文・コードフェンス（```json 等）を除去する。
fn strip_wrapper(raw: &str) -> &str {
    let trimmed = raw.trim();
    let start = trimmed.find('{');
    let end = trimmed.rfind('}');
    match (start, end) {
        (Some(s), Some(e)) if e >= s => &trimmed[s..=e],
        _ => trimmed,
    }
}

fn clamp_importance_score(obj: &mut serde_json::Map<String, Value>) {
    let score = obj
        .get("importanceScore")
        .and_then(|v| v.as_i64())
        .unwrap_or(5)
        .clamp(1, 10);
    obj.insert("importanceScore".into(), Value::from(score));
}

fn normalize_severity(obj: &mut serde_json::Map<String, Value>) {
    let valid = matches!(obj.get("severity").and_then(|v| v.as_str()), Some("high" | "medium" | "low"));
    if !valid {
        obj.insert("severity".into(), Value::String("medium".into()));
    }
}

fn normalize_severity_array(value: &mut Value, key: &str) {
    if let Some(arr) = value.get_mut(key).and_then(|v| v.as_array_mut()) {
        for item in arr.iter_mut() {
            if let Some(obj) = item.as_object_mut() {
                normalize_severity(obj);
            }
        }
    }
}

/// ファイル単位AI解析結果を検証・補完する（04_実装詳細.md §5.1）。
pub fn parse_file_result(file_path: &str, raw: &str) -> Result<FileAnalysisResult, AppError> {
    let json_str = strip_wrapper(raw);
    let mut value: Value =
        serde_json::from_str(json_str).map_err(|e| invalid_response("JSONパース", e))?;

    let obj = value
        .as_object_mut()
        .ok_or_else(|| invalid_response("トップレベル構造", "オブジェクトではありません"))?;

    for required in ["roleSummary", "publicApis", "designPatterns", "potentialIssues"] {
        if !obj.contains_key(required) {
            return Err(invalid_response("必須キー欠損", required));
        }
    }

    clamp_importance_score(obj);
    obj.insert("filePath".into(), Value::String(file_path.to_string()));
    obj.insert("cacheHit".into(), Value::Bool(false));
    normalize_severity_array(&mut value, "potentialIssues");

    serde_json::from_value(value).map_err(|e| invalid_response("型変換", e))
}

/// プロジェクト全体AI解析結果を検証・補完する（04_実装詳細.md §5.2）。
pub fn parse_project_result(raw: &str) -> Result<ProjectAnalysisResult, AppError> {
    let json_str = strip_wrapper(raw);
    let mut value: Value =
        serde_json::from_str(json_str).map_err(|e| invalid_response("JSONパース", e))?;

    {
        let obj = value
            .as_object_mut()
            .ok_or_else(|| invalid_response("トップレベル構造", "オブジェクトではありません"))?;
        for required in ["systemSpec", "basicDesign"] {
            if !obj.contains_key(required) {
                return Err(invalid_response("必須キー欠損", required));
            }
        }
    }

    if let Some(basic_design) = value.get_mut("basicDesign") {
        normalize_severity_array(basic_design, "technicalDebts");
    }

    serde_json::from_value(value).map_err(|e| invalid_response("型変換", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_code_fence_and_clamps_score() {
        let raw = "以下がJSONです。\n```json\n{\"roleSummary\":\"テスト\",\"publicApis\":[],\"designPatterns\":[],\"importanceScore\":99,\"potentialIssues\":[{\"severity\":\"critical\",\"description\":\"x\",\"suggestion\":\"y\"}]}\n```";
        let result = parse_file_result("src/foo.ts", raw).unwrap();
        assert_eq!(result.importance_score, 10);
        assert!(matches!(result.potential_issues[0].severity, crate::models::Severity::Medium));
    }

    #[test]
    fn missing_required_key_is_retryable_ai_error() {
        let raw = "{\"publicApis\":[]}";
        let err = parse_file_result("src/foo.ts", raw).unwrap_err();
        match err {
            AppError::Ai { retryable, recoverable, .. } => {
                assert!(retryable);
                assert!(recoverable);
            }
            _ => panic!("expected AppError::Ai"),
        }
    }
}
