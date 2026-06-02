import { useEffect, useRef, useState } from "react";
import {
  deriveConfigureCardOrder,
  derivePriorityFieldGroups,
  resolveConfigureIssueTargetCard
} from "../utils/configureFieldPriority";
import { getRuntimeStatusMeta, runtimeStatusLabel } from "../utils/runtimeStatus";
import DiagnosticsPanel from "./DiagnosticsPanel";
import RuntimeDiagnosticsPanel from "./RuntimeDiagnosticsPanel";
import { formatValue } from "../hooks/propertyPanelShared";
import {
  FieldGroup,
  PropertyPanelShell,
  PropertySection,
  PropertySubsection,
  StatusChip,
  WorkspaceInspectorShell,
  renderFieldInput
} from "./propertyPanelLayoutPrimitives";

import {
  QuantScriptAuthoringFlowCard,
  QuantScriptAuthoringPoolCard,
  QuantScriptAuthoringSourceCard,
  QuantScriptAuthoringStateCard,
  sectionsToSelection
} from "./propertyPanelAuthoringCards";

import {
  CompileSummaryCard,
  FormalQuantScriptEditorCard,
  GraphOverviewCard,
  QuantScriptEditorCard,
  RepairPathContextPanel,
  StrategyIrEditorCard
} from "./propertyPanelCompileSourceCards";

export {
  FieldGroup,
  PropertyPanelShell,
  PropertySection,
  PropertySubsection,
  StatusChip,
  WorkspaceInspectorShell,
  renderFieldInput
} from "./propertyPanelLayoutPrimitives";

export {
  QuantScriptAuthoringFlowCard,
  QuantScriptAuthoringPoolCard,
  QuantScriptAuthoringSourceCard,
  QuantScriptAuthoringStateCard,
  lineRangeToSelection,
  sectionsToSelection
} from "./propertyPanelAuthoringCards";

export {
  CompileSummaryCard,
  FormalQuantScriptEditorCard,
  GraphOverviewCard,
  QuantScriptEditorCard,
  RepairPathContextPanel,
  StrategyIrEditorCard
} from "./propertyPanelCompileSourceCards";

export function NodeOverviewCard({ selectedNode, moduleDef, updateNodeName }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点概览</div>
        <div className="property-card-caption">把节点身份、模块边界和可编辑名称收在同一处。</div>
      </div>
      <label className="field-block">
        <span>节点名称</span>
        <input
          data-testid="prop-input-node-name"
          value={selectedNode.name}
          onChange={(event) => updateNodeName(selectedNode.id, event.target.value)}
        />
      </label>
      <div className="kv-line">
        <span>模块</span>
        <strong>{moduleDef?.display_name || selectedNode.module_key}</strong>
      </div>
      <div className="kv-line">
        <span>模块键</span>
        <strong>{selectedNode.module_key}</strong>
      </div>
      <div className="kv-line">
        <span>类别</span>
        <strong>{moduleDef?.category || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>节点 ID</span>
        <strong>{selectedNode.id}</strong>
      </div>
    </div>
  );
}

export function NodeConfigCard({
  selectedNode,
  moduleDef,
  updateNodeConfig,
  prioritizePathFields = false,
  nodeIssues = []
}) {
  const fieldGroups = derivePriorityFieldGroups({
    moduleDef,
    nodeIssues,
    nodeType: selectedNode?.type || null,
    prioritizePathFields
  });

  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">配置</div>
        <div className="property-card-caption">可编辑配置与运行状态、诊断信息保持分离。</div>
      </div>
      {(moduleDef?.config_schema?.fields || []).length === 0 ? (
        <div className="muted-line">这个节点当前没有可编辑配置项。</div>
      ) : null}
      {fieldGroups.map((group) => (
        <FieldGroup key={group.id} title={group.title} summary={group.summary}>
          {group.fields.map((field) => (
            <label key={field.key} className="field-block">
              <span>{field.label}</span>
              {renderFieldInput(field, selectedNode.config[field.key], (value) =>
                updateNodeConfig(selectedNode.id, field.key, value)
              )}
            </label>
          ))}
        </FieldGroup>
      ))}
    </div>
  );
}

export function ConnectionsCard({ graph, selectedNode }) {
  const incoming = graph.edges.filter((edge) => edge.target_node_id === selectedNode.id);
  const outgoing = graph.edges.filter((edge) => edge.source_node_id === selectedNode.id);

  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">连接关系</div>
        <div className="property-card-caption">在不离开节点面板的前提下查看上下游连线。</div>
      </div>
      <div className="mini-list">
        <div className="mini-list-title">输入</div>
        {incoming.length === 0 ? <div className="muted-line">当前没有输入边。</div> : null}
        {incoming.map((edge) => {
          const source = graph.nodes.find((node) => node.id === edge.source_node_id);
          return (
            <div key={edge.id} className="mini-item">
              {source?.name} -&gt; {edge.target_port}
            </div>
          );
        })}
      </div>
      <div className="mini-list">
        <div className="mini-list-title">输出</div>
        {outgoing.length === 0 ? <div className="muted-line">当前没有输出边。</div> : null}
        {outgoing.map((edge) => {
          const target = graph.nodes.find((node) => node.id === edge.target_node_id);
          return (
            <div key={edge.id} className="mini-item">
              {edge.source_port} -&gt; {target?.name}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function ValidationCard({ issues }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点校验</div>
        <div className="property-card-caption">节点级问题紧贴配置显示，不与编译诊断混在一起。</div>
      </div>
      {issues.length === 0 ? <div className="muted-line">这个节点当前没有校验问题。</div> : null}
      {issues.map((issue) => (
        <div key={issue.id} className={`issue-row issue-${issue.level}`}>
          <div className="issue-msg">{issue.message}</div>
          {issue.hint ? <div className="issue-hint">{issue.hint}</div> : null}
        </div>
      ))}
    </div>
  );
}

function configureCardLabel(cardId) {
  if (cardId === "connections") return "连接";
  if (cardId === "config") return "配置";
  return "校验";
}

export function ActionableValidationCard({ issues, onSelectIssue = null }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点校验</div>
        <div className="property-card-caption">节点级问题紧贴配置显示，不与编译诊断混在一起。</div>
      </div>
      {issues.length === 0 ? <div className="muted-line">这个节点当前没有校验问题。</div> : null}
      {issues.map((issue) => {
        const targetCardId = resolveConfigureIssueTargetCard(issue);
        const body = (
          <>
            <div className="issue-row__meta">
              <div className="issue-msg">{issue.message}</div>
              {onSelectIssue ? (
                <span className="diagnostic-chip diagnostic-chip--segment">
                  {configureCardLabel(targetCardId)}
                </span>
              ) : null}
            </div>
            {issue.hint ? <div className="issue-hint">{issue.hint}</div> : null}
          </>
        );

        if (!onSelectIssue) {
          return (
            <div key={issue.id} className={`issue-row issue-${issue.level}`}>
              {body}
            </div>
          );
        }

        return (
          <button
            key={issue.id}
            type="button"
            className={`issue-row issue-${issue.level} issue-row--actionable`}
            onClick={() => onSelectIssue(issue, targetCardId)}
          >
            {body}
          </button>
        );
      })}
    </div>
  );
}

export function NodeRuntimeCard({ selectedNode }) {
  const runtimeMeta = getRuntimeStatusMeta(selectedNode.runtime_state.status);
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">运行状态</div>
        <div className="property-card-caption">展示当前节点执行状态和最近一次可见运行信号。</div>
      </div>
      <div className="kv-line">
        <span>状态</span>
        <strong>
          <StatusChip tone={runtimeMeta.tone}>
            {runtimeStatusLabel(selectedNode.runtime_state.status)}
          </StatusChip>
        </strong>
      </div>
      <div className="kv-line">
        <span>最近事件</span>
        <strong>{selectedNode.runtime_state.last_event_type || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>最近消息</span>
        <strong>{selectedNode.runtime_state.last_message || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>最近时间</span>
        <strong>
          {selectedNode.runtime_state.last_event_time
            ? new Date(selectedNode.runtime_state.last_event_time).toLocaleTimeString()
            : "-"}
        </strong>
      </div>
    </div>
  );
}

export function NodeMetricsCard({ metrics }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">运行指标</div>
        <div className="property-card-caption">原始节点指标与状态文本分开显示，阅读路径更清晰。</div>
      </div>
      {metrics.length === 0 ? <div className="muted-line">当前还没有运行指标。</div> : null}
      {metrics.map(([key, value]) => (
        <div key={key} className="kv-line">
          <span>{key}</span>
          <strong>{formatValue(value)}</strong>
        </div>
      ))}
    </div>
  );
}

export function NodeQuantScriptCard({ nodeSource }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点源码工件</div>
        <div className="property-card-caption">只读展示当前节点对应的 graph-source 输出。</div>
      </div>
      <textarea readOnly value={nodeSource} rows={10} />
    </div>
  );
}

export function EdgeOverviewCard({ selectedEdge, sourceNode, targetNode, removeSelected }) {
  return (
    <div className="property-card">
      <div className="kv-line">
        <span>源节点</span>
        <strong>{sourceNode?.name || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>目标节点</span>
        <strong>{targetNode?.name || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>端口映射</span>
        <strong>
          {selectedEdge.source_port} -&gt; {selectedEdge.target_port}
        </strong>
      </div>
      <button className="ad-btn ad-btn--danger" onClick={removeSelected} data-testid="prop-action-delete-edge">
        删除边
      </button>
    </div>
  );
}

export function GraphConfigSection({ model }) {
  return (
    <PropertySection
      kicker="配置"
      title="策略图"
      summary="图的核心身份和问题计数保持在最上方。"
      testId="property-section-graph-config"
    >
      <GraphOverviewCard graph={model.graph} compileSummary={model.compileSummary} />
    </PropertySection>
  );
}

export function DiagnosticsSection({
  model,
  onRouteDiagnostic = null,
  graph = null,
  repairPathState = null
}) {
  return (
    <PropertySection
      kicker="编译"
      title="编译与诊断"
      summary="将编译结论、预检边界和结构化诊断收在一起。"
      testId="property-section-diagnostics"
    >
      <CompileSummaryCard compileSummary={model.compileSummary} />
      <DiagnosticsPanel
        compileSummary={model.compileSummary}
        onRouteDiagnostic={onRouteDiagnostic}
        graph={graph}
        repairPathState={repairPathState}
      />
    </PropertySection>
  );
}

export function SourceSection({
  model,
  includeNodeSource = false,
  onActivateSourceLane = null
}) {
  const [activeSectionIds, setActiveSectionIds] = useState([]);
  const [activeEdgeKey, setActiveEdgeKey] = useState(null);
  const [activeFormalSourceSelection, setActiveFormalSourceSelection] = useState(null);
  const graphSourceEditorRef = useRef(null);
  const formalSourceEditorRef = useRef(null);

  useEffect(() => {
    setActiveSectionIds([]);
    setActiveEdgeKey(null);
    setActiveFormalSourceSelection(null);
  }, [model.authoringView, model.formalQuantScriptSource]);

  function selectSections(sectionIds, edgeKey = null) {
    const sections = (model.authoringView?.sections || []).filter((section) =>
      sectionIds.includes(section.id)
    );
    if (!sections.length) return;
    onActivateSourceLane?.();
    setActiveSectionIds(sections.map((section) => section.id));
    setActiveEdgeKey(edgeKey);
    setActiveFormalSourceSelection(sectionsToSelection(model.formalQuantScriptSource, sections));
  }

  const authoringView = model.authoringView
    ? {
        ...model.authoringView,
        activeSectionIds,
        activeEdgeKey,
        onSelectSection: (sectionIds) => selectSections(sectionIds, null),
        onSelectEdge: (edge) =>
          selectSections([edge.from, edge.to], `${edge.from}_${edge.to}_${edge.reason}`)
      }
    : null;

  return (
    <PropertySection
      kicker="源码"
      title="脚本与 IR"
      summary="在不干扰编译结论的前提下编辑 graph-source 与 IR 工件。"
      testId="property-section-source"
    >
      {includeNodeSource ? <NodeQuantScriptCard nodeSource={model.nodeSource} /> : null}
      <QuantScriptAuthoringStateCard authoringViewState={model.authoringViewState} />
      <QuantScriptAuthoringSourceCard authoringView={authoringView} />
      <QuantScriptAuthoringFlowCard authoringView={authoringView} />
      <QuantScriptAuthoringPoolCard authoringView={authoringView} />
      <FormalQuantScriptEditorCard
        formalQuantScriptSource={model.formalQuantScriptSource}
        formalQuantScriptOverrideActive={model.formalQuantScriptOverrideActive}
        formalApplyError={model.formalApplyError}
        updateFormalQuantScriptDraft={model.updateFormalQuantScriptDraft}
        handleResetFormalQuantScript={model.handleResetFormalQuantScript}
        handleApplyFormalQuantScript={model.handleApplyFormalQuantScript}
        setFormalApplyError={model.setFormalApplyError}
        onActivateSourceLane={onActivateSourceLane}
        formalSourceEditorRef={formalSourceEditorRef}
        activeFormalSourceSelection={activeFormalSourceSelection}
      />
      <QuantScriptEditorCard
        graphSource={model.graphSource}
        sourceMode={model.graph.metadata.source_mode}
        applyError={model.applyError}
        updateQuantScriptDraft={model.updateQuantScriptDraft}
        handleResetQuantScript={model.handleResetQuantScript}
        handleApplyQuantScript={model.handleApplyQuantScript}
        setApplyError={model.setApplyError}
        onActivateSourceLane={onActivateSourceLane}
        graphSourceEditorRef={graphSourceEditorRef}
        activeSourceSelection={null}
      />
      <StrategyIrEditorCard
        strategyIrSource={model.strategyIrSource}
        strategyIrEditorRef={model.strategyIrEditorRef}
        selectedCompileDiagnosticTarget={model.selectedCompileDiagnosticTarget}
        strategyIrApplyError={model.strategyIrApplyError}
        updateStrategyIrDraft={model.updateStrategyIrDraft}
        handleResetStrategyIr={model.handleResetStrategyIr}
        handleApplyStrategyIr={model.handleApplyStrategyIr}
        setStrategyIrApplyError={model.setStrategyIrApplyError}
        onActivateSourceLane={onActivateSourceLane}
      />
    </PropertySection>
  );
}

export function NodeParamsSection({ model, prioritizePathFields = false }) {
  return (
    <PropertySection
      kicker="配置"
      title="节点设置"
      summary="将可编辑字段、校验和接线关系放进同一条设置泳道。"
      testId="property-section-node-params"
    >
      <NodeOverviewCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeName={model.updateNodeName}
      />
      <NodeConfigCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeConfig={model.updateNodeConfig}
        prioritizePathFields={prioritizePathFields}
        nodeIssues={model.nodeIssues}
      />
      <ConnectionsCard graph={model.graph} selectedNode={model.selectedNode} />
      <ValidationCard issues={model.nodeIssues} />
      <button className="ad-btn ad-btn--danger full-width" onClick={model.removeSelected} data-testid="prop-action-delete-node">
        删除节点
      </button>
    </PropertySection>
  );
}

export function LaneAwareNodeParamsSection({ model, prioritizePathFields = false }) {
  const [activeCardId, setActiveCardId] = useState(null);
  const cardRefs = useRef({});
  const selectedNodeId = model.selectedNode?.id || null;
  const cardOrder = deriveConfigureCardOrder({
    nodeIssues: model.nodeIssues,
    prioritizePathFields
  });

  useEffect(() => {
    setActiveCardId(null);
  }, [selectedNodeId]);

  useEffect(() => {
    if (!activeCardId) return;
    const cardNode = cardRefs.current[activeCardId];
    if (cardNode && typeof cardNode.scrollIntoView === "function") {
      cardNode.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [activeCardId]);

  function handleSelectIssue(issue, targetCardId) {
    setActiveCardId(targetCardId || resolveConfigureIssueTargetCard(issue));
  }

  const orderedCards = {
    config: (
      <NodeConfigCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeConfig={model.updateNodeConfig}
        prioritizePathFields={prioritizePathFields}
        nodeIssues={model.nodeIssues}
      />
    ),
    connections: <ConnectionsCard graph={model.graph} selectedNode={model.selectedNode} />,
    validation: (
      <ActionableValidationCard issues={model.nodeIssues} onSelectIssue={handleSelectIssue} />
    )
  };

  return (
    <PropertySection
      kicker="配置"
      title="节点设置"
      summary="先固定节点身份，再根据当前问题类型重新排序配置、连接和校验区域。"
      testId="property-section-node-params"
    >
      <NodeOverviewCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeName={model.updateNodeName}
      />
      {cardOrder.map((cardId) => (
        <div
          key={cardId}
          ref={(node) => {
            cardRefs.current[cardId] = node;
          }}
          data-configure-card={cardId}
          className={`configure-card-anchor${
            activeCardId === cardId ? " configure-card-anchor--active" : ""
          }`}
        >
          {orderedCards[cardId]}
        </div>
      ))}
      <button className="ad-btn ad-btn--danger full-width" onClick={model.removeSelected} data-testid="prop-action-delete-node">
        删除节点
      </button>
    </PropertySection>
  );
}

export function NodeRuntimeSection({ model }) {
  return (
    <PropertySection
      kicker="运行"
      title="运行状态"
      summary="将当前节点状态与原始运行指标分开展示，便于快速扫读。"
      testId="property-section-node-runtime"
    >
      <NodeRuntimeCard selectedNode={model.selectedNode} />
      <RuntimeDiagnosticsPanel
        graph={model.graph}
        runtime={model.runtime}
        selectedNodeId={model.selectedNode?.id || null}
        title="节点运行诊断"
        subtitle="把当前节点的最近事件、输入输出快照和最近一次警告集中到同一张卡片。"
      />
      <NodeMetricsCard metrics={model.nodeMetrics} />
    </PropertySection>
  );
}
