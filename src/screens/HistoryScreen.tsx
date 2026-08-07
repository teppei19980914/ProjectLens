import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { deleteAnalysisHistory, getAnalysisHistory } from "@/lib/tauri";
import type { AnalysisHistoryEntry } from "@/lib/types";
import { useUiStore } from "@/store/uiStore";

export function HistoryScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const [entries, setEntries] = useState<AnalysisHistoryEntry[]>([]);

  const load = () => {
    getAnalysisHistory().then(setEntries);
  };

  useEffect(load, []);

  const handleDelete = async (id: number) => {
    if (!window.confirm(t("history.deleteConfirm"))) return;
    await deleteAnalysisHistory(id);
    load();
  };

  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-6 p-8">
      <h1 className="text-2xl font-bold">{t("history.title")}</h1>

      {entries.length === 0 ? (
        <p className="text-sm text-gray-500">{t("history.empty")}</p>
      ) : (
        <Card className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b text-left text-gray-500">
                <th className="py-1 pr-4">{t("history.columns.startedAt")}</th>
                <th className="py-1 pr-4">{t("history.columns.status")}</th>
                <th className="py-1 pr-4">{t("history.columns.totalTokens")}</th>
                <th className="py-1 pr-4">{t("history.columns.durationMs")}</th>
                <th className="py-1"></th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <tr key={entry.id} className="border-b last:border-0">
                  <td className="py-1 pr-4">{entry.startedAt}</td>
                  <td className="py-1 pr-4">{t(`history.status.${entry.status}`, entry.status)}</td>
                  <td className="py-1 pr-4">{entry.totalTokens}</td>
                  <td className="py-1 pr-4">{entry.durationMs}ms</td>
                  <td className="py-1">
                    <Button variant="ghost" onClick={() => handleDelete(entry.id)}>
                      {t("common.delete")}
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
      )}

      <Button variant="secondary" onClick={() => navigate("home")}>
        {t("common.back")}
      </Button>
    </div>
  );
}
