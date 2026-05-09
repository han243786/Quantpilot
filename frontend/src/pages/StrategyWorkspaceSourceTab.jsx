import { useEffect, useState } from "react";
import { useGraphStore } from "../store/graphStore";
import { API_BASE } from "../store/graphStorePersistenceHelpers";

export default function StrategyWorkspaceSourceTab({ graphId, onRunScenario }) {
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

  if (loading) return <div className="muted-line">加载源码中...</div>;
  if (error) return <div className="muted-line">无法加载源码: {error}</div>;

  return (
    <div className="qs-source-tab" style={{ padding: "16px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
        <h3 style={{ margin: 0 }}>QuantScript 源码</h3>
        <button
          className="primary-btn"
          onClick={handleRunScenario}
          disabled={running || !source}
          data-testid="source-tab-run-scenario"
        >
          {running ? "运行中..." : "运行测试"}
        </button>
      </div>

      <pre
        style={{
          background: "var(--ad-panel)",
          color: "var(--ad-text)",
          padding: "16px",
          borderRadius: "4px",
          fontFamily: "Consolas, Monaco, monospace",
          fontSize: "13px",
          lineHeight: "1.5",
          overflow: "auto",
          maxHeight: "60vh",
          border: "1px solid var(--ad-border)",
        }}
        data-testid="source-tab-code"
      >
        {source || "无源码"}
      </pre>

      {runResult && (
        <div style={{ marginTop: "16px" }}>
          {runResult.error ? (
            <div style={{ padding: "12px", background: "var(--ad-error-soft)", border: "1px solid var(--ad-error)", borderRadius: "4px", color: "var(--ad-error)" }}>
              {runResult.error}
            </div>
          ) : (
            <div>
              <div style={{ marginBottom: "12px", display: "flex", alignItems: "center", gap: "12px" }}>
                <h4 style={{ margin: 0 }}>{runResult.scenario_name}</h4>
                <span style={{
                  padding: "2px 8px", borderRadius: "12px", fontSize: "12px",
                  background: runResult.failed_count > 0 ? "var(--ad-error)" : "var(--ad-success)",
                  color: "white",
                }}>
                  {runResult.passed_count}/{runResult.steps?.length || 0} 通过
                  {runResult.failed_count > 0 && ` ${runResult.failed_count} 失败`}
                </span>
                <span style={{ fontSize: "12px", color: "var(--ad-text-muted)" }}>{runResult.duration_ms}ms</span>
              </div>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
                <thead>
                  <tr style={{ background: "var(--ad-card)" }}>
                    <th style={{ padding: "8px", textAlign: "left", width: "30px" }}></th>
                    <th style={{ padding: "8px", textAlign: "left" }}>步骤</th>
                    <th style={{ padding: "8px", textAlign: "right", width: "70px" }}>耗时</th>
                    <th style={{ padding: "8px", textAlign: "left" }}>详情</th>
                  </tr>
                </thead>
                <tbody>
                  {runResult.steps?.map((step, i) => {
                    const icon = step.status === "passed" ? "✓" : step.status === "failed" ? "✗" : "⊘";
                    const color = step.status === "passed" ? "var(--ad-success)" : step.status === "failed" ? "var(--ad-error)" : "var(--ad-text-muted)";
                    return (
                      <tr key={i} style={{ borderBottom: "1px solid var(--ad-border)" }}>
                        <td style={{ padding: "8px", color, fontWeight: "bold" }}>{icon}</td>
                        <td style={{ padding: "8px" }}>{step.name}</td>
                        <td style={{ padding: "8px", textAlign: "right", color: "var(--ad-text-muted)" }}>{step.duration_ms}ms</td>
                        <td style={{ padding: "8px", fontSize: "12px", color: "var(--ad-text-secondary)", maxWidth: "500px", overflow: "hidden", textOverflow: "ellipsis" }}>
                          {step.message?.slice(0, 200)}
                          {step.status === "failed" && step.message?.includes("actual:") && (
                            <span style={{ color: "var(--ad-error)", fontWeight: "bold" }}>
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
