import { useState, useCallback, useEffect, useMemo } from "react";
import { useI18n } from "../i18n";
import { fetchWithTimeout } from "../utils/api";
import { humanizeErrorText } from "../utils/errorText";

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

const DRAFT_STORAGE_KEY = "quantpilot.quantscript.draft";

function loadDraft() {
  try {
    return window.localStorage?.getItem(DRAFT_STORAGE_KEY) || DEFAULT_QS;
  } catch (_) {
    return DEFAULT_QS;
  }
}

function toast(type, message) {
  if (typeof window === "undefined") return;
  window.dispatchEvent(new CustomEvent("qp-toast", { detail: { type, message } }));
}

export default function QuantScriptEditor() {
  const { t } = useI18n();
  const [source, setSource] = useState(loadDraft);
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState(null);
  const [error, setError] = useState(null);
  const lineNumbers = useMemo(
    () => Array.from({ length: Math.max(1, source.split("\n").length) }, (_, index) => index + 1),
    [source],
  );

  useEffect(() => {
    try {
      if (window.localStorage?.getItem("quantpilot.quantscript.autosave") === "0") return;
      window.localStorage?.setItem(DRAFT_STORAGE_KEY, source);
    } catch (_) {
      toast("error", t("本地存储空间不足，QuantScript 草稿未保存。"));
    }
  }, [source, t]);

  const handleRun = useCallback(async () => {
    setRunning(true);
    setError(null);
    setReport(null);
    try {
      const resp = await fetchWithTimeout("/api/test/scenario/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ source }),
      });
      if (!resp.ok) {
        const text = await resp.text();
        setError(humanizeErrorText(text) || t("QuantScript 测试运行失败"));
      } else {
        setReport(await resp.json());
        toast("success", t("QuantScript 测试完成"));
      }
    } catch (e) {
      setError(humanizeErrorText(e.message));
    } finally {
      setRunning(false);
    }
  }, [source, t]);

  const handleReset = useCallback(() => {
    setSource(DEFAULT_QS);
    setReport(null);
    setError(null);
  }, []);

  const handleKeyDown = useCallback(
    (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        void handleRun();
        return;
      }
      if (e.key !== "Tab") return;
      e.preventDefault();
      const target = e.currentTarget;
      const start = target.selectionStart;
      const end = target.selectionEnd;
      const indent = "    ";
      setSource((prev) => `${prev.slice(0, start)}${indent}${prev.slice(end)}`);
      requestAnimationFrame(() => {
        target.selectionStart = start + indent.length;
        target.selectionEnd = start + indent.length;
      });
    },
    [handleRun],
  );

  return (
    <div className="qs-editor-page">
      <div className="qs-editor-header">
        <h1>{t("QuantScript 编辑器")}</h1>
        <div className="qs-editor-header__actions">
          <button
            className="ad-btn ad-btn--ghost"
            onClick={handleReset}
            disabled={running}
          >
            {t("重置示例")}
          </button>
          <button
            className="ad-btn ad-btn--primary"
            onClick={handleRun}
            disabled={running}
            data-testid="qs-editor-run"
          >
            {running ? t("运行中...") : t("运行测试")}
          </button>
        </div>
      </div>

      <div className="qs-editor-shell">
        <pre className="qs-editor-line-numbers" aria-hidden="true">
          {lineNumbers.map((lineNumber) => (
            <span key={lineNumber}>{lineNumber}</span>
          ))}
        </pre>
        <textarea
          className="qs-editor-textarea"
          value={source}
          maxLength={50000}
          onChange={(e) => setSource(e.target.value)}
          onKeyDown={handleKeyDown}
          onPaste={(e) => {
            if (e.clipboardData.getData("text").length > 100_000) {
              e.preventDefault();
              toast("error", t("粘贴内容超过 100KB，请分批粘贴。"));
            }
          }}
          data-testid="qs-editor-textarea"
          spellCheck={false}
        />
      </div>

      {error && (
        <div className="qs-editor-error" data-testid="qs-editor-error">
          {error}
        </div>
      )}

      {report && (
        <div className="qs-editor-report" data-testid="qs-editor-report">
          <div className="qs-editor-report__header">
            <h3>{report.scenario_name}</h3>
            <span className={`qs-editor-report__badge${report.failed_count > 0 ? " qs-editor-report__badge--failed" : " qs-editor-report__badge--passed"}`}>
              {report.passed_count}/{report.steps.length} {t("通过")}
              {report.failed_count > 0 && ` ${report.failed_count} ${t("失败")}`}
              {report.skipped_count > 0 && ` ${report.skipped_count} ${t("跳过")}`}
            </span>
            <span className="qs-editor-report__duration">{report.duration_ms}ms</span>
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
                const tone = ["passed", "failed"].includes(step.status) ? step.status : "skipped";
                return (
                  <tr key={i}>
                    <td className={`qs-step-icon qs-step-icon--${tone}`}>{icon}</td>
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
