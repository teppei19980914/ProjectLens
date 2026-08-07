import { create } from "zustand";
import type { Phase } from "@/lib/constants";
import type { AnalysisResultSnapshot, AnalysisSummary } from "@/lib/types";

interface AnalysisState {
  projectPath: string | null;
  isRunning: boolean;
  phase: Phase | null;
  processed: number;
  total: number;
  currentFile: string | null;
  summary: AnalysisSummary | null;
  result: AnalysisResultSnapshot | null;
  errorMessage: string | null;

  setProjectPath: (path: string | null) => void;
  startRun: (path: string) => void;
  updateProgress: (phase: Phase, processed: number, total: number, currentFile?: string | null) => void;
  clearCurrentFile: () => void;
  finishSuccess: (summary: AnalysisSummary, result: AnalysisResultSnapshot | null) => void;
  finishError: (message: string) => void;
  finishCancelled: () => void;
  reset: () => void;
}

export const useAnalysisStore = create<AnalysisState>((set) => ({
  projectPath: null,
  isRunning: false,
  phase: null,
  processed: 0,
  total: 0,
  currentFile: null,
  summary: null,
  result: null,
  errorMessage: null,

  setProjectPath: (path) => set({ projectPath: path }),
  startRun: (path) =>
    set({
      projectPath: path,
      isRunning: true,
      phase: null,
      processed: 0,
      total: 0,
      currentFile: null,
      summary: null,
      errorMessage: null,
    }),
  updateProgress: (phase, processed, total, currentFile) => set({ phase, processed, total, currentFile: currentFile ?? null }),
  clearCurrentFile: () => set({ currentFile: null }),
  finishSuccess: (summary, result) => set({ isRunning: false, summary, result, errorMessage: null }),
  finishError: (message) => set({ isRunning: false, errorMessage: message }),
  finishCancelled: () => set({ isRunning: false, errorMessage: null }),
  reset: () =>
    set({
      projectPath: null,
      isRunning: false,
      phase: null,
      processed: 0,
      total: 0,
      currentFile: null,
      summary: null,
      result: null,
      errorMessage: null,
    }),
}));
