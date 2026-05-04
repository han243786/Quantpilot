import { Suspense, lazy } from "react";
import {
  WorkspaceActionCard,
  WorkspaceMetricCard,
  WorkspaceSection
} from "./StrategyWorkspacePageSections";
import { WorkspaceIssueQueueCard } from "./StrategyWorkspaceIssueQueueCard";
import { diagnosticQueueSource } from "../utils/strategyWorkspaceIssueQueue";
import { WorkspacePanelFallback } from "./StrategyWorkspacePanelFallbacks";
import RuntimeDiagnosticsPanel from "../components/RuntimeDiagnosticsPanel";

const DiagnosticsPanel = lazy(() => import("../components/DiagnosticsPanel"));

function compileOutputsText(outputs) {
  if (!outputs) return "-";
  return [
    `${outputs.data_sources || 0} 数据`,
    `${outputs.intent_generators || 0} 意图`,
    `${outputs.agents || 0} 代理`,
    `${outputs.risk_controls || 0} 风控`,
    `${outputs.executions || 0} 执行`
  ].join(" / ");
}

export default function StrategyWorkspaceDiagnosticsTab({
  graph,
  runtime,
  selectedNodeId,
  ui,
  compileSummary,
  compileCounts,
  readiness,
  issueQueue,
  issueQueueCounts,
  issueQueueSources,
  issueQueueSourceCounts,
  diagnosticsStatusHighlights,
  canvasRecommendationState
}) {
  const diagnosticsSourceCards = issueQueueSources.map((source) => ({
    source,
    kicker: diagnosticQueueSource({ source }),
    title:
      source === "validation"
        ? "校验阻塞"
        : source === "runtime"
          ? "运行阻塞"
          : source === "strategy_ir"
            ? "Strategy IR 诊断"
            : "Formal QuantScript 诊断",
    note:
      source === "validation"
        ? "来自策略图校验和结构检查的问题。"
        : source === "runtime"
          ? "运行路径上发现的编译或执行阻塞。"
          : source === "strategy_ir"
            ? "与 Strategy IR 字段或生成过程相关的问题。"
            : "Formal 编写与编译管线内发现的问题。",
    meta: `${issueQueueSourceCounts[source] || 0} 项问题`,
    tone:
      source === "runtime"
        ? "danger"
        : source === "validation"
          ? "warning"
          : "info",
    cta: "筛选队列",
    onClick: () =>
      ui.handleIssueQueueFiltersChange({
        showSourceFilters: true,
        sourceFilter: source,
        nodeTypeFilter: "all"
      })
  }));

  return (
    <div className="strategy-workspace-diagnostics" data-testid="strategy-workspace-diagnostics-tab">
      <div className="strategy-workspace-diagnostics__main">
        <section className="workspace-diagnostics-hero">
          <div className="workspace-diagnostics-hero__main">
            <div className="workspace-diagnostics-hero__eyebrow">诊断总览</div>
            <h2>定位阻塞项、检查编译输出，并直接进入修复流程。</h2>
            <p>
              诊断聚焦当前修复路径。先从队列缩小范围，只有选中问题需要时再打开完整结构化诊断。
            </p>
            <div className="workspace-diagnostics-hero__status">
              <span className={`status-pill ${readiness.tone}`}>{readiness.label}</span>
              <span className="status-pill warning">
                {`${issueQueueCounts.error} 阻塞 / ${issueQueueCounts.warning} 警告`}
              </span>
              <span className="status-pill muted">{ui.diagnosticsQueueScope}</span>
            </div>
          </div>
          <div className="workspace-diagnostics-hero__metrics">
            {diagnosticsStatusHighlights.map((item) => (
              <div key={item.label} className="workspace-diagnostics-hero__metric">
                <span>{item.label}</span>
                <strong>{item.value}</strong>
                <small>{item.note}</small>
              </div>
            ))}
          </div>
        </section>

        <section className="workspace-diagnostics-actions" aria-label="诊断来源通道">
          {diagnosticsSourceCards.length > 0 ? (
            diagnosticsSourceCards.map((item) => <WorkspaceActionCard key={item.source} {...item} />)
          ) : (
            <div className="workspace-section-card workspace-diagnostics-empty-state">
              <div className="muted-line">
                暂无活跃问题来源。编译或校验策略后会生成通道。
              </div>
            </div>
          )}
        </section>

        <WorkspaceSection
          title="优先修复队列"
          subtitle="先从收窄后的修复队列开始，必要时再进入完整诊断。"
          testId="workspace-priority-repair-queue-section"
        >
          <WorkspaceIssueQueueCard
            title="优先修复"
            subtitle="直接定位阻塞当前编译路径的节点或路由。"
            items={issueQueue}
            emptyText="当前没有活跃修复项。"
            onSelectItem={ui.handleSelectIssueQueueItem}
            filters={ui.issueQueueFilters}
            onFiltersChange={ui.handleIssueQueueFiltersChange}
            graph={graph}
            repairPathState={canvasRecommendationState}
          />
        </WorkspaceSection>

        <WorkspaceSection
          title="结构化诊断"
          subtitle="在分流后保留完整编译视图，用户无需离开工作区就能检查完整上下文。"
          testId="workspace-structured-diagnostics-section"
          actions={
            <div className="strategy-inspector-actions">
              <button className="ghost-btn compact-btn" onClick={() => ui.setActiveTab("code")}>
                打开构建模式
              </button>
              <button className="ghost-btn compact-btn" onClick={() => ui.setActiveTab("research")}>
                打开研究
              </button>
            </div>
          }
        >
          <div className="workspace-metric-grid workspace-metric-grid--triple">
            <WorkspaceMetricCard
              label="可编译"
              value={compileSummary.compilable ? "是" : "否"}
              note={compileSummary.backend_verified ? "后端已验证" : "仅本地摘要"}
              tone={compileSummary.compilable ? "success" : "danger"}
            />
            <WorkspaceMetricCard
              label="协议"
              value={compileSummary.protocol_name || "-"}
              note={compileSummary.config_hash || "未记录配置哈希"}
              tone="info"
            />
            <WorkspaceMetricCard
              label="编译输出"
              value={compileOutputsText(compileSummary.outputs)}
              note="已生成的运行管线形态。"
              tone="muted"
            />
          </div>
          <Suspense fallback={<WorkspacePanelFallback title="正在加载诊断面板" />}>
            <DiagnosticsPanel
              compileSummary={compileSummary}
              onRouteDiagnostic={ui.handleRouteDiagnostic}
              graph={graph}
              repairPathState={canvasRecommendationState}
            />
          </Suspense>
        </WorkspaceSection>

        <WorkspaceSection
          title="运行诊断"
          subtitle="在编译队列旁保留节点级运行状态、最新输入输出快照和近期警告。"
          testId="workspace-runtime-diagnostics-section"
        >
          <RuntimeDiagnosticsPanel
            graph={graph}
            runtime={runtime}
            selectedNodeId={selectedNodeId}
            title="运行诊断"
            subtitle="使用当前节点选择和运行事件日志，检查所选节点最近收到、输出和报告的问题。"
          />
        </WorkspaceSection>
      </div>

      <aside className="strategy-workspace-diagnostics__side">
        <WorkspaceSection
          title="来源通道"
          subtitle="保持来源通道摘要可见，切换筛选时无需重新打开完整诊断面。"
        >
          <div className="workspace-diagnostics-source-list">
            {diagnosticsSourceCards.length === 0 ? (
              <div className="muted-line">暂无活跃来源通道。</div>
            ) : (
              diagnosticsSourceCards.map((item) => (
                <button
                  key={item.source}
                  type="button"
                  className={`workspace-diagnostics-source-item${
                    ui.issueQueueFilters.sourceFilter === item.source
                      ? " workspace-diagnostics-source-item--active"
                      : ""
                  }`}
                  onClick={item.onClick}
                >
                  <strong>{item.kicker}</strong>
                  <span>{item.meta}</span>
                </button>
              ))
            )}
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="编译上下文"
          subtitle="修复诊断时保留编译身份信息。"
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
              <span>协议</span>
              <strong>{compileSummary.protocol_name || "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Strategy IR 角色</span>
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
          </div>
        </WorkspaceSection>

        <WorkspaceSection
          title="下一步切换"
          subtitle="在结构修复和结果复盘之间快速切换。"
        >
          <div className="strategy-inspector-actions">
            <button className="primary-btn" onClick={() => ui.setActiveTab("code")}>
              在构建模式修复
            </button>
            <button className="ghost-btn" onClick={() => ui.setActiveTab("research")}>
              打开研究
            </button>
          </div>
        </WorkspaceSection>
      </aside>
    </div>
  );
}
