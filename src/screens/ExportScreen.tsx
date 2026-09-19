import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Card } from "@/components/ui/Card";
import { DocTypes, ExportFormats, type DocType, type ExportFormat } from "@/lib/constants";
import { exportDocument, selectOutputFolder } from "@/lib/tauri";
import { useUiStore } from "@/store/uiStore";

const ALL_DOC_TYPES: DocType[] = [DocTypes.SYSTEM_SPEC, DocTypes.BASIC_DESIGN, DocTypes.DETAIL_DESIGN, DocTypes.DIRECTORY_STRUCTURE];
const ALL_FORMATS: ExportFormat[] = [ExportFormats.MARKDOWN, ExportFormats.HTML, ExportFormats.JSON];

const DOC_TYPE_LABEL_KEYS: Record<DocType, string> = {
  [DocTypes.SYSTEM_SPEC]: "result.tabs.systemSpec",
  [DocTypes.BASIC_DESIGN]: "result.tabs.basicDesign",
  [DocTypes.DETAIL_DESIGN]: "result.tabs.detailDesign",
  [DocTypes.DIRECTORY_STRUCTURE]: "result.tabs.directoryStructure",
};

export function ExportScreen() {
  const { t } = useTranslation();
  const navigate = useUiStore((s) => s.navigate);
  const [docTypes, setDocTypes] = useState<DocType[]>(ALL_DOC_TYPES);
  const [format, setFormat] = useState<ExportFormat>(ExportFormats.MARKDOWN);
  const [embedMermaid, setEmbedMermaid] = useState(true);
  const [outputDir, setOutputDir] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [resultDir, setResultDir] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const toggleDocType = (docType: DocType) => {
    setDocTypes((prev) => (prev.includes(docType) ? prev.filter((d) => d !== docType) : [...prev, docType]));
  };

  const handleChooseOutputDir = async () => {
    const path = await selectOutputFolder();
    if (path) setOutputDir(path);
  };

  const handleExport = async () => {
    setStatus("loading");
    try {
      const result = await exportDocument({ format, docTypes, embedMermaid, outputDir: outputDir || undefined });
      setResultDir(result.outputDir);
      setStatus("success");
    } catch (e) {
      setErrorMessage(String(e));
      setStatus("error");
    }
  };

  return (
    <div className="mx-auto flex max-w-xl flex-col gap-6 p-8">
      <h1 className="text-2xl font-bold">{t("export.title")}</h1>

      <Card className="flex flex-col gap-4">
        <div>
          <h2 className="mb-2 text-sm font-semibold">{t("export.docTypes")}</h2>
          {ALL_DOC_TYPES.map((docType) => (
            <label key={docType} className="flex items-center gap-2 text-sm">
              <input type="checkbox" checked={docTypes.includes(docType)} onChange={() => toggleDocType(docType)} />
              {t(DOC_TYPE_LABEL_KEYS[docType])}
            </label>
          ))}
        </div>

        <div>
          <h2 className="mb-2 text-sm font-semibold">{t("export.format")}</h2>
          <select value={format} onChange={(e) => setFormat(e.target.value as ExportFormat)} className="rounded border px-2 py-1 text-sm dark:bg-gray-800">
            {ALL_FORMATS.map((f) => (
              <option key={f} value={f}>
                {f}
              </option>
            ))}
          </select>
        </div>

        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={embedMermaid} onChange={(e) => setEmbedMermaid(e.target.checked)} />
          {t("export.embedMermaid")}
        </label>

        <div>
          <h2 className="mb-2 text-sm font-semibold">{t("export.outputDir")}</h2>
          <div className="flex gap-2">
            <input
              type="text"
              value={outputDir}
              onChange={(e) => setOutputDir(e.target.value)}
              placeholder="projectlens-docs/"
              className="w-full rounded border px-2 py-1 text-sm dark:bg-gray-800"
            />
            <Button variant="secondary" onClick={handleChooseOutputDir}>
              {t("export.chooseOutputDir")}
            </Button>
          </div>
        </div>

        <Button onClick={handleExport} disabled={status === "loading" || docTypes.length === 0}>
          {status === "loading" ? t("export.exporting") : t("export.exportButton")}
        </Button>

        {status === "success" && (
          <p className="text-sm text-green-600">
            {t("export.exportSuccess")}: {resultDir}
          </p>
        )}
        {status === "error" && <p className="text-sm text-red-600">{t("export.exportError")}: {errorMessage}</p>}
      </Card>

      <Button variant="secondary" onClick={() => navigate("result")}>
        {t("common.back")}
      </Button>
    </div>
  );
}
