// 解析結果・設定の共通型定義。src-tauri/src/models/{analysis.rs,config.rs} の
// serde(camelCase) 出力と1:1で対応させる（CLAUDE.md 原則2.1.3、単一の参照元）。
import type { Phase } from "@/lib/constants";

// ---------------------------------------------------------------------
// 設定 (config.rs)
// ---------------------------------------------------------------------

export interface RpaDetectionPatterns {
  powerplatform: string[];
  pad: string[];
  uipath: string[];
}

export interface RpaScanConfig {
  enabled: boolean;
  tools: string[];
  detectionPatterns: RpaDetectionPatterns;
  maxFileSizeKb: number;
}

export interface VbaScanConfig {
  enabled: boolean;
  extensions: string[];
  maxFileSizeKb: number;
}

export interface ScanConfig {
  excludePatterns: string[];
  maxFileSizeKb: number;
  maxFileCount: number;
  extensions: string[];
  rpa: RpaScanConfig;
  vba: VbaScanConfig;
}

export interface NewtonxConfig {
  host: string;
  assistantUid: string;
}

export interface AiConfig {
  provider: string;
  newtonx: NewtonxConfig;
  concurrency: number;
  timeoutSecs: number;
  maxRetries: number;
  retryAfterSecs: number;
  retryAfterMaxSecs: number;
  detailLevel: string;
  maxTokensPerFile: number;
  outputLanguage: string;
  pythonExe: string;
}

export interface CacheConfig {
  enabled: boolean;
  ttlDays: number;
  maxSizeMb: number;
  autoCleanup: boolean;
}

export interface ExportConfig {
  defaultFormat: string;
  defaultDocTypes: string[];
  outputDir: string;
  embedMermaid: boolean;
}

export interface UiConfig {
  theme: string;
  language: string;
  recentProjectsCount: number;
}

export interface AppConfig {
  scan: ScanConfig;
  ai: AiConfig;
  cache: CacheConfig;
  export: ExportConfig;
  ui: UiConfig;
}

// ---------------------------------------------------------------------
// ファイル単位AI解析結果 (analysis.rs)
// ---------------------------------------------------------------------

export type Severity = "high" | "medium" | "low";

export interface PublicApi {
  name: string;
  kind: string;
  description: string;
}

export interface Issue {
  severity: Severity;
  description: string;
  suggestion: string;
}

export interface FileAnalysisResult {
  filePath: string;
  roleSummary: string;
  publicApis: PublicApi[];
  designPatterns: string[];
  importanceScore: number;
  potentialIssues: Issue[];
  cacheHit: boolean;
}

// ---------------------------------------------------------------------
// プロジェクト全体AI解析結果 (analysis.rs)
// ---------------------------------------------------------------------

export interface FeatureSummary {
  name: string;
  description: string;
}

export interface SystemSpec {
  purpose: string;
  mainFeatures: FeatureSummary[];
  userFlows: string;
}

export interface ModuleSummary {
  name: string;
  responsibility: string;
  keyFiles: string[];
}

export interface TechnicalDebt {
  severity: Severity;
  area: string;
  description: string;
  recommendation: string;
}

export interface MermaidDiagram {
  type: string;
  title: string;
  source: string;
}

export interface BasicDesign {
  architecturePattern: string;
  modules: ModuleSummary[];
  techStack: string[];
  dataFlow: string;
  technicalDebts: TechnicalDebt[];
  mermaidDiagrams: MermaidDiagram[];
}

export interface ProjectAnalysisResult {
  systemSpec: SystemSpec;
  basicDesign: BasicDesign;
}

// ---------------------------------------------------------------------
// 進捗イベント (analysis.rs::ProgressPayload)
// ---------------------------------------------------------------------

export interface ProgressPayload {
  phase: Phase;
  processed: number;
  total: number;
  currentFile: string | null;
}

// ---------------------------------------------------------------------
// キャッシュ・履歴
// ---------------------------------------------------------------------

export interface CacheStats {
  fileEntries: number;
  projectEntries: number;
  sizeMb: number;
}

export interface AnalysisHistoryEntry {
  id: number;
  projectPath: string;
  status: "completed" | "cancelled" | "error";
  totalFiles: number;
  analyzedFiles: number;
  cacheHits: number;
  totalTokens: number;
  durationMs: number;
  errorSummary: string | null;
  startedAt: string;
  finishedAt: string | null;
}

// ---------------------------------------------------------------------
// ドキュメント出力
// ---------------------------------------------------------------------

export interface ExportedFile {
  docType: string;
  path: string;
  sourceFilePath: string | null;
}

export interface ExportResult {
  outputDir: string;
  files: ExportedFile[];
}

export interface AnalysisResultSnapshot {
  projectResult: ProjectAnalysisResult;
  fileResults: FileAnalysisResult[];
}

export interface AnalysisSummary {
  totalFiles: number;
  analyzedFiles: number;
  cacheHits: number;
  totalTokens: number;
  durationMs: number;
  status: string;
}

// ---------------------------------------------------------------------
// NewtonXアシスタント一覧
// ---------------------------------------------------------------------

export interface AssistantInfo {
  uid: string;
  name: string;
}
