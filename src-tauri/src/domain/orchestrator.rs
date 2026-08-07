//! 4フェーズの実行制御・進捗イベント発火・キャンセル伝播・履歴記録（03_設計書.md §2.2 AnalysisOrchestrator）。
//! 各フェーズから直接emitせず、本ファイルのヘルパーに集約する（CLAUDE.md 原則2.2.1）。

use crate::domain::ai_analyzer::AiAnalyzer;
use crate::domain::rpa_analyzer;
use crate::domain::scanner::Scanner;
use crate::domain::static_analyzer::{dependency_graph, StaticAnalyzer};
use crate::infra::ai_client::AiClient;
use crate::infra::cache_repo::CacheRepo;
use crate::infra::history_repo::{HistoryRepo, NewHistoryEntry};
use crate::models::config::AppConfig;
use crate::models::constants::{events, phases, NEWTONX_CHAT_FOLDER_NAME};
use crate::models::{AnalysisSummary, AppError, AppResult, FileAnalysisResult, ProgressPayload, StaticAnalysisResult};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

pub struct Orchestrator {
    pub ai_client: Arc<dyn AiClient>,
    pub cache_repo: Arc<CacheRepo>,
    pub history_repo: Arc<HistoryRepo>,
}

pub struct FullAnalysisOutput {
    pub summary: AnalysisSummary,
    pub static_results: Vec<StaticAnalysisResult>,
    pub file_results: Vec<FileAnalysisResult>,
    pub rpa_components: Vec<crate::models::RpaComponent>,
    pub project_result: crate::models::ProjectAnalysisResult,
    pub dependency_graph: crate::models::DependencyGraph,
}

impl Orchestrator {
    fn emit_progress(app: &AppHandle, phase: &str, processed: u64, total: u64, current_file: Option<String>) {
        let _ = app.emit(
            events::PROGRESS,
            ProgressPayload {
                phase: phase.to_string(),
                processed,
                total,
                current_file,
            },
        );
    }

    fn emit_phase_complete(app: &AppHandle, phase: &str) {
        let _ = app.emit(events::PHASE_COMPLETE, serde_json::json!({ "phase": phase }));
    }

    pub async fn run_full_analysis(
        &self,
        app: AppHandle,
        project_path: &Path,
        config: &AppConfig,
        cancel: CancellationToken,
    ) -> AppResult<FullAnalysisOutput> {
        let started_at = Utc::now();
        let started_at_str = started_at.to_rfc3339();

        let mut result = self.run_phases(&app, project_path, config, &cancel).await;

        let finished_at = Utc::now();
        let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;
        if let Ok(output) = &mut result {
            output.summary.duration_ms = duration_ms;
        }

        match &result {
            Ok(output) => {
                self.history_repo
                    .record(NewHistoryEntry {
                        project_path: project_path.to_string_lossy().to_string(),
                        status: "completed".into(),
                        total_files: output.summary.total_files,
                        analyzed_files: output.summary.analyzed_files,
                        cache_hits: output.summary.cache_hits,
                        total_tokens: output.summary.total_tokens,
                        duration_ms,
                        error_summary: None,
                        started_at: started_at_str,
                        finished_at: finished_at.to_rfc3339(),
                    })
                    .ok();
                let _ = app.emit(events::COMPLETE, &output.summary);
            }
            Err(e) => {
                let status = if cancel.is_cancelled() { "cancelled" } else { "error" };
                self.history_repo
                    .record(NewHistoryEntry {
                        project_path: project_path.to_string_lossy().to_string(),
                        status: status.into(),
                        total_files: 0,
                        analyzed_files: 0,
                        cache_hits: 0,
                        total_tokens: 0,
                        duration_ms,
                        error_summary: Some(e.to_string()),
                        started_at: started_at_str,
                        finished_at: finished_at.to_rfc3339(),
                    })
                    .ok();
                if cancel.is_cancelled() {
                    let _ = app.emit(events::CANCELLED, serde_json::json!({}));
                } else {
                    let _ = app.emit(events::ERROR, e);
                }
            }
        }

        result
    }

    async fn run_phases(
        &self,
        app: &AppHandle,
        project_path: &Path,
        config: &AppConfig,
        cancel: &CancellationToken,
    ) -> AppResult<FullAnalysisOutput> {
        // --- Phase 1: scan ---
        Self::emit_progress(app, phases::SCAN, 0, 0, None);
        let scanner = Scanner::new(&config.scan);
        let scan_result = scanner.scan(project_path, cancel)?;
        Self::emit_phase_complete(app, phases::SCAN);
        check_cancelled(cancel)?;

        // --- Phase 2: static analysis (+ RPA構造解析) ---
        let mut static_results = Vec::new();
        let mut rpa_pairs: Vec<(crate::models::RpaComponent, String, String)> = Vec::new(); // (component, hash, raw_content)
        let rpa_analyzers = rpa_analyzer::all_analyzers();
        let total_scan = scan_result.total_count as u64;
        let mut processed: u64 = 0;

        for file in &scan_result.files {
            check_cancelled(cancel)?;
            let abs_path = project_path.join(&file.path);
            let Ok(content) = std::fs::read_to_string(&abs_path) else {
                processed += 1;
                continue; // 読み取り不能ファイルはスキップし継続（NFR-03）
            };

            if let Some(tool) = &file.rpa_tool {
                if let Some(analyzer) = rpa_analyzers.iter().find(|a| a.tool_name() == tool) {
                    if let Ok(component) = analyzer.parse(Path::new(&file.path), &content) {
                        rpa_pairs.push((component, file.hash.clone(), content.clone()));
                    }
                }
            } else {
                let ext = Path::new(&file.path).extension().and_then(|e| e.to_str()).unwrap_or("");
                if StaticAnalyzer::supports_extension(ext) {
                    if let Ok(result) = StaticAnalyzer::analyze(&file.path, &content, ext) {
                        static_results.push(result);
                    }
                }
            }

            processed += 1;
            Self::emit_progress(app, phases::STATIC, processed, total_scan, Some(file.path.clone()));
        }

        let dep_graph = dependency_graph::build(&static_results);
        Self::emit_phase_complete(app, phases::STATIC);
        check_cancelled(cancel)?;

        // --- Phase 3: AI analysis ---
        let chat_uid = self.ai_client.open_session(NEWTONX_CHAT_FOLDER_NAME).await?;
        let analyzer = Arc::new(AiAnalyzer::new(
            self.ai_client.clone(),
            self.cache_repo.clone(),
            config.ai.clone(),
            config.cache.ttl_days,
            chat_uid,
        ));

        let semaphore = Arc::new(Semaphore::new(config.ai.concurrency.max(1) as usize));
        let total_ai = (static_results.len() + rpa_pairs.len()) as u64;
        let mut tasks = tokio::task::JoinSet::new();

        for static_result in static_results.clone() {
            let permit_sem = semaphore.clone();
            let analyzer = analyzer.clone();
            let app = app.clone();
            let cancel = cancel.clone();
            let abs_path = project_path.join(&static_result.file_path);
            let hash = scan_result
                .files
                .iter()
                .find(|f| f.path == static_result.file_path)
                .map(|f| f.hash.clone())
                .unwrap_or_default();
            tasks.spawn(async move {
                let _permit = permit_sem.acquire_owned().await.ok();
                if cancel.is_cancelled() {
                    return None;
                }
                let content = std::fs::read_to_string(&abs_path).unwrap_or_default();
                let result = analyzer.analyze_file(&hash, &static_result, &content).await;
                let _ = app.emit(
                    events::FILE_COMPLETE,
                    serde_json::json!({ "filePath": static_result.file_path, "ok": result.is_ok() }),
                );
                result.ok()
            });
        }

        for (component, hash, raw_content) in rpa_pairs.clone() {
            let permit_sem = semaphore.clone();
            let analyzer = analyzer.clone();
            let app = app.clone();
            let cancel = cancel.clone();
            tasks.spawn(async move {
                let _permit = permit_sem.acquire_owned().await.ok();
                if cancel.is_cancelled() {
                    return None;
                }
                let path = component.component_path.clone();
                let result = analyzer.analyze_rpa_component(&hash, &component, &raw_content).await;
                let _ = app.emit(events::FILE_COMPLETE, serde_json::json!({ "filePath": path, "ok": result.is_ok() }));
                result.ok()
            });
        }

        let mut file_results = Vec::new();
        let mut ai_processed: u64 = 0;
        while let Some(joined) = tasks.join_next().await {
            ai_processed += 1;
            Self::emit_progress(app, phases::AI, ai_processed, total_ai, None);
            if let Ok(Some(result)) = joined {
                file_results.push(result);
            }
        }
        check_cancelled(cancel)?;

        let cache_hits = file_results.iter().filter(|r| r.cache_hit).count() as u64;

        let language_stats = language_stats_summary(&static_results);
        let file_summaries = file_summaries_text(&file_results);
        let project_hash = project_hash(project_path, &scan_result.files);

        let project_result = analyzer
            .analyze_project(
                &project_hash,
                &project_path.to_string_lossy(),
                static_results.len(),
                &language_stats,
                &dep_graph,
                &file_summaries,
            )
            .await?;

        Self::emit_phase_complete(app, phases::AI);
        self.ai_client.close_session(&analyzer.current_chat_uid().await).await.ok();

        // --- Phase 4: doc generation ---
        Self::emit_progress(app, phases::DOC_GEN, 0, 1, None);
        let rpa_components: Vec<crate::models::RpaComponent> = rpa_pairs.iter().map(|(c, _, _)| c.clone()).collect();
        let doc_output_dir = crate::domain::doc_generator::resolve_output_dir(project_path, &config.export.output_dir);
        crate::domain::doc_generator::generate_all(
            &doc_output_dir,
            &project_result,
            &static_results,
            &file_results,
            &rpa_components,
            &config.export,
        )?;
        Self::emit_phase_complete(app, phases::DOC_GEN);

        let summary = AnalysisSummary {
            total_files: scan_result.total_count as u64,
            analyzed_files: file_results.len() as u64,
            cache_hits,
            total_tokens: 0, // token_count集計は将来拡張（キャッシュ保存時のtoken_countをサマリ側でも保持する場合に対応）
            duration_ms: 0,  // run_full_analysisで確定させるため呼び出し元で上書き
            status: "completed".into(),
        };

        Ok(FullAnalysisOutput {
            summary,
            static_results,
            file_results,
            rpa_components,
            project_result,
            dependency_graph: dep_graph,
        })
    }
}

fn check_cancelled(cancel: &CancellationToken) -> AppResult<()> {
    if cancel.is_cancelled() {
        Err(AppError::scan("解析はキャンセルされました"))
    } else {
        Ok(())
    }
}

fn language_stats_summary(results: &[StaticAnalysisResult]) -> String {
    use std::collections::HashMap;
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for r in results {
        *counts.entry(r.language.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(lang, count)| format!("{lang}:{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn file_summaries_text(results: &[FileAnalysisResult]) -> String {
    let mut sorted: Vec<&FileAnalysisResult> = results.iter().collect();
    sorted.sort_by(|a, b| b.importance_score.cmp(&a.importance_score));
    sorted
        .iter()
        .map(|r| format!("- {} (重要度{}): {}", r.file_path, r.importance_score, r.role_summary))
        .collect::<Vec<_>>()
        .join("\n")
}

fn project_hash(project_path: &Path, files: &[crate::models::ScannedFile]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(project_path.to_string_lossy().as_bytes());
    let mut sorted: Vec<&crate::models::ScannedFile> = files.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    for f in sorted {
        hasher.update(f.path.as_bytes());
        hasher.update(f.hash.as_bytes());
    }
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
