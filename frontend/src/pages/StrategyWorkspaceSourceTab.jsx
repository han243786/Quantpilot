import { useEffect, useState } from "react";
import { useI18n } from "../i18n";
import { useGraphStore } from "../store/graphStore";
import { API_BASE } from "../store/graphStorePersistenceHelpers";

export default function StrategyWorkspaceSourceTab({ graphId, onRunScenario }) {
  const { t } = useI18n();
  const [source, setSource] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [runResult, setRunResult] = useState(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    if (!graphId) return;
    setLoading(true);
    fetch(`${API_BASE}/graphs/${encodeURIComponent(graphId)}/quantscript`)
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.text();
      })
      .then((text) => {
        setSource(text);
        setError(null);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [graphId]);

  const handleRunScenario = async () => {
    if (!source) return;
    setRunning(true);
    setRunResult(null);
    try {
      const resp = await fetch("/api/test/scenario/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ source }),
      });
      if (!resp.ok) {
        const text = await resp.text();
        setRunResult({ error: `HTTP ${resp.status}: ${text.slice(0, 300)}` });
      } else {
        setRunResult(await resp.json());
      }
    } catch (e) {
      setRunResult({ error: e.message });
    } finally {
      setRunning(false);
    }
  };

  if (loading) return <div className="muted-line">{t("加载源码中...")}</div>;
  if (error) return <div className="muted-line">{t("无法加载源码")}: {error}</div>;

  return (
    <div className="qs-source-tab">
      <div className="qs-source-tab__header">
        <h3>{t("QuantScript 源码")}</h3>
        <button
          className="primary-btn"
          onClick={handleRunScenario}
          disabled={running || !source}
          data-testid="source-tab-run-scenario"
        >
          {running ? t("运行中...") : t("运行测试")}
        </button>
      </div>

      <pre className="qs-source-code" data-testid="source-tab-code">
        {source || t("无源码")}
      </pre>

      {runResult && (
        <div className="qs-editor-report">
          {runResult.error ? (
            <div className="qs-editor-error">{runResult.error}</div>
          ) : (
            <div>
              <div className="qs-editor-report__header">
                <h4 style={{ margin: 0 }}>{runResult.scenario_name}</h4>
                <span
                  className="qs-editor-report__badge"
                  style={{ background: runResult.failed_count > 0 ? "var(--ad-error)" : "var(--ad-success)" }}
                >
                  {runResult.passed_count}/{runResult.steps?.length || 0} {t("通过")}
                  {runResult.failed_count > 0 && ` ${runResult.failed_count} ${t("失败")}`}
                </span>
                <span style={{ fontSize: "12px", color: "var(--ad-text-muted)" }}>{runResult.duration_ms}ms</span>
              </div>
              <table className="qs-editor-report-table">
                <thead>
                  <tr>
                    <th></th>
                    <th>{t("步骤")}</th>
                    <th>{t("耗时")}</th>
                    <th>{t("详情")}</th>
                  </tr>
                </thead>
                <tbody>
                  {runResult.steps?.map((step, i) => {
                    const icon = step.status === "passed" ? "✓" : step.status === "failed" ? "✗" : "⊘";
                    const color = step.status === "passed" ? "var(--ad-success)" : step.status === "failed" ? "var(--ad-error)" : "var(--ad-text-muted)";
                    return (
                      <tr key={i}>
                        <td className="qs-step-icon" style={{ color }}>{icon}</td>
                        <td>{step.name}</td>
                        <td className="qs-step-duration">{step.duration_ms}ms</td>
                        <td className="qs-step-message">
                          {step.message?.slice(0, 200)}
                          {step.status === "failed" && step.message?.includes("actual:") && (
                            <span className="qs-step-actual">
                              {" "}[actual: {step.message.match(/actual:\s*([^)]+)/)?.[1] || "?"}]
                            </span>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
