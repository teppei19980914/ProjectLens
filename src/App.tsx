import { useTranslation } from "react-i18next";
import { isTauriRuntime } from "@/lib/tauri";
import { AnalyzeScreen } from "@/screens/AnalyzeScreen";
import { AuthScreen } from "@/screens/AuthScreen";
import { ExportScreen } from "@/screens/ExportScreen";
import { HistoryScreen } from "@/screens/HistoryScreen";
import { HomeScreen } from "@/screens/HomeScreen";
import { ResultScreen } from "@/screens/ResultScreen";
import { SettingsScreen } from "@/screens/SettingsScreen";
import { useUiStore } from "@/store/uiStore";

function NotInTauriRuntimeNotice() {
  const { t } = useTranslation();
  return (
    <div className="mx-auto flex max-w-lg flex-col gap-3 p-8">
      <h1 className="text-xl font-bold text-red-600">{t("common.notInTauriRuntime.title")}</h1>
      <p className="text-sm">{t("common.notInTauriRuntime.body")}</p>
      <p className="text-sm text-gray-500">{t("common.notInTauriRuntime.instruction")}</p>
    </div>
  );
}

function App() {
  const view = useUiStore((s) => s.view);

  return (
    <div className="min-h-screen bg-white text-gray-900 dark:bg-gray-950 dark:text-gray-100">
      {!isTauriRuntime() ? (
        <NotInTauriRuntimeNotice />
      ) : (
        <>
          {view === "home" && <HomeScreen />}
          {view === "analyze" && <AnalyzeScreen />}
          {view === "result" && <ResultScreen />}
          {view === "export" && <ExportScreen />}
          {view === "settings" && <SettingsScreen />}
          {view === "auth" && <AuthScreen />}
          {view === "history" && <HistoryScreen />}
        </>
      )}
    </div>
  );
}

export default App;
