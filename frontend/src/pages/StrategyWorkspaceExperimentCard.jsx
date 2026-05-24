import { useMemo, useState } from "react";
import {
  getCapabilityActionBlockReason,
  isCapabilitySyncBlocked
} from "../capabilities/supportMatrix";
import { projectUiActions } from "../capabilities/capabilityProjection";
import { useGraphStore } from "../store/graphStore";
import { backtestDetailPath, navigateTo } from "../router";

function parseNumberList(input, parser = Number) {
  return input
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean)
    .map((value) => parser(value))
    .filter((value) => Number.isFinite(value));
}

function formatPercent(value) {
  return `${value >= 0 ? "+" : ""}${(value * 100).toFixed(2)}%`;
}

export default function StrategyWorkspaceExperimentCard({ strategyId, currentGraph }) {
  const experiments = useGraphStore((state) => state.runtime.experiments);
  const experimentsStatus = useGraphStore((state) => state.runtime.experimentsStatus);
  const selectedExperiment = useGraphStore((state) => state.runtime.selectedExperiment);
  const selectedExperimentId = useGraphStore((state) => state.runtime.selectedExperimentId);
  const selectedExperimentStatus = useGraphStore((state) => state.runtime.selectedExperimentStatus);
  const backendError = useGraphStore((state) => state.runtime.backendError);
  const startBacktestExperiment = useGraphStore((state) => state.startBacktestExperiment);
  const loadExperimentDetail = useGraphStore((state) => state.loadExperimentDetail);
  const capabilityStatus = useGraphStore((state) => state.capabilityStatus);
  const capabilitySource = useGraphStore((state) => state.capabilitySource);
  const capabilityMessage = useGraphStore((state) => state.capabilityMessage);
  const capabilities = useGraphStore((state) => state.capabilities);

  const [experimentName, setExperimentName] = useState("");
  const [feeGridDraft, setFeeGridDraft] = useState("5, 10, 20");
  const [slippageGridDraft, setSlippageGridDraft] = useState("5");
  const [latencyGridDraft, setLatencyGridDraft] = useState("0, 100");

  const graphExperiments = useMemo(
    () =>
      (experiments || []).filter(
        (entry) => entry.graph_id === (currentGraph?.metadata?.graph_id || strategyId)
      ),
    [currentGraph?.metadata?.graph_id, experiments, strategyId]
  );
  const activeExperiment =
    selectedExperiment?.graph_id === (currentGraph?.metadata?.graph_id || strategyId)
      ? selectedExperiment
      : null;
  const capabilitySyncBlocked = isCapabilitySyncBlocked(capabilityStatus, capabilitySource);
  const runSweepAction = projectUiActions({
    capabilities,
    capabilityStatus,
    capabilitySource,
    capabilityMessage
  }).run_parameter_sweep;
  const runSweepBlockedReason = getCapabilityActionBlockReason({
    actionKey: "run_parameter_sweep",
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    capabilities
  });

  async function handleStartExperiment() {
    if (capabilitySyncBlocked || !runSweepAction?.enabled) {
      return;
    }

    await startBacktestExperiment({
      experimentName,
      feeBps: parseNumberList(feeGridDraft, Number),
      slippageBps: parseNumberList(slippageGridDraft, Number),
      latencyMs: parseNumberList(latencyGridDraft, (value) => Number.parseInt(value, 10))
    });
  }

  return (
    <div className="open-orders-card" data-testid="workspace-experiment-card">
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">参数扫描</div>
          <div className="muted-line">
            执行窄范围执行假设扫描，不另开第二套实验协议。
          </div>
        </div>
        <strong>{graphExperiments.length}</strong>
      </div>

      <div className="workspace-version-save-form" data-testid="workspace-experiment-form">
        <label className="field-label">
          实验名称
          <input
            type="text"
            className="field-input"
            value={experimentName}
            data-testid="workspace-experiment-name-input"
            onChange={(event) => setExperimentName(event.target.value)}
            placeholder="执行假设扫描"
          />
        </label>
        <label className="field-label">
          手续费 bps 网格
          <input
            type="text"
            className="field-input"
            value={feeGridDraft}
            data-testid="workspace-experiment-fee-grid-input"
            onChange={(event) => setFeeGridDraft(event.target.value)}
            placeholder="5, 10, 20"
          />
        </label>
        <label className="field-label">
          滑点 bps 网格
          <input
            type="text"
            className="field-input"
            value={slippageGridDraft}
            data-testid="workspace-experiment-slippage-grid-input"
            onChange={(event) => setSlippageGridDraft(event.target.value)}
            placeholder="5"
          />
        </label>
        <label className="field-label">
          延迟 ms 网格
          <input
            type="text"
            className="field-input"
            value={latencyGridDraft}
            data-testid="workspace-experiment-latency-grid-input"
            onChange={(event) => setLatencyGridDraft(event.target.value)}
            placeholder="0, 100"
          />
        </label>
        <div className="strategy-inspector-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid="workspace-experiment-run-action"
            disabled={capabilitySyncBlocked || !runSweepAction?.enabled}
            title={runSweepBlockedReason || undefined}
            onClick={handleStartExperiment}
          >
            运行扫描
          </button>
        </div>
      </div>

      {runSweepBlockedReason ? (
        <div
          className="history-note history-note-warning"
          data-testid="workspace-experiment-capability-note"
        >
          {runSweepBlockedReason}
        </div>
      ) : null}

      {experimentsStatus === "loading" ? (
        <div className="muted-line">正在加载实验历史...</div>
      ) : null}
      {graphExperiments.length === 0 && experimentsStatus !== "loading" ? (
        <div className="muted-line">该策略图尚未记录参数扫描。</div>
      ) : null}
      {backendError && selectedExperimentStatus === "error" ? (
        <div className="history-note history-note-warning">{backendError}</div>
      ) : null}

      <div className="workspace-version-history-list">
        {graphExperiments.map((entry) => (
          <div
            key={entry.experiment_id}
            className="open-order-item"
            data-testid={`workspace-experiment-entry-${entry.experiment_id}`}
          >
            <div className="open-order-topline">
              <strong>{entry.experiment_name || entry.experiment_id}</strong>
              <span>{entry.variant_count} 个变体</span>
            </div>
            <div className="muted-line">
              轴：{entry.sweep_axes.length > 0 ? entry.sweep_axes.join(", ") : "单一变体"}
            </div>
            {entry.best_backtest_id ? (
              <div className="muted-line">
                最佳回测：{entry.best_backtest_id}
                {typeof entry.best_total_return_ratio === "number"
                  ? ` (${formatPercent(entry.best_total_return_ratio)})`
                  : ""}
              </div>
            ) : null}
            <div className="strategy-inspector-actions">
              <button
                type="button"
                className="ghost-btn compact-btn"
                data-testid={`workspace-experiment-open-${entry.experiment_id}`}
                onClick={() => loadExperimentDetail(entry.experiment_id)}
              >
                打开结果
              </button>
            </div>
          </div>
        ))}
      </div>

      {activeExperiment && selectedExperimentId ? (
        <div className="workspace-experiment-results" data-testid="workspace-experiment-results">
          <div className="open-orders-header">
            <div>
              <div className="mini-list-title">
                {activeExperiment.definition.experiment_name || activeExperiment.experiment_id}
              </div>
              <div className="muted-line">
                回放来源：{activeExperiment.definition.replay_source}
              </div>
            </div>
            <strong>{activeExperiment.variants.length}</strong>
          </div>
          <div className="workspace-experiment-table">
            {activeExperiment.variants.map((variant) => (
              <div
                key={variant.variant_id}
                className="workspace-experiment-row"
                data-testid={`workspace-experiment-variant-${variant.variant_id}`}
              >
                <div className="workspace-experiment-row__head">
                  <strong>{variant.variant_id}</strong>
                  <span>{variant.backtest_id}</span>
                </div>
                <div className="workspace-experiment-row__metrics">
                  <span>手续费 {variant.fee_bps} bps</span>
                  <span>滑点 {variant.slippage_bps} bps</span>
                  <span>延迟 {variant.latency_ms} ms</span>
                  <span>收益 {formatPercent(variant.summary.total_return_ratio)}</span>
                  <span>回撤 {formatPercent(variant.summary.drawdown_analysis?.max_drawdown_ratio ?? variant.summary.max_drawdown_ratio)}</span>
                  <span>交易 {variant.summary.trade_count}</span>
                </div>
                <div className="strategy-inspector-actions">
                  <button
                    type="button"
                    className="ghost-btn compact-btn"
                    data-testid={`workspace-experiment-detail-${variant.variant_id}`}
                    onClick={() => navigateTo(backtestDetailPath(variant.backtest_id, strategyId))}
                  >
                    打开回测详情
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
