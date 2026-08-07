import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { Tabs } from "@/components/ui/Tabs";
import { DocTypes } from "@/lib/constants";
import { useAnalysisStore } from "@/store/analysisStore";
import { useUiStore } from "@/store/uiStore";

export function ResultScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const result = useAnalysisStore((s) => s.result);
  const [tab, setTab] = useState<string>(DocTypes.SYSTEM_SPEC);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  if (!result) {
    return (
      <div className="mx-auto max-w-2xl p-8">
        <p className="text-gray-500">{t("result.noSelection")}</p>
        <Button variant="secondary" onClick={() => navigate("home")} className="mt-4">
          {t("common.back")}
        </Button>
      </div>
    );
  }

  const { systemSpec } = result.projectResult;
  const { basicDesign } = result.projectResult;
  const sortedFiles = [...result.fileResults].sort((a, b) => b.importanceScore - a.importanceScore);
  const selected = sortedFiles.find((f) => f.filePath === selectedFile) ?? null;

  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-4 p-8">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">{t("result.title")}</h1>
        <Button onClick={() => navigate("export")}>{t("export.title")}</Button>
      </div>

      <Tabs
        items={[
          { key: DocTypes.SYSTEM_SPEC, label: t("result.tabs.systemSpec") },
          { key: DocTypes.BASIC_DESIGN, label: t("result.tabs.basicDesign") },
          { key: DocTypes.DETAIL_DESIGN, label: t("result.tabs.detailDesign") },
        ]}
        active={tab}
        onChange={setTab}
      />

      {tab === DocTypes.SYSTEM_SPEC && (
        <Card className="flex flex-col gap-4">
          <section>
            <h2 className="font-semibold">{t("result.systemSpec.purpose")}</h2>
            <p className="text-sm">{systemSpec.purpose}</p>
          </section>
          <section>
            <h2 className="font-semibold">{t("result.systemSpec.mainFeatures")}</h2>
            <ul className="list-disc pl-5 text-sm">
              {systemSpec.mainFeatures.map((f) => (
                <li key={f.name}>
                  <strong>{f.name}</strong>: {f.description}
                </li>
              ))}
            </ul>
          </section>
          <section>
            <h2 className="font-semibold">{t("result.systemSpec.userFlows")}</h2>
            <p className="text-sm">{systemSpec.userFlows}</p>
          </section>
        </Card>
      )}

      {tab === DocTypes.BASIC_DESIGN && (
        <Card className="flex flex-col gap-4">
          <section>
            <h2 className="font-semibold">{t("result.basicDesign.architecturePattern")}</h2>
            <p className="text-sm">{basicDesign.architecturePattern}</p>
          </section>
          <section>
            <h2 className="font-semibold">{t("result.basicDesign.modules")}</h2>
            <ul className="list-disc pl-5 text-sm">
              {basicDesign.modules.map((m) => (
                <li key={m.name}>
                  <strong>{m.name}</strong>: {m.responsibility}
                </li>
              ))}
            </ul>
          </section>
          <section>
            <h2 className="font-semibold">{t("result.basicDesign.technicalDebts")}</h2>
            {basicDesign.technicalDebts.length === 0 ? (
              <p className="text-sm text-gray-500">{t("result.basicDesign.noneDetected")}</p>
            ) : (
              <ul className="list-disc pl-5 text-sm">
                {basicDesign.technicalDebts.map((d, i) => (
                  <li key={i}>
                    [{d.severity}] {d.area}: {d.description}
                  </li>
                ))}
              </ul>
            )}
          </section>
        </Card>
      )}

      {tab === DocTypes.DETAIL_DESIGN && (
        <div className="grid grid-cols-3 gap-4">
          <Card className="col-span-1 max-h-[60vh] overflow-y-auto">
            <ul className="flex flex-col gap-1 text-sm">
              {sortedFiles.map((f) => (
                <li key={f.filePath}>
                  <button
                    onClick={() => setSelectedFile(f.filePath)}
                    className={`w-full truncate rounded px-2 py-1 text-left hover:bg-gray-100 dark:hover:bg-gray-800 ${
                      selectedFile === f.filePath ? "bg-gray-100 dark:bg-gray-800" : ""
                    }`}
                    title={f.filePath}
                  >
                    <span className="mr-2 inline-block w-6 text-right text-gray-400">{f.importanceScore}</span>
                    {f.filePath}
                  </button>
                </li>
              ))}
            </ul>
          </Card>
          <Card className="col-span-2">
            {!selected ? (
              <p className="text-sm text-gray-500">{t("result.noSelection")}</p>
            ) : (
              <div className="flex flex-col gap-3 text-sm">
                <h2 className="font-semibold">{selected.filePath}</h2>
                <p>{selected.roleSummary}</p>
                <p>
                  {t("result.importanceScore")}: {selected.importanceScore}/10
                </p>
                {selected.publicApis.length > 0 && (
                  <div>
                    <h3 className="font-semibold">{t("result.detail.publicApis")}</h3>
                    <ul className="list-disc pl-5">
                      {selected.publicApis.map((a) => (
                        <li key={a.name}>
                          {a.name} ({a.kind}): {a.description}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
                {selected.potentialIssues.length > 0 && (
                  <div>
                    <h3 className="font-semibold">{t("result.detail.potentialIssues")}</h3>
                    <ul className="list-disc pl-5">
                      {selected.potentialIssues.map((issue, i) => (
                        <li key={i}>
                          [{issue.severity}] {issue.description}（{issue.suggestion}）
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            )}
          </Card>
        </div>
      )}
    </div>
  );
}
