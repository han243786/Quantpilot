import { useState, useCallback } from "react";
import { useI18n } from "../i18n";

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
  const { t } = useI18n();
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
    <div className="qs-editor-page">
      <div className="qs-editor-header">
        <h1>{t("QuantScript 编辑器")}</h1>
        <div className="qs-editor-header__actions">
          <button
            className="ghost-btn"
            onClick={() => setSource(DEFAULT_QS)}
            disabled={running}
          >
            {t("重置示例")}
          </button>
          <button
            className="primary-btn"
            onClick={handleRun}
            disabled={running}
            data-testid="qs-editor-run"
          >
            {running ? t("运行中...") : t("运行测试")}
          </button>
        </div>
      </div>

      <textarea
        className="qs-editor-textarea"
        value={source}
        maxLength={50000}
        onChange={(e) => setSource(e.target.value)}
        onPaste={(e) => {
          if (e.clipboardData.getData("text").length > 100_000) {
            e.preventDefault();
            alert(t("粘贴内容超过 100KB，请分批粘贴。"));
          }
        }}
        data-testid="qs-editor-textarea"
        spellCheck={false}
      />

      {error && (
        <div className="qs-editor-error" data-testid="qs-editor-error">
          {error}
        </div>
      )}

      {report && (
        <div className="qs-editor-report" data-testid="qs-editor-report">
          <div className="qs-editor-report__header">
            <h3>{report.scenario_name}</h3>
            <span
              className="qs-editor-report__badge"
              style={{ background: report.failed_count > 0 ? "var(--ad-error)" : "var(--ad-success)" }}
            >
              {report.passed_count}/{report.steps.length} {t("通过")}
              {report.failed_count > 0 && ` ${report.failed_count} ${t("失败")}`}
              {report.skipped_count > 0 && ` ${report.skipped_count} ${t("跳过")}`}
            </span>
            <span style={{ fontSize: "12px", color: "var(--ad-text-muted)" }}>{report.duration_ms}ms</span>
          </div>

          {report.graph_id && (
            <div className="qs-editor-report__graph-id">
              {t("策略已保存")}: <code>{report.graph_id}</code>
            </div>
          )}

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
              {report.steps.map((step, i) => {
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

      <div className="qs-editor-footer">
        <strong>{t("支持指令")}</strong>: @test @step @compile @run @backtest @assert @save_run @modify @wait @compare_backtests @debug
        {" | "}
        <strong>{t("快捷键")}</strong>: Tab 缩进, Ctrl+Enter 运行
      </div>
    </div>
  );
}
