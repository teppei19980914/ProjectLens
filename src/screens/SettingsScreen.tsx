import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { loadConfig, resetConfig, saveConfig } from "@/lib/tauri";
import type { AppConfig } from "@/lib/types";
import { useUiStore } from "@/store/uiStore";

export function SettingsScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    loadConfig().then(setConfig);
  }, []);

  if (!config) {
    return <div className="p-8 text-sm text-gray-500">{t("common.loading")}</div>;
  }

  const update = (updater: (draft: AppConfig) => AppConfig) => {
    setConfig((prev) => (prev ? updater(structuredClone(prev)) : prev));
    setSaved(false);
  };

  const handleSave = async () => {
    await saveConfig(config);
    setSaved(true);
  };

  const handleReset = async () => {
    if (!window.confirm(t("settings.resetConfirm"))) return;
    const defaults = await resetConfig();
    setConfig(defaults);
    setSaved(true);
  };

  return (
    <div className="mx-auto flex max-w-2xl flex-col gap-6 p-8">
      <h1 className="text-2xl font-bold">{t("settings.title")}</h1>

      <Card className="flex flex-col gap-3">
        <h2 className="font-semibold">{t("settings.sections.scan")}</h2>
        <label className="flex items-center justify-between text-sm">
          {t("settings.scan.maxFileSizeKb")}
          <input
            type="number"
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.scan.maxFileSizeKb}
            onChange={(e) => update((d) => ({ ...d, scan: { ...d.scan, maxFileSizeKb: Number(e.target.value) } }))}
          />
        </label>
        <label className="flex items-center justify-between text-sm">
          {t("settings.scan.maxFileCount")}
          <input
            type="number"
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.scan.maxFileCount}
            onChange={(e) => update((d) => ({ ...d, scan: { ...d.scan, maxFileCount: Number(e.target.value) } }))}
          />
        </label>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={config.scan.rpa.enabled}
            onChange={(e) => update((d) => ({ ...d, scan: { ...d.scan, rpa: { ...d.scan.rpa, enabled: e.target.checked } } }))}
          />
          {t("settings.scan.rpaEnabled")}
        </label>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={config.scan.vba.enabled}
            onChange={(e) => update((d) => ({ ...d, scan: { ...d.scan, vba: { ...d.scan.vba, enabled: e.target.checked } } }))}
          />
          {t("settings.scan.vbaEnabled")}
        </label>
      </Card>

      <Card className="flex flex-col gap-3">
        <h2 className="font-semibold">{t("settings.sections.ai")}</h2>
        <label className="flex items-center justify-between text-sm">
          {t("settings.ai.concurrency")}
          <input
            type="number"
            min={1}
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.ai.concurrency}
            onChange={(e) => update((d) => ({ ...d, ai: { ...d.ai, concurrency: Number(e.target.value) } }))}
          />
        </label>
        <label className="flex items-center justify-between text-sm">
          {t("settings.ai.timeoutSecs")}
          <input
            type="number"
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.ai.timeoutSecs}
            onChange={(e) => update((d) => ({ ...d, ai: { ...d.ai, timeoutSecs: Number(e.target.value) } }))}
          />
        </label>
        <label className="flex items-center justify-between text-sm">
          {t("settings.ai.maxRetries")}
          <input
            type="number"
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.ai.maxRetries}
            onChange={(e) => update((d) => ({ ...d, ai: { ...d.ai, maxRetries: Number(e.target.value) } }))}
          />
        </label>
        <label className="flex items-center justify-between text-sm">
          {t("settings.ai.maxTokensPerFile")}
          <input
            type="number"
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.ai.maxTokensPerFile}
            onChange={(e) => update((d) => ({ ...d, ai: { ...d.ai, maxTokensPerFile: Number(e.target.value) } }))}
          />
        </label>
        <label className="flex flex-col gap-1 text-sm">
          {t("settings.ai.pythonExe")}
          <input
            type="text"
            className="w-full rounded border px-2 py-1 dark:bg-gray-800"
            value={config.ai.pythonExe}
            onChange={(e) => update((d) => ({ ...d, ai: { ...d.ai, pythonExe: e.target.value } }))}
          />
        </label>
      </Card>

      <Card className="flex flex-col gap-3">
        <h2 className="font-semibold">{t("settings.sections.cache")}</h2>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={config.cache.enabled}
            onChange={(e) => update((d) => ({ ...d, cache: { ...d.cache, enabled: e.target.checked } }))}
          />
          {t("settings.cache.enabled")}
        </label>
        <label className="flex items-center justify-between text-sm">
          {t("settings.cache.ttlDays")}
          <input
            type="number"
            className="w-32 rounded border px-2 py-1 dark:bg-gray-800"
            value={config.cache.ttlDays}
            onChange={(e) => update((d) => ({ ...d, cache: { ...d.cache, ttlDays: Number(e.target.value) } }))}
          />
        </label>
      </Card>

      <div className="flex gap-2">
        <Button onClick={handleSave}>{t("common.save")}</Button>
        <Button variant="secondary" onClick={handleReset}>
          {t("common.reset")}
        </Button>
        <Button variant="ghost" onClick={() => navigate("home")}>
          {t("common.back")}
        </Button>
      </div>
      {saved && <p className="text-sm text-green-600">{t("settings.saved")}</p>}
    </div>
  );
}
