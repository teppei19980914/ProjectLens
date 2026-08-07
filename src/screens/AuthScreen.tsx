import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { clearNewtonxCredentials, loadConfig, newtonxAuthStatus, saveNewtonxCredentials, testAiConnection } from "@/lib/tauri";
import { useUiStore } from "@/store/uiStore";

export function AuthScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const [host, setHost] = useState("");
  const [pat, setPat] = useState("");
  const [authenticated, setAuthenticated] = useState<boolean | null>(null);
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string } | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    loadConfig().then((c) => setHost(c.ai.newtonx.host));
    newtonxAuthStatus()
      .then((s) => setAuthenticated(s.authenticated))
      .catch(() => setAuthenticated(false));
  }, []);

  const handleSave = async () => {
    setBusy(true);
    try {
      await saveNewtonxCredentials(host, pat);
      const status = await newtonxAuthStatus();
      setAuthenticated(status.authenticated);
      setPat("");
    } finally {
      setBusy(false);
    }
  };

  const handleClear = async () => {
    setBusy(true);
    try {
      await clearNewtonxCredentials();
      setAuthenticated(false);
    } finally {
      setBusy(false);
    }
  };

  const handleTest = async () => {
    setBusy(true);
    try {
      const result = await testAiConnection();
      setTestResult(result);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto flex max-w-xl flex-col gap-6 p-8">
      <h1 className="text-2xl font-bold">{t("auth.title")}</h1>

      <Card className="flex flex-col gap-4">
        <p className="text-sm">
          {authenticated ? t("auth.status.authenticated") : t("auth.status.notAuthenticated")}
        </p>

        <div>
          <label className="mb-1 block text-sm font-semibold">{t("auth.host")}</label>
          <input
            type="text"
            value={host}
            onChange={(e) => setHost(e.target.value)}
            placeholder={t("auth.hostPlaceholder")}
            className="w-full rounded border px-2 py-1 text-sm dark:bg-gray-800"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-semibold">{t("auth.personalAccessToken")}</label>
          <input
            type="password"
            value={pat}
            onChange={(e) => setPat(e.target.value)}
            placeholder={t("auth.patPlaceholder")}
            className="w-full rounded border px-2 py-1 text-sm dark:bg-gray-800"
          />
        </div>

        <div className="flex gap-2">
          <Button onClick={handleSave} disabled={busy || !host || !pat}>
            {t("auth.save")}
          </Button>
          <Button variant="secondary" onClick={handleClear} disabled={busy}>
            {t("auth.clear")}
          </Button>
          <Button variant="secondary" onClick={handleTest} disabled={busy}>
            {t("auth.testConnection")}
          </Button>
        </div>

        {testResult && (
          <p className={testResult.ok ? "text-sm text-green-600" : "text-sm text-red-600"}>
            {testResult.ok ? t("auth.testSuccess") : t("auth.testFailure")}: {testResult.message}
          </p>
        )}
      </Card>

      <Button variant="ghost" onClick={() => navigate("home")}>
        {t("common.back")}
      </Button>
    </div>
  );
}
