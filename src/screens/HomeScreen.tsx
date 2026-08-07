import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { selectProjectFolder } from "@/lib/tauri";
import { useAnalysisStore } from "@/store/analysisStore";
import { useUiStore } from "@/store/uiStore";

export function HomeScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const startRun = useAnalysisStore((s) => s.startRun);

  const handleSelectFolder = async () => {
    const path = await selectProjectFolder();
    if (path) {
      startRun(path);
      navigate("analyze");
    }
  };

  return (
    <div className="mx-auto flex max-w-2xl flex-col gap-6 p-8">
      <div>
        <h1 className="text-2xl font-bold">{t("common.appName")}</h1>
      </div>

      <Card className="flex flex-col items-start gap-4">
        <Button onClick={handleSelectFolder}>{t("home.selectFolder")}</Button>
      </Card>

      <Card>
        <h2 className="mb-2 text-sm font-semibold text-gray-500">{t("home.recentProjects")}</h2>
        <p className="text-sm text-gray-400">{t("home.noRecentProjects")}</p>
      </Card>

      <div className="flex gap-2">
        <Button variant="secondary" onClick={() => navigate("history")}>
          {t("home.openHistory")}
        </Button>
        <Button variant="secondary" onClick={() => navigate("settings")}>
          {t("home.openSettings")}
        </Button>
        <Button variant="secondary" onClick={() => navigate("auth")}>
          {t("auth.title")}
        </Button>
      </div>
    </div>
  );
}
