import {
  backtestComparePath,
  backtestDetailPath,
  navigateTo,
  strategyBacktestsPath
} from "../router";
import {
  OverviewList,
  WorkspaceActionCard,
  WorkspaceMetricCard,
  WorkspaceSection
} from "./StrategyWorkspacePageSections";
import { WorkspaceIssueQueueCard } from "./StrategyWorkspaceIssueQueueCard";
import StrategyWorkspaceCollaborationCard from "./StrategyWorkspaceCollaborationCard";
import StrategyWorkspaceExperimentCard from "./StrategyWorkspaceExperimentCard";
import StrategyWorkspaceVersionHistoryCard from "./StrategyWorkspaceVersionHistoryCard";
import { buildWorkspaceOverviewActionCards } from "./strategyWorkspaceDashboardOverviewShell";

export default function StrategyWorkspaceOverviewTab({
  strategyId,
  graph,
  ui,
  compileSummary,
  compileCounts,
  readiness,
  recentRuns,
  recentBacktests,
  compareSelection,
  issueQueue,
  canvasRecommendationState,
  overviewMetrics,
  runPreviewItems,
  overviewStatusHighlights,
  backtestPreviewItems,
  lastRun,
  lastBacktest,
  formatTime
}) {
  const overviewActionCards = buildWorkspaceOverviewActionCards({
    graph,
    compileCounts,
    recentRuns,
    recentBacktests
  }).map((item) => ({
    ...item,
    onClick: item.targetTab
      ? () => ui.setActiveTab(item.targetTab)
      : () => navigateTo(strategyBacktestsPath(strategyId))
  }));

  return (
    <div className="strategy-workspace-overview" data-testid="strategy-workspace-overview-tab">
      <div className="strategy-workspace-overview__main">
        <section className="workspace-overview-hero">
          <div className="workspace-overview-hero__main">
            <div className="workspace-overview-hero__eyebrow">工作区总览</div>
            <h2>从就绪状态、问题流和近期研究上下文推进策略。</h2>
            <p>
              总览只保留当前状态和下一步动作。需要更深操作时，再进入构建、诊断或研究。
            </p>
            <div className="workspace-overview-hero__status">
              <span className={`status-pill ${readiness.tone}`}>{readiness.label}</span>
              <span className="status-pill info">
                {compileSummary.protocol_name || "协议待生成"}
              </span>
              <span className="status-pill muted">对比队列 {compareSelection.length}/2</span>
            </div>
          </div>
          <div className="workspace-overview-hero__metrics">
            {overviewStatusHighlights.map((item) => (
              <div key={item.label} className="workspace-overview-hero__metric">
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <small>{item.note}</small>
              </div>
            ))}
          </div>
        </section>

        <section className="workspace-overview-actions">
          {overviewActionCards.map((item) => (
            <WorkspaceActionCard key={item.kicker} {...item} />
          ))}
        </section>

        <WorkspaceSection
          title="就绪度与阻塞项"
          subtitle="先查看当前编译与校验队列，必要时再打开完整诊断。"
          testId="workspace-readiness-section"
          actions={
            <button className="ad-btn ad-btn--ghost compact-btn" onClick={() => ui.setActiveTab("diagnostics")}>
              打开诊断
            </button>
          }
        >
          <WorkspaceIssueQueueCard
            title="修复队列"
            subtitle="优先处理最紧急、可定位的问题。"
            items={issueQueue}
            emptyText="当前策略图没有阻塞项。"
            actionLabel="打开诊断"
            onAction={() => ui.setActiveTab("diagnostics")}
            onSelectItem={ui.handleSelectIssueQueueItem}
            filters={ui.issueQueueFilters}
            onFiltersChange={ui.handleIssueQueueFiltersChange}
            graph={graph}
            repairPathState={canvasRecommendationState}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="研究快照"
          subtitle="先查看近期模拟和回测，再进入完整研究面板。"
          testId="workspace-research-section"
          actions={
            <button
              className="ad-btn ad-btn--ghost compact-btn"
              onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
            >
              打开回测索引
            </button>
          }
        >
          <div className="workspace-overview-grid workspace-overview-grid--dual">
            <OverviewList
              title="最近模拟"
              items={runPreviewItems}
              emptyText="该策略暂无最近模拟。"
            />
            <OverviewList
              title="最近回测"
              items={backtestPreviewItems}
              emptyText="该策略暂无最近回测。"
              renderActions={(item) => (
                <button
                  className="ad-btn ad-btn--ghost compact-btn"
                  onClick={() => navigateTo(backtestDetailPath(item.backtest_id, strategyId))}
                >
                  详情
                </button>
              )}
            />
          </div>
        </WorkspaceSection>
      </div>

      <aside className="strategy-workspace-overview__side">
        <div className="workspace-metric-grid">
          {overviewMetrics.map((item) => (
            <WorkspaceMetricCard key={item.label} {...item} />
          ))}
        </div>

        <WorkspaceSection
          title="当前上下文"
          subtitle="在工作区模式切换时保留高频策略上下文。"
        >
          <div className="strategy-inspector-metrics">
            <div className="kv-line">
              <span>最新编译 ID</span>
              <strong>{graph.metadata?.runtime_binding?.last_compile_id || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>配置哈希</span>
              <strong>{compileSummary.config_hash || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>策略中间表示角色</span>
              <strong>{compileSummary.artifact_resolution?.strategy_ir_role_label || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>运行来源</span>
              <strong>{compileSummary.artifact_resolution?.runtime_source_label || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>可运行依据</span>
              <strong>{compileSummary.artifact_resolution?.source_of_truth_label || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>最新模拟</span>
              <strong>{lastRun ? formatTime(lastRun.created_at_ms) : "-"}</strong>
            </div>
            <div className="kv-line">
              <span>最新回测</span>
              <strong>{lastBacktest ? formatTime(lastBacktest.created_at_ms) : "-"}</strong>
            </div>
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="所有者、协作者与审计"
          subtitle="在当前草稿内保留权限和近期图操作上下文。"
          testId="workspace-collaboration-section"
        >
          <StrategyWorkspaceCollaborationCard
            graphId={graph.metadata?.graph_id || strategyId || "draft_graph"}
            collaboration={graph.metadata?.collaboration}
            lastRun={lastRun}
            lastBacktest={lastBacktest}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="实验扫描"
          subtitle="执行窄范围执行假设扫描，并对比持久化回测结果。"
          testId="workspace-experiment-section"
        >
          <StrategyWorkspaceExperimentCard strategyId={strategyId} currentGraph={graph} />
        </WorkspaceSection>

        <WorkspaceSection
          title="持久化版本"
          subtitle="预览已保存版本，不覆盖当前草稿。"
          testId="workspace-persisted-versions-section"
        >
          <StrategyWorkspaceVersionHistoryCard
            graphId={graph.metadata?.graph_id || strategyId || "draft_graph"}
            currentGraph={graph}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="下一步"
          subtitle="直接给出下一步入口，减少寻找成本。"
        >
          <div className="strategy-inspector-actions">
            <button className="ad-btn ad-btn--primary" onClick={() => ui.setActiveTab("code")}>
              打开构建模式
            </button>
            <button className="ad-btn ad-btn--ghost" onClick={() => ui.setActiveTab("diagnostics")}>
              查看诊断
            </button>
            <button className="ad-btn ad-btn--ghost" onClick={() => ui.setActiveTab("research")}>
              打开研究
            </button>
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="对比队列"
          subtitle="在工作区内保留对比流程入口，不强制打开完整历史。"
        >
          <div className="strategy-compare-queue">
            <div className="strategy-compare-queue__chips">
              {compareSelection.length === 0 ? (
                <span className="status-pill muted">未选择回测</span>
              ) : (
                compareSelection.map((backtestId) => (
                  <span key={backtestId} className="status-pill info">
                    {backtestId}
                  </span>
                ))
              )}
            </div>
            <div className="strategy-inspector-actions">
              <button
                className="ad-btn ad-btn--primary"
                disabled={compareSelection.length !== 2}
                onClick={() => navigateTo(backtestComparePath(compareSelection, strategyId))}
              >
                打开对比
              </button>
              <button
                className="ad-btn ad-btn--ghost"
                onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
              >
                打开回测索引
              </button>
            </div>
          </div>
        </WorkspaceSection>
      </aside>
    </div>
  );
}
