import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { Phases } from "@/lib/constants";
import { cancelAnalysis, formatInvokeError, getLastAnalysisResult, startFullAnalysis, subscribeAnalysisEvents } from "@/lib/tauri";
import { useAnalysisStore } from "@/store/analysisStore";
import { useUiStore } from "@/store/uiStore";

const PHASE_ORDER = [Phases.SCAN, Phases.STATIC, Phases.AI, Phases.DOC_GEN] as const;

export function AnalyzeScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const store = useAnalysisStore();
  const startedRef = useRef(false);

  useEffect(() => {
    if (!store.projectPath || startedRef.current) return;
    startedRef.current = true;

    let unlisten: (() => void) | undefined;

    subscribeAnalysisEvents({
      onProgress: (p) => store.updateProgress(p.phase, p.processed, p.total, p.currentFile),
      // フェーズ切替時に前フェーズの「処理中のファイル」表示が残り続けないよう明示的にクリアする
      // （AI解析フェーズはファイル単位のprogressイベントを出さないため、スキャン/静的解析時の
      // 最後のファイル名が誤って表示され続ける不具合があった）
      onPhaseComplete: () => store.clearCurrentFile(),
      onComplete: async (summary) => {
        const result = await getLastAnalysisResult().catch(() => null);
        store.finishSuccess(summary, result);
        navigate("result");
      },
      onCancelled: () => {
        store.finishCancelled();
        navigate("home");
      },
      onError: (payload) => {
        store.finishError(formatInvokeError(payload));
      },
    }).then((fn) => {
      unlisten = fn;
    });

    startFullAnalysis(store.projectPath).catch((e) => {
      store.finishError(formatInvokeError(e));
    });

    return () => {
      unlisten?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [store.projectPath]);

  const handleCancel = async () => {
    await cancelAnalysis();
  };

  const percent = store.total > 0 ? Math.round((store.processed / store.total) * 100) : 0;

  return (
    <div className="mx-auto flex max-w-2xl flex-col gap-6 p-8">
      <h1 className="text-2xl font-bold">{t("analysis.title")}</h1>

      <Card className="flex flex-col gap-4">
        <div className="flex flex-col gap-2">
          {PHASE_ORDER.map((phase) => {
            const isActive = store.phase === phase;
            const isDone = PHASE_ORDER.indexOf(phase) < PHASE_ORDER.indexOf(store.phase ?? PHASE_ORDER[0]) || (!store.isRunning && store.summary);
            return (
              <div key={phase} className="flex items-center gap-3">
                <span
                  className={
                    isActive
                      ? "h-2 w-2 rounded-full bg-blue-600"
                      : isDone
                        ? "h-2 w-2 rounded-full bg-green-500"
                        : "h-2 w-2 rounded-full bg-gray-300"
                  }
                />
                <span className={isActive ? "font-semibold" : "text-gray-500"}>{t(`analysis.phase.${phase}`)}</span>
              </div>
            );
          })}
        </div>

        {store.isRunning && store.total > 0 && (
          <div>
            <div className="h-2 w-full overflow-hidden rounded bg-gray-200 dark:bg-gray-800">
              <div className="h-full bg-blue-600 transition-all" style={{ width: `${percent}%` }} />
            </div>
            <p className="mt-1 text-sm text-gray-500">
              {t("analysis.processedOf", { processed: store.processed, total: store.total })}
            </p>
          </div>
        )}

        {store.currentFile && <p className="truncate text-xs text-gray-400">{t("analysis.currentFile")}: {store.currentFile}</p>}

        {store.errorMessage && <p className="text-sm text-red-600">{store.errorMessage}</p>}

        {store.isRunning && (
          <Button variant="danger" onClick={handleCancel}>
            {t("analysis.cancel")}
          </Button>
        )}
      </Card>
    </div>
  );
}
