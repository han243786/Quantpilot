import V4RuntimeEvidencePanel from "../../../components/V4RuntimeEvidencePanel";
import {
  AnalysisSection,
  DrawdownChart,
  MonthlyReturnsHeatmap,
  MetricPair,
  formatRatio,
  formatTime,
  formatValue
} from "../shared";

export function BacktestDetailCoreArtifactSections({
  t = (value) => value,
  selectedSummary = null,
  manifest = null,
  metrics = null,
  summary = null,
  startedAt = null,
  endedAt = null,
  governanceRows = [],
  equityCurve = [],
  periodReturns = [],
  metricsArtifactId = "-",
  eventsLength = 0
}) {
  return (
    <>
      <AnalysisSection
        testId="backtest-detail-core-artifacts"
        kicker={t("工件概览")}
        title={t("核心工件")}
        summary={t("先以持久化的 manifest 和 metrics 工件作为主要审查入口，再按需展开回放预览或完整事件流。")}
      >
        <div className="analysis-card-grid analysis-card-grid--two">
          <div className="open-orders-card" data-testid="backtest-detail-manifest-card">
            <div className="open-orders-header">
              <div>
                <div className="mini-list-title">{t("清单工件")}</div>
                <div className="muted-line">
                  {t("与策略关联的 manifest 上下文、编译链路与回放来源。")}
                </div>
              </div>
              <strong>{manifest?.manifest_id || "-"}</strong>
            </div>
            <MetricPair label={t("创建时间")} value={formatTime(manifest?.created_at_ms)} />
            <MetricPair
              label={t("编译 ID")}
              value={manifest?.compile_id || selectedSummary?.compile_id || "-"}
            />
            <MetricPair
              label={t("运行规格")}
              value={manifest?.backtest_spec?.run_spec?.schema_version || "-"}
            />
            <MetricPair
              label={t("回放来源")}
              value={manifest?.backtest_spec?.replay_source || "-"}
            />
            <MetricPair
              label={t("策略工件")}
              value={manifest?.compile_artifacts?.strategy?.artifact_id || "-"}
              testId="backtest-detail-manifest-strategy-artifact"
            />
            <MetricPair
              label={t("编译工件")}
              value={manifest?.compile_artifacts?.compile?.artifact_id || "-"}
              testId="backtest-detail-manifest-compile-artifact"
            />
            <MetricPair
              label={t("核心中间表示工件")}
              value={manifest?.compile_artifacts?.core_ir?.artifact_id || "-"}
              testId="backtest-detail-manifest-core-ir-artifact"
            />
            <div className="mini-list" data-testid="backtest-detail-governance-card">
              <div className="mini-list-title">{t("治理身份")}</div>
              {governanceRows.map((row) => (
                <MetricPair
                  key={row.key}
                  label={row.label}
                  value={row.value}
                  fullValue={row.fullValue}
                  testId={`backtest-detail-governance-${row.key}`}
                />
              ))}
            </div>
          </div>

          <div className="open-orders-card" data-testid="backtest-detail-metrics-card">
            <div className="open-orders-header">
              <div>
                <div className="mini-list-title">{t("指标工件")}</div>
                <div className="muted-line">
                  {t("当前策略实验的持久化结果摘要。")}
                </div>
              </div>
              <strong>{metricsArtifactId || "-"}</strong>
            </div>
            <MetricPair label={t("开始时间")} value={formatTime(startedAt)} />
            <MetricPair label={t("结束时间")} value={formatTime(endedAt)} />
            <MetricPair
              label={t("步数")}
              value={formatValue(summary?.step_count)}
              testId="backtest-detail-metrics-step-count"
            />
            <MetricPair label={t("会话数")} value={formatValue(metrics?.session_count)} />
            <MetricPair
              label={t("事件数")}
              value={formatValue(metrics?.event_count || eventsLength)}
              testId="backtest-detail-metrics-event-count"
            />
          </div>
        </div>
      </AnalysisSection>

      <AnalysisSection
        testId="backtest-detail-drawdown-chart"
        kicker={t("回撤分析")}
        title={t("回撤曲线")}
        summary={t("峰值到谷底的回撤深度可视化，展示策略风险暴露的持续时间。")}
      >
        <DrawdownChart equityCurve={equityCurve} title={t("回撤深度")} />
      </AnalysisSection>

      <AnalysisSection
        testId="backtest-detail-monthly-returns"
        kicker={t("收益率分解")}
        title={t("月度收益率")}
        summary={t("每月收益率热力图，颜色深浅表示收益大小，用于评估策略在不同月份的一致性。")}
      >
        <MonthlyReturnsHeatmap periodReturns={periodReturns} title={t("月度收益")} />
      </AnalysisSection>
    </>
  );
}

export function BacktestDetailV4ArtifactSection({
  v4Artifact = null,
  v4MicroMetrics = null
}) {
  if (!v4Artifact) return null;

  return (
    <AnalysisSection
      testId="backtest-detail-v4-evidence"
      kicker="v4"
      title="v4 Machine Evidence"
      summary="State-machine trajectory, Risk Plane decisions, and execution capability sources captured by the v4 backtest artifact."
    >
      <V4RuntimeEvidencePanel
        source={v4Artifact}
        testId="backtest-detail-v4-evidence-panel"
      />
      <div className="open-orders-card" data-testid="backtest-detail-v4-artifact-card">
        <div className="open-orders-header">
          <div>
            <div className="mini-list-title">v4 Backtest Artifact</div>
            <div className="muted-line">{v4Artifact.schema_version}</div>
          </div>
          <strong>{v4Artifact.symbols?.join(", ") || "-"}</strong>
        </div>
        <MetricPair label="Replay" value={v4Artifact.replay_mode || "-"} />
        <MetricPair label="Bars" value={formatValue(v4Artifact.input_bar_count)} />
        <MetricPair label="Ticks" value={formatValue(v4Artifact.input_tick_count || 0)} />
        <MetricPair
          label="Trajectory"
          value={formatValue(v4Artifact.machine_trajectory?.length || 0)}
        />
        <MetricPair
          label="Risk decisions"
          value={formatValue(v4Artifact.risk_plane_decisions?.length || 0)}
        />
        <MetricPair
          label="Capability sources"
          value={formatValue(v4Artifact.execution_capability_sources?.length || 0)}
        />
      </div>
      {v4MicroMetrics ? (
        <div className="open-orders-card" data-testid="backtest-detail-v4-microstructure-card">
          <div className="open-orders-header">
            <div>
              <div className="mini-list-title">Microstructure Metrics</div>
              <div className="muted-line">v4.5.0 tick replay execution evidence</div>
            </div>
            <strong>{formatValue(v4MicroMetrics.submitted_order_count)}</strong>
          </div>
          <MetricPair label="Fill rate" value={formatRatio(v4MicroMetrics.fill_rate)} />
          <MetricPair
            label="Avg slippage bps"
            value={formatValue(v4MicroMetrics.average_slippage_bps)}
          />
          <MetricPair
            label="Queue estimate"
            value={formatRatio(v4MicroMetrics.queue_position_estimate)}
          />
          <MetricPair
            label="VWAP deviation bps"
            value={formatValue(v4MicroMetrics.vwap_deviation_bps)}
          />
        </div>
      ) : null}
    </AnalysisSection>
  );
}

export default BacktestDetailCoreArtifactSections;
