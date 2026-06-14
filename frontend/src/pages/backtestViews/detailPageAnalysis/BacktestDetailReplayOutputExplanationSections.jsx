import {
  AnalysisSection,
  MetricPair,
  formatTime,
  formatValue
} from "../shared";

function ExplanationDetailCard({ title, summary, entries, testId, emptyText }) {
  return (
    <div className="open-orders-card" data-testid={testId}>
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">{title}</div>
          <div className="muted-line">{summary}</div>
        </div>
        <strong>{entries.length}</strong>
      </div>
      {entries.length === 0 ? <div className="muted-line">{emptyText}</div> : null}
      {entries.map((entry) => (
        <div
          key={entry.nodeId}
          className="open-order-item"
          data-testid={`${testId}-entry-${entry.nodeId}`}
        >
          <div className="open-order-topline">
            <strong>{entry.nodeName}</strong>
            <span>{entry.nodeId}</span>
          </div>
          {entry.explanationSummary ? <div className="muted-line">{entry.explanationSummary}</div> : null}
          <div className="open-order-grid">
            {entry.rows.map((row) => (
              <div key={`${entry.nodeId}_${row.key}`}>
                <span>{row.label}</span>
                <strong>{row.value}</strong>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export function BacktestDetailReplayOutputExplanationSections({
  t = (value) => value,
  curvePreview = [],
  tradePreview = [],
  equityCurveArtifactId = "-",
  tradeLedgerArtifactId = "-",
  outputArtifacts = [],
  riskExplanationEntries = [],
  orderExplanationEntries = []
}) {
  return (
    <>
      <AnalysisSection
        testId="backtest-detail-replay-preview"
        kicker={t("回放预览")}
        title={t("权益曲线与成交样本")}
        summary={t("详情页只保留高信号的回放切片，让它保持为策略分析视图，而不是原始日志堆叠。")}
      >
        <div className="analysis-card-grid analysis-card-grid--two">
          <div className="open-orders-card" data-testid="backtest-detail-equity-card">
            <div className="open-orders-header">
              <div>
                <div className="mini-list-title">{t("权益曲线工件")}</div>
                <div className="muted-line">
                  {t("预览曲线首尾片段，以便快速确认策略层面的权益表现。")}
                </div>
              </div>
              <strong>{equityCurveArtifactId || "-"}</strong>
            </div>
            {curvePreview.length === 0 ? (
              <div className="muted-line">{t("这次回测没有可用的权益曲线样本。")}</div>
            ) : null}
            {curvePreview.map((point, index) => (
              <div key={`${point.ts_ms}_${index}`} className="open-order-item">
                <div className="open-order-topline">
                  <strong>{formatTime(point.ts_ms)}</strong>
                </div>
                <div className="open-order-grid">
                  <div>
                    <span>{t("权益")}</span>
                    <strong>{formatValue(point.equity)}</strong>
                  </div>
                  <div>
                    <span>{t("现金")}</span>
                    <strong>{formatValue(point.cash_balance)}</strong>
                  </div>
                  <div>
                    <span>{t("净名义价值")}</span>
                    <strong>{formatValue(point.net_notional)}</strong>
                  </div>
                </div>
              </div>
            ))}
          </div>

          <div className="open-orders-card" data-testid="backtest-detail-trade-card">
            <div className="open-orders-header">
              <div>
                <div className="mini-list-title">{t("成交账本工件")}</div>
                <div className="muted-line">
                  {t("抽样展示已执行成交，便于审计与回放交叉核验。")}
                </div>
              </div>
              <strong>{tradeLedgerArtifactId || "-"}</strong>
            </div>
            {tradePreview.length === 0 ? (
              <div className="muted-line">{t("这次回测没有记录成交。")}</div>
            ) : null}
            {tradePreview.map((trade) => (
              <div key={trade.fill_id} className="open-order-item">
                <div className="open-order-topline">
                  <strong>{trade.fill_id}</strong>
                  <span>{trade.cycle_name}</span>
                </div>
                <div className="open-order-grid">
                  <div>
                    <span>{t("方向")}</span>
                    <strong>{trade.side}</strong>
                  </div>
                  <div>
                    <span>{t("数量")}</span>
                    <strong>{formatValue(trade.filled_qty)}</strong>
                  </div>
                  <div>
                    <span>{t("价格")}</span>
                    <strong>{formatValue(trade.filled_price)}</strong>
                  </div>
                  <div>
                    <span>{t("手续费")}</span>
                    <strong>{formatValue(trade.fee_paid)}</strong>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </AnalysisSection>

      <AnalysisSection
        testId="backtest-detail-output-artifacts"
        kicker={t("输出引用")}
        title={t("持久化输出文件")}
        summary={t("保留文件级可追溯性，但不把页面变成纯存储列表。")}
      >
        <div className="open-orders-card" data-testid="backtest-detail-output-card">
          <div className="open-orders-header">
            <div>
              <div className="mini-list-title">{t("输出文件")}</div>
              <div className="muted-line">{t("记录在当前策略实验 manifest 下的文件列表。")}</div>
            </div>
            <strong>{outputArtifacts.length}</strong>
          </div>
          {outputArtifacts.length === 0 ? (
            <div className="muted-line">{t("这次回测没有记录任何输出文件引用。")}</div>
          ) : null}
          {outputArtifacts.map((artifact) => (
            <MetricPair key={artifact.artifact_id} label={artifact.kind} value={artifact.file_name} />
          ))}
        </div>
      </AnalysisSection>

      <AnalysisSection
        testId="backtest-detail-explanations"
        kicker={t("执行解释")}
        title={t("风控与订单解释")}
        summary={t("复用同一套 runtime_diagnostics explanation rows，在详情页直接展示风控裁剪和订单执行语义。")}
      >
        <div className="analysis-card-grid analysis-card-grid--two">
          <ExplanationDetailCard
            title={t("风控详情")}
            summary={t("选取 detail payload 中已结构化的 risk_detail_rows，不再重新拼第二套解释协议。")}
            entries={riskExplanationEntries}
            testId="backtest-detail-risk-card"
            emptyText={t("当前回测详情还没有可展示的风控解释。")}
          />
          <ExplanationDetailCard
            title={t("订单详情")}
            summary={t("沿用同一 explanation rows 展示下单来源、生命周期和订单语义。")}
            entries={orderExplanationEntries}
            testId="backtest-detail-order-card"
            emptyText={t("当前回测详情还没有可展示的订单解释。")}
          />
        </div>
      </AnalysisSection>
    </>
  );
}
