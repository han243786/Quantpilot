import { useGraphStore } from "../store/graphStore";
import { useI18n } from "../i18n";
import { buildRepairPathInsight } from "../utils/repairPathInsights";

function severityLabel(severity) {
  if (severity === "error") return "阻塞";
  if (severity === "warning") return "警告";
  if (severity === "info") return "提示";
  return "阻塞";
}

function severityTone(severity) {
  if (severity === "warning") return "warning";
  if (severity === "info") return "info";
  return "danger";
}

function sourceLabel(source, t) {
  if (source === "strategy_ir") return t("策略预检");
  if (source === "formal_quantscript") return t("策略脚本");
  if (source === "runtime") return t("运行");
  return t("策略图");
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
        <div className="property-card-title" data-testid="diagnostics-panel-title">{t("编译诊断")}</div>
        <div className="property-card-caption">
          {t(
            "将级别、来源、目标和当前修复路径放在同一条可读诊断流中。"
          )}
        </div>
      </div>

      {diagnostics.length === 0 ? (
        <div className="empty-state diagnostics-empty" data-testid="diagnostics-empty">{t("暂无编译诊断。请先点击顶部\"编译\"按钮生成策略诊断。")}</div>
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
                    <span className="diagnostic-chip">{t("可定位目标")}</span>
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
                      "点击后会定位到匹配的节点、连线或策略中间表示位置，并同步画布焦点。"
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
