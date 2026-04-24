import { useGraphStore } from "../store/graphStore";
import { useI18n } from "../i18n";
import { buildRepairPathInsight } from "../utils/repairPathInsights";

function severityLabel(severity) {
  if (severity === "error") return "Blocker";
  if (severity === "warning") return "Warning";
  if (severity === "info") return "Info";
  return "Blocker";
}

function severityTone(severity) {
  if (severity === "warning") return "warning";
  if (severity === "info") return "info";
  return "danger";
}

function sourceLabel(source, t) {
  if (source === "strategy_ir") return t("Strategy IR");
  if (source === "formal_quantscript") return t("Formal QuantScript");
  if (source === "runtime") return t("Runtime");
  return t("Strategy graph");
}

export default function DiagnosticsPanel({
  compileSummary,
  onRouteDiagnostic = null,
  graph = null,
  repairPathState = null
}) {
  const { t } = useI18n();
  const focusCompileDiagnostic = useGraphStore((state) => state.focusCompileDiagnostic);
  const diagnostics = Array.isArray(compileSummary?.diagnostics)
    ? compileSummary.diagnostics
    : [];

  return (
    <div className="property-card diagnostics-card" data-testid="diagnostics-panel">
      <div className="property-card-heading">
        <div className="property-card-title" data-testid="diagnostics-panel-title">{t("Compile diagnostics")}</div>
        <div className="property-card-caption">
          {t(
            "Keep severity, source, target, and active repair-path context in one readable stream."
          )}
        </div>
      </div>

      {diagnostics.length === 0 ? (
        <div className="empty-state diagnostics-empty" data-testid="diagnostics-empty">{t("No structured compile diagnostics yet.")}</div>
      ) : (
        <div className="diagnostics-list" data-testid="diagnostics-list">
          {diagnostics.map((diagnostic, index) => {
            const pathInsight = buildRepairPathInsight(
              diagnostic?.target || null,
              graph,
              repairPathState
            );
            const repairPathInsight = pathInsight
              ? {
                  ...pathInsight,
                  note: pathInsight.note.replaceAll("item", "diagnostic")
                }
              : null;

            return (
              <button
                key={`${diagnostic.code || "diag"}_${index}`}
                type="button"
                data-testid={`diagnostics-row-${diagnostic.code || index}`}
                className={`issue-row issue-${diagnostic.severity || "error"} diagnostics-row${
                  diagnostic.target ? " diagnostic-actionable" : ""
                }${repairPathInsight ? " diagnostics-row--path" : ""}`}
                onClick={() => {
                  if (diagnostic.target) {
                    focusCompileDiagnostic(diagnostic.target);
                    onRouteDiagnostic?.(diagnostic);
                  }
                }}
              >
                <div className="diagnostic-meta" data-testid={`diagnostics-meta-${diagnostic.code || index}`}>
                  <span className={`status-pill ${severityTone(diagnostic.severity)}`}>
                    {severityLabel(diagnostic.severity)}
                  </span>
                  <span className="diagnostic-chip">{diagnostic.code || "COMPILE_DIAGNOSTIC"}</span>
                  <span className="diagnostic-chip">{sourceLabel(diagnostic.source, t)}</span>
                  {diagnostic.target?.label ? (
                    <span className="diagnostic-chip">{diagnostic.target.label}</span>
                  ) : null}
                  {diagnostic.target ? (
                    <span className="diagnostic-chip">{t("Actionable target")}</span>
                  ) : null}
                  {repairPathInsight ? (
                    <>
                      <span className="diagnostic-chip diagnostic-chip--path">
                        {repairPathInsight.chip}
                      </span>
                      <span className="diagnostic-chip diagnostic-chip--segment">
                        {repairPathInsight.segment}
                      </span>
                    </>
                  ) : null}
                </div>
                <div className="issue-msg" data-testid={`diagnostics-message-${diagnostic.code || index}`}>{diagnostic.message}</div>
                {repairPathInsight ? (
                  <div
                    className="issue-hint diagnostic-path-note"
                    data-testid={`diagnostics-path-note-${diagnostic.code || index}`}
                  >
                    {repairPathInsight.note}
                  </div>
                ) : null}
                {diagnostic.target ? (
                  <div className="issue-hint">
                    {t(
                      "Click to route into the matching node, edge, or Strategy IR location and keep the canvas aligned."
                    )}
                  </div>
                ) : null}
                {diagnostic.hint ? <div className="issue-hint">{diagnostic.hint}</div> : null}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
