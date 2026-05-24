import { useEffect, useMemo, useState } from "react";
import StrategyBacktestsPanel from "../components/StrategyBacktestsPanel";
import {
  backtestComparePath,
  backtestDetailPath,
  navigateTo,
  strategiesPath,
  strategyWorkspacePath
} from "../router";
import {
  AnalysisHero,
  AnalysisSection,
  AnalysisStatusBanner
} from "./BacktestAnalysisLayout";
import {
  formatPercent,
  formatTime,
  formatValue,
  MetricPair
} from "./backtestAnalysisShared";
import { useStrategyWorkspaceSharedModel } from "../hooks/useStrategyWorkspaceSharedModel";
import { useStrategyResearchUiState } from "../hooks/useStrategyResearchUiState";
import { useStrategyResearchSelectors } from "../hooks/strategyResearchSelectors";
import { useStrategyResearchActions } from "../hooks/useStrategyResearchActions";

export default function StrategyBacktestsPage({ strategyId }) {
  const { graph, runtime, loadGraphById } = useStrategyWorkspaceSharedModel();
  const [panelNotice, setPanelNotice] = useState(null);
  const uiState = useStrategyResearchUiState(strategyId);
  const selectors = useStrategyResearchSelectors(uiState);
  const actions = useStrategyResearchActions(uiState, {
    onNotice(type, message) {
      setPanelNotice({
        id: `${type}-${Date.now()}`,
        type,
        message
      });
    }
  });

  useEffect(() => {
    if (!strategyId || graph.metadata?.graph_id === strategyId) return;
    void loadGraphById(strategyId);
  }, [graph.metadata?.graph_id, loadGraphById, strategyId]);

  useEffect(() => {
    if (!panelNotice || panelNotice.type === "error") return undefined;
    const timeoutId = window.setTimeout(() => {
      setPanelNotice((current) => (current?.id === panelNotice.id ? null : current));
    }, 3200);
    return () => window.clearTimeout(timeoutId);
  }, [panelNotice]);

  const strategyName =
    graph.metadata?.graph_id === strategyId ? graph.metadata?.name || strategyId : strategyId;
  const selectedBacktest = selectors.filteredBacktests[0] || null;
  const summaryItems = [
    { label: "回测数", value: formatValue(selectors.filteredBacktests.length) },
    { label: "对比队列", value: formatValue(selectors.compareSelection.length) },
    {
      label: "最近收益",
      value: formatPercent(selectedBacktest?.summary?.total_return_ratio)
    },
    {
      label: "最近回测",
      value: selectedBacktest ? formatTime(selectedBacktest.created_at_ms) : "-"
    }
  ];

  const compareButtonDisabled = selectors.compareSelection.length !== 2;
  const isGraphLoading = graph.metadata?.graph_id !== strategyId;

  const datasetText = useMemo(() => {
    if (graph.metadata?.graph_id !== strategyId) return "-";
    const labels = selectors.filteredBacktests[0]?.filters?.dataset_labels;
    return labels?.join(", ") || "-";
  }, [graph.metadata?.graph_id, selectors.filteredBacktests, strategyId]);

  return (
    <div className="detail-page strategy-backtests-page">
      <AnalysisHero
        testId="strategy-backtests-hero"
        routeItems={[
          { label: "策略", onClick: () => navigateTo(strategiesPath()) },
          { label: strategyName },
          { label: "回测", current: true }
        ]}
        kicker="策略研究"
        title={`${strategyName} 回测`}
        subtitle="在不重新打开完整工作区的前提下，直接通过策略范围内的回测索引查看持久化实验，并把对比选择、筛选条件与详情入口留在同一路径。"
        meta={`策略：${strategyId} | 当前策略图：${graph.metadata?.graph_id || "-"}`}
        actions={
          <>
            <button
              className="ghost-btn"
              data-testid="strategy-backtests-list-button"
              onClick={() => navigateTo(strategiesPath())}
            >
              返回策略列表
            </button>
            <button
              className="ghost-btn"
              data-testid="strategy-backtests-workspace-button"
              onClick={() => navigateTo(strategyWorkspacePath(strategyId))}
            >
              打开工作区
            </button>
            <button
              className="primary-btn"
              data-testid="strategy-backtests-compare-button"
              disabled={compareButtonDisabled}
              onClick={() => navigateTo(backtestComparePath(selectors.compareSelection, strategyId))}
            >
              打开对比
            </button>
          </>
        }
        summaryItems={summaryItems}
      />

      {isGraphLoading ? (
        <AnalysisStatusBanner>正在加载策略图上下文...</AnalysisStatusBanner>
      ) : null}
      {panelNotice ? (
        <AnalysisStatusBanner variant={panelNotice.type === "error" ? "error" : "info"}>
          {panelNotice.message}
        </AnalysisStatusBanner>
      ) : null}

      <div className="analysis-page-grid">
        <div className="analysis-main-column">
          <AnalysisSection
            kicker="策略回测"
            title="回测索引"
            summary="查看持久化实验，按编译或数据集筛选，并在不重新打开完整研究控制台的前提下直接进入详情或对比。"
          >
            <StrategyBacktestsPanel
              graph={selectors.graph}
              runtime={selectors.runtime}
              selectedBacktestSummary={selectors.selectedBacktestSummary}
              backtestSummary={selectors.backtestSummary}
              backtestStartedAt={selectors.backtestStartedAt}
              backtestEndedAt={selectors.backtestEndedAt}
              backtestFilters={uiState.backtestFilters}
              pagedBacktests={selectors.pagedBacktests}
              filteredBacktests={selectors.filteredBacktests}
              backtestCurrentPage={selectors.backtestCurrentPage}
              backtestTotalPages={selectors.backtestTotalPages}
              compareSelection={selectors.compareSelection}
              handleRefreshBacktestHistory={actions.handleRefreshBacktestHistory}
              setBacktestHistoryFilter={actions.setBacktestHistoryFilter}
              setBacktestCompileFilter={actions.setBacktestCompileFilter}
              setBacktestDatasetFilter={actions.setBacktestDatasetFilter}
              setBacktestParameterFilter={actions.setBacktestParameterFilter}
              setBacktestFromTime={actions.setBacktestFromTime}
              setBacktestToTime={actions.setBacktestToTime}
              setBacktestPage={actions.setBacktestPage}
              setBacktestPageSize={actions.setBacktestPageSize}
              toggleBacktestCompareSelection={actions.toggleBacktestCompareSelection}
              clearBacktestCompareSelection={actions.clearBacktestCompareSelection}
              loadBacktestDetail={actions.loadBacktestDetail}
              onOpenBacktestDetail={(backtestId) =>
                navigateTo(backtestDetailPath(backtestId, strategyId))
              }
            />
          </AnalysisSection>
        </div>

        <aside className="analysis-sidebar-column">
          <AnalysisSection
            kicker="范围"
            title="策略上下文"
            summary="在浏览研究轨迹时持续展示策略的编译身份。"
          >
            <div className="open-orders-card">
              <MetricPair label="策略 ID" value={strategyId} />
              <MetricPair label="策略名称" value={strategyName} />
              <MetricPair
                label="最近编译"
                value={graph.metadata?.runtime_binding?.last_compile_id || "-"}
              />
              <MetricPair label="协议" value={graph.compile_summary?.protocol_name || "-"} />
              <MetricPair label="配置哈希" value={graph.compile_summary?.config_hash || "-"} />
              <MetricPair label="数据集" value={datasetText} />
            </div>
          </AnalysisSection>

          <AnalysisSection
            kicker="队列"
            title="对比队列"
            summary="在进入全局对比视图前，持续显示当前策略范围内的对比选择。"
          >
            <div className="open-orders-card">
              <MetricPair
                label="已选择"
                value={
                  selectors.compareSelection.length === 0
                    ? "未选择回测"
                    : selectors.compareSelection.join(", ")
                }
              />
              <MetricPair label="可开始对比" value={compareButtonDisabled ? "否" : "是"} />
              <div className="toolbar-group">
                <button
                  className="ghost-btn compact-btn"
                  onClick={() => actions.clearBacktestCompareSelection()}
                >
                  清空选择
                </button>
                <button
                  className="primary-btn compact-btn"
                  disabled={compareButtonDisabled}
                  onClick={() =>
                    navigateTo(backtestComparePath(selectors.compareSelection, strategyId))
                  }
                >
                  打开对比
                </button>
              </div>
            </div>
          </AnalysisSection>
        </aside>
      </div>
    </div>
  );
}
