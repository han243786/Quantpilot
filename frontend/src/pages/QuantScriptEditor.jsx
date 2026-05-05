import { useState, useCallback } from "react";

const DEFAULT_QS = `fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景一：BTC双均线"
    cover: ["P-03"]
}

@step("编译") {
    @compile
    @assert compile.compilable == true
}

@step("回测") {
    @backtest { source: "deterministic_mock" }
    @assert backtest.metrics.step_count >= 100
}
`;

export default function QuantScriptEditor() {
  const [source, setSource] = useState(DEFAULT_QS);
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState(null);
  const [error, setError] = useState(null);

  const handleRun = useCallback(async () => {
    setRunning(true);
    setError(null);
    setReport(null);
    try {
      const resp = await fetch("/api/test/scenario/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ source }),
      });
      if (!resp.ok) {
        const text = await resp.text();
        setError(`HTTP ${resp.status}: ${text.slice(0, 200)}`);
      } else {
        setReport(await resp.json());
      }
    } catch (e) {
      setError(e.message);
    } finally {
      setRunning(false);
    }
  }, [source]);

  return (
    <div className="qs-editor-page" style={{ padding: "20px", maxWidth: "1200px", margin: "0 auto" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
        <h1 style={{ margin: 0 }}>QuantScript 编辑器</h1>
        <div style={{ display: "flex", gap: "8px" }}>
          <button
            className="ghost-btn"
            onClick={() => setSource(DEFAULT_QS)}
            disabled={running}
          >
            重置示例
          </button>
          <button
            className="primary-btn"
            onClick={handleRun}
            disabled={running}
            data-testid="qs-editor-run"
          >
            {running ? "运行中..." : "▶ 运行测试"}
          </button>
        </div>
      </div>

      <textarea
        value={source}
        onChange={(e) => setSource(e.target.value)}
        data-testid="qs-editor-textarea"
        spellCheck={false}
        style={{
          width: "100%",
          height: "400px",
          fontFamily: "Consolas, Monaco, monospace",
          fontSize: "13px",
          lineHeight: "1.4",
          padding: "12px",
          border: "1px solid #444",
          borderRadius: "4px",
          background: "#1a1a2e",
          color: "#e0e0e0",
          resize: "vertical",
        }}
      />

      {error && (
        <div
          data-testid="qs-editor-error"
          style={{
            marginTop: "16px",
            padding: "12px",
            background: "#3a1a1a",
            border: "1px solid #a33",
            borderRadius: "4px",
            color: "#f88",
            fontFamily: "monospace",
          }}
        >
          {error}
        </div>
      )}

      {report && (
        <div data-testid="qs-editor-report" style={{ marginTop: "16px" }}>
          <div style={{ marginBottom: "12px", display: "flex", alignItems: "center", gap: "12px" }}>
            <h3 style={{ margin: 0 }}>{report.scenario_name}</h3>
            <span style={{
              padding: "2px 8px", borderRadius: "12px", fontSize: "12px",
              background: report.failed_count > 0 ? "#a33" : "#3a3",
              color: "white",
            }}>
              {report.passed_count}/{report.steps.length} 通过
              {report.failed_count > 0 && ` ${report.failed_count} 失败`}
              {report.skipped_count > 0 && ` ${report.skipped_count} 跳过`}
            </span>
            <span style={{ fontSize: "12px", color: "#888" }}>{report.duration_ms}ms</span>
          </div>

          {report.graph_id && (
            <div style={{ marginBottom: "12px", fontSize: "12px", color: "#888" }}>
              策略已保存: <code>{report.graph_id}</code>
            </div>
          )}

          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
            <thead>
              <tr style={{ background: "#222" }}>
                <th style={{ padding: "8px", textAlign: "left", width: "30px" }}></th>
                <th style={{ padding: "8px", textAlign: "left" }}>步骤</th>
                <th style={{ padding: "8px", textAlign: "right", width: "70px" }}>耗时</th>
                <th style={{ padding: "8px", textAlign: "left" }}>详情</th>
              </tr>
            </thead>
            <tbody>
              {report.steps.map((step, i) => {
                const icon = step.status === "passed" ? "✓" : step.status === "failed" ? "✗" : "⊘";
                const color = step.status === "passed" ? "#3a3" : step.status === "failed" ? "#a33" : "#888";
                return (
                  <tr key={i} style={{ borderBottom: "1px solid #333" }}>
                    <td style={{ padding: "8px", color, fontWeight: "bold" }}>{icon}</td>
                    <td style={{ padding: "8px" }}>{step.name}</td>
                    <td style={{ padding: "8px", textAlign: "right", color: "#888" }}>{step.duration_ms}ms</td>
                    <td style={{ padding: "8px", fontSize: "12px", color: "#aaa", maxWidth: "500px", overflow: "hidden", textOverflow: "ellipsis" }}>
                      {step.message?.slice(0, 200)}
                      {step.status === "failed" && step.message?.includes("actual:") && (
                        <span style={{ color: "#f44", fontWeight: "bold" }}>
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

      <div style={{ marginTop: "24px", fontSize: "11px", color: "#555", borderTop: "1px solid #333", paddingTop: "12px" }}>
        <strong>支持指令</strong>: @test @step @compile @run @backtest @assert @save_run @modify @wait @compare_backtests @debug
        {" | "}
        <strong>快捷键</strong>: Tab 缩进, Ctrl+Enter 运行
      </div>
    </div>
  );
}
