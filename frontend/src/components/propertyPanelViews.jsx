import { useEffect, useRef, useState } from "react";
import {
  deriveConfigureCardOrder,
  resolveConfigureIssueTargetCard
} from "../utils/configureFieldPriority";
import DiagnosticsPanel from "./DiagnosticsPanel";
import RuntimeDiagnosticsPanel from "./RuntimeDiagnosticsPanel";
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

import {
  ActionableValidationCard,
  ConnectionsCard,
  EdgeOverviewCard,
  NodeConfigCard,
  NodeMetricsCard,
  NodeOverviewCard,
  NodeQuantScriptCard,
  NodeRuntimeCard,
  ValidationCard
} from "./propertyPanelEntityCards";

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

export {
  ActionableValidationCard,
  ConnectionsCard,
  EdgeOverviewCard,
  NodeConfigCard,
  NodeMetricsCard,
  NodeOverviewCard,
  NodeQuantScriptCard,
  NodeRuntimeCard,
  ValidationCard
} from "./propertyPanelEntityCards";

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
