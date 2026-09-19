// Tauri invoke ラッパー・イベント購読の一元定義（CLAUDE.md 原則2.2.1）。
// 04_実装詳細.md §7 のコマンドシグネチャに準拠した型付きラッパーのみをここに置き、
// 各画面から直接 invoke/listen を呼ばない。
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import i18n from "@/lib/i18n";
import { Events } from "@/lib/constants";
import type {
  AnalysisHistoryEntry,
  AnalysisResultSnapshot,
  AnalysisSummary,
  AppConfig,
  AssistantInfo,
  CacheStats,
  ExportResult,
  ProgressPayload,
} from "@/lib/types";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

interface AppErrorPayload {
  category?: string;
  message?: string;
  recoverable?: boolean;
  retryable?: boolean;
}

/** AppError（Rust側 thiserror + serde tag="category"）を表示用文字列に変換する（CLAUDE.md 原則2.2.1）。 */
export function formatInvokeError(error: unknown): string {
  const payload = error as AppErrorPayload;
  if (payload && typeof payload === "object" && typeof payload.message === "string") {
    const category = payload.category ? i18n.t(`errorCategory.${payload.category}`, payload.category) : null;
    return category ? `${category}: ${payload.message}` : payload.message;
  }
  return String(error ?? i18n.t("common.unknownError"));
}

// --- 解析 ---

export async function selectProjectFolder(): Promise<string | null> {
  return invoke<string | null>("select_project_folder");
}

export async function startFullAnalysis(projectPath: string): Promise<AnalysisSummary> {
  return invoke<AnalysisSummary>("start_full_analysis", { projectPath });
}

export async function getLastAnalysisResult(): Promise<AnalysisResultSnapshot | null> {
  return invoke<AnalysisResultSnapshot | null>("get_last_analysis_result");
}

export async function cancelAnalysis(): Promise<void> {
  return invoke<void>("cancel_analysis");
}

interface AnalysisEventHandlers {
  onProgress: (payload: ProgressPayload) => void;
  onPhaseComplete: (phase: string) => void;
  onComplete: (summary: AnalysisSummary) => void;
  onCancelled: () => void;
  onError: (payload: unknown) => void;
}

/** analysis://* イベントの一括購読（マウント時listen/アンマウント時解除、03_設計書.md §6.3）。 */
export async function subscribeAnalysisEvents(handlers: AnalysisEventHandlers): Promise<() => void> {
  const unlistenFns = await Promise.all([
    listen<ProgressPayload>(Events.PROGRESS, (e) => handlers.onProgress(e.payload)),
    listen<{ phase: string }>(Events.PHASE_COMPLETE, (e) => handlers.onPhaseComplete(e.payload.phase)),
    listen<AnalysisSummary>(Events.COMPLETE, (e) => handlers.onComplete(e.payload)),
    listen(Events.CANCELLED, () => handlers.onCancelled()),
    listen(Events.ERROR, (e) => handlers.onError(e.payload)),
  ]);
  return () => {
    unlistenFns.forEach((fn) => fn());
  };
}

// --- エクスポート ---

export async function selectOutputFolder(): Promise<string | null> {
  return invoke<string | null>("select_output_folder");
}

export interface ExportDocumentArgs {
  format: string;
  docTypes: string[];
  embedMermaid: boolean;
  outputDir?: string;
  [key: string]: unknown;
}

export async function exportDocument(args: ExportDocumentArgs): Promise<ExportResult> {
  return invoke<ExportResult>("export_document", args);
}

// --- NewtonX認証・接続 ---

export async function newtonxAuthStatus(): Promise<{ authenticated: boolean }> {
  return invoke<{ authenticated: boolean }>("newtonx_auth_status");
}

export async function saveNewtonxCredentials(host: string, personalAccessToken: string): Promise<{ success: boolean }> {
  return invoke<{ success: boolean }>("save_newtonx_credentials", { host, personalAccessToken });
}

export async function clearNewtonxCredentials(): Promise<void> {
  return invoke<void>("clear_newtonx_credentials");
}

export async function newtonxListAssistants(): Promise<AssistantInfo[]> {
  return invoke<AssistantInfo[]>("newtonx_list_assistants");
}

export async function testAiConnection(): Promise<{ ok: boolean; message: string }> {
  return invoke<{ ok: boolean; message: string }>("test_ai_connection");
}

// --- 設定 ---

export async function loadConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("load_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke<void>("save_config", { config });
}

export async function resetConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("reset_config");
}

// --- キャッシュ・履歴 ---

export async function getCacheStats(): Promise<CacheStats> {
  return invoke<CacheStats>("get_cache_stats");
}

export async function clearCache(): Promise<void> {
  return invoke<void>("clear_cache");
}

export async function getAnalysisHistory(limit?: number): Promise<AnalysisHistoryEntry[]> {
  return invoke<AnalysisHistoryEntry[]>("get_analysis_history", { limit });
}

export async function deleteAnalysisHistory(id: number): Promise<void> {
  return invoke<void>("delete_analysis_history", { id });
}
