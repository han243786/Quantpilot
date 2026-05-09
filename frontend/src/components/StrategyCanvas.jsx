import {
  Suspense,
  lazy,
  startTransition,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState
} from "react";
import {
  Background,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  useStore,
  useViewport
} from "@xyflow/react";
import BaseNodeCard from "../nodes/BaseNodeCard";
import { buildNodeCardData } from "../nodes/nodeCardPresentation";
import { isValidConnection } from "../graph/validation";
import { useGraphStore } from "../store/graphStore";
import {
  buildCanvasFocusBounds,
  collectIssueNodeIds,
  collectRecentNodeIds,
  CANVAS_FOCUS_MODES,
  cycleCanvasFocusTarget,
  resolveCanvasActiveTargetId,
  resolveCanvasFocusAnchorId,
  resolveCanvasRecommendations,
  resolveCanvasFocusTargetIds
} from "./strategyCanvasFocus";
import { collectVisibleNodeIds } from "./strategyCanvasViewport";
import { canvasFocusStatusLabel } from "../utils/workspaceContextLabels";

const StrategyCanvasMiniMap = lazy(() => import("./StrategyCanvasMiniMap"));

const nodeTypes = {
  data: BaseNodeCard,
  intent: BaseNodeCard,
  agent: BaseNodeCard,
  risk: BaseNodeCard,
  execution: BaseNodeCard,
  runtime: BaseNodeCard
};

const CANVAS_LANE_ORDER = ["data", "intent", "agent", "risk", "execution", "runtime"];
const CANVAS_LANE_LABELS = {
  data: "数据",
  intent: "意图",
  agent: "代理",
  risk: "风控",
  execution: "执行",
  runtime: "运行时"
};

function resolveNodeCardMode() {
  if (typeof window === "undefined") return "staged";
  const params = new URLSearchParams(window.location.search);
  return params.get("node_card_mode") === "full" ? "full" : "staged";
}

function CanvasOverlayFallback() {
  return <div className="canvas-overlay-skeleton" aria-hidden="true" />;
}

function scheduleAfterFirstPaint(callback) {
  if (typeof window === "undefined") {
    callback();
    return () => {};
  }

  let frameId = null;
  let idleId = null;
  let timeoutId = null;
  let disposed = false;

  const run = () => {
    if (disposed) return;
    startTransition(() => {
      callback();
    });
  };

  const queueIdle = () => {
    if (typeof window.requestIdleCallback === "function") {
      idleId = window.requestIdleCallback(run, { timeout: 600 });
      return;
    }
    timeoutId = window.setTimeout(run, 0);
  };

  frameId = window.requestAnimationFrame(queueIdle);

  return () => {
    disposed = true;
    if (frameId !== null) window.cancelAnimationFrame(frameId);
    if (idleId !== null && typeof window.cancelIdleCallback === "function") {
      window.cancelIdleCallback(idleId);
    }
    if (timeoutId !== null) window.clearTimeout(timeoutId);
  };
}

function focusCanvasTargets(reactFlow, nodes, targetIds, anchorId = null) {
  if (!Array.isArray(targetIds) || targetIds.length === 0) return;

  if (anchorId) {
    const node = nodes.find((item) => item.id === anchorId);
    if (!node) return;

    reactFlow.setCenter(node.position.x + 120, node.position.y + 60, {
      zoom: 0.92,
      duration: 260
    });
    return;
  }

  const bounds = buildCanvasFocusBounds(nodes, targetIds);
  if (!bounds) return;

  if (targetIds.length === 1) {
    const node = nodes.find((item) => item.id === targetIds[0]);
    if (!node) return;

    reactFlow.setCenter(node.position.x + 120, node.position.y + 60, {
      zoom: 0.92,
      duration: 260
    });
    return;
  }

  reactFlow.fitBounds(bounds, {
    duration: 280,
    padding: 0.18
  });
}

function FlowInner({
  focusMode,
  focusTargetIds,
  focusAnchorId,
  recommendedNodeIds = [],
  repairPathNodeIds = [],
  repairPathEdgeIds = []
}) {
  const graph = useGraphStore((state) => state.graph);
  const registry = useGraphStore((state) => state.registry);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const selectedEdgeId = useGraphStore((state) => state.selectedEdgeId);
  const highlightedNodeIds = useGraphStore((state) => state.runtime.highlightedNodeIds || []);
  const setSelectedNode = useGraphStore((state) => state.setSelectedNode);
  const setSelectedEdge = useGraphStore((state) => state.setSelectedEdge);
  const updateNodePosition = useGraphStore((state) => state.updateNodePosition);
  const addGraphEdge = useGraphStore((state) => state.addEdge);
  const updateEditorViewport = useGraphStore((state) => state.updateEditorViewport);
  const reactFlow = useReactFlow();
  const viewportWidth = useStore((state) => state.width);
  const viewportHeight = useStore((state) => state.height);
  const viewport = useViewport();
  const [showCanvasDecorations, setShowCanvasDecorations] = useState(false);
  const [showMiniMap, setShowMiniMap] = useState(false);
  const [showRichNodeCards, setShowRichNodeCards] = useState(resolveNodeCardMode() === "full");
  const [showDeferredFlowDetails, setShowDeferredFlowDetails] = useState(
    resolveNodeCardMode() === "full"
  );
  const lastAppliedFocusKeyRef = useRef("");
  const focusTargetKey = useMemo(() => focusTargetIds.join("|"), [focusTargetIds]);
  const focusNodePositionKey = useMemo(
    () =>
      focusTargetIds
        .map((targetId) => {
          const node = graph.nodes.find((item) => item.id === targetId);
          if (!node) return `${targetId}:missing`;
          return `${targetId}:${node.position.x}:${node.position.y}`;
        })
        .join("|"),
    [focusTargetIds, graph.nodes]
  );
  const focusTargetSet = useMemo(() => new Set(focusTargetIds), [focusTargetIds]);
  const recommendedNodeSet = useMemo(() => new Set(recommendedNodeIds), [recommendedNodeIds]);
  const repairPathEdgeSet = useMemo(() => new Set(repairPathEdgeIds), [repairPathEdgeIds]);

  const defaultViewport = graph.metadata.editor?.viewport || { x: 0, y: 0, zoom: 0.8 };
  const defaultViewportKey = JSON.stringify(defaultViewport);
  const initializedRef = useRef(Boolean(graph.metadata.editor?.viewport));
  const appliedViewportRef = useRef(defaultViewportKey);
  const nodeCardMode = resolveNodeCardMode();
  const nodeIssues = graph.validation_state?.node_issues || {};
  const visibleNodeIds = useMemo(
    () =>
      collectVisibleNodeIds(graph.nodes, viewport, {
        width: viewportWidth,
        height: viewportHeight
      }),
    [graph.nodes, viewport.x, viewport.y, viewport.zoom, viewportHeight, viewportWidth]
  );

  const nodes = useMemo(
    () =>
      graph.nodes.map((node) => ({
        id: node.id,
        type: node.type,
        position: node.position,
        selected: selectedNodeId === node.id,
        hidden:
          nodeCardMode !== "full" &&
          !showDeferredFlowDetails &&
          node.type === "runtime" &&
          selectedNodeId !== node.id,
        data: buildNodeCardData({
          node,
          registry,
          nodeIssues,
          highlightedNodeIds,
          simplified:
            nodeCardMode !== "full" && !showRichNodeCards && selectedNodeId !== node.id,
          showHandles: selectedNodeId === node.id || visibleNodeIds.has(node.id),
          focusMode,
          focusedNodeIds: focusTargetSet,
          recommendedNodeIds: recommendedNodeSet,
          repairPathNodeIds,
          selectedNodeId
        })
      })),
    [
      focusMode,
      focusTargetSet,
      graph.nodes,
      highlightedNodeIds,
      nodeCardMode,
      nodeIssues,
      recommendedNodeSet,
      repairPathNodeIds,
      registry,
      selectedNodeId,
      showDeferredFlowDetails,
      showRichNodeCards,
      visibleNodeIds
    ]
  );

  const edges = useMemo(
    () =>
      (!showDeferredFlowDetails && nodeCardMode !== "full" ? [] : graph.edges).map((edge) => {
        const isSelected = selectedEdgeId === edge.id;
        const isHighlighted =
          highlightedNodeIds.includes(edge.source_node_id) &&
          highlightedNodeIds.includes(edge.target_node_id);
        const isFocusEdge =
          focusTargetSet.size > 0 &&
          focusTargetSet.has(edge.source_node_id) &&
          focusTargetSet.has(edge.target_node_id);
        const isRepairPathEdge = repairPathEdgeSet.has(edge.id);
        const isDimmed = focusTargetSet.size > 0 && !isFocusEdge;

        return {
          id: edge.id,
          type: "default",
          source: edge.source_node_id,
          target: edge.target_node_id,
          sourceHandle: edge.source_port,
          targetHandle: edge.target_port,
          className: `${isHighlighted ? "highlighted-runtime-edge" : ""} ${
            isRepairPathEdge ? "repair-path-edge" : ""
          }`.trim(),
          style: {
            stroke: isSelected
              ? "var(--ad-warning)"
              : isHighlighted
                ? "var(--ad-text-secondary)"
                : isRepairPathEdge
                  ? "var(--ad-success)"
                : isFocusEdge
                  ? "var(--ad-text-secondary)"
                  : "var(--ad-text-muted)",
            strokeWidth: isSelected ? 2.4 : isHighlighted || isFocusEdge || isRepairPathEdge ? 2.2 : 1.6,
            opacity: isDimmed ? 0.24 : 1
          }
        };
      }),
    [
      focusTargetSet,
      graph.edges,
      highlightedNodeIds,
      nodeCardMode,
      repairPathEdgeSet,
      selectedEdgeId,
      showDeferredFlowDetails
    ]
  );

  const connectValidator = useCallback(
    (connection) => isValidConnection(graph, registry, connection).valid,
    [graph, registry]
  );

  useEffect(() => scheduleAfterFirstPaint(() => setShowCanvasDecorations(true)), []);

  useEffect(() => {
    if (nodeCardMode === "full" || showRichNodeCards) return undefined;
    return scheduleAfterFirstPaint(() => setShowRichNodeCards(true));
  }, [nodeCardMode, showRichNodeCards]);

  useEffect(() => {
    if (nodeCardMode === "full" || showDeferredFlowDetails) return undefined;
    return scheduleAfterFirstPaint(() => setShowDeferredFlowDetails(true));
  }, [nodeCardMode, showDeferredFlowDetails]);

  useEffect(() => {
    if (!showCanvasDecorations) return undefined;
    return scheduleAfterFirstPaint(() => setShowMiniMap(true));
  }, [showCanvasDecorations]);

  useEffect(() => {
    if (focusTargetKey.length === 0) {
      lastAppliedFocusKeyRef.current = "";
      return undefined;
    }

    const nextFocusKey = [
      focusMode,
      focusAnchorId || "",
      focusTargetKey,
      focusNodePositionKey
    ].join("::");

    if (lastAppliedFocusKeyRef.current === nextFocusKey) {
      return undefined;
    }

    lastAppliedFocusKeyRef.current = nextFocusKey;
    return scheduleAfterFirstPaint(() =>
      focusCanvasTargets(reactFlow, graph.nodes, focusTargetIds, focusAnchorId)
    );
  }, [focusAnchorId, focusMode, focusNodePositionKey, focusTargetKey, reactFlow]);

  useEffect(() => {
    const nextViewport = graph.metadata.editor?.viewport;
    if (!nextViewport) return;

    const viewportKey = JSON.stringify(nextViewport);
    if (!initializedRef.current) {
      initializedRef.current = true;
      appliedViewportRef.current = viewportKey;
      return;
    }

    if (appliedViewportRef.current !== viewportKey) {
      reactFlow.setViewport(nextViewport, { duration: 0 });
      appliedViewportRef.current = viewportKey;
    }
  }, [defaultViewportKey, graph.metadata.editor, reactFlow]);

  return (
    <ReactFlow
      className="quantpilot-flow"
      proOptions={{ hideAttribution: true }}
      nodes={nodes}
      edges={edges}
      nodeTypes={nodeTypes}
      defaultViewport={defaultViewport}
      onNodeDrag={(_, node) => updateNodePosition(node.id, node.position, false)}
      onNodeDragStop={(_, node) => updateNodePosition(node.id, node.position, true)}
      onMove={(_, nextViewport) => updateEditorViewport(nextViewport, false)}
      onMoveEnd={(_, nextViewport) => {
        appliedViewportRef.current = JSON.stringify(nextViewport);
        updateEditorViewport(nextViewport, true);
      }}
      onConnect={(connection) => addGraphEdge(connection)}
      isValidConnection={connectValidator}
      onNodeClick={(_, node) => setSelectedNode(node.id)}
      onEdgeClick={(_, edge) => setSelectedEdge(edge.id)}
      onPaneClick={() => {
        setSelectedNode(null);
        setSelectedEdge(null);
      }}
      defaultEdgeOptions={{ type: "default" }}
      nodesDraggable
      panOnDrag
      selectionOnDrag
    >
      {showCanvasDecorations ? <Background gap={24} color="#1a1e24" /> : null}
      <Suspense fallback={<CanvasOverlayFallback />}>
        {showMiniMap ? (
          <StrategyCanvasMiniMap
            graphNodes={graph.nodes}
            graphEdges={graph.edges}
            highlightedNodeIds={highlightedNodeIds}
          />
        ) : (
          <CanvasOverlayFallback />
        )}
      </Suspense>
    </ReactFlow>
  );
}

function canvasFocusState(graph, selectedNodeId, focusMode) {
  const targetIds = resolveCanvasFocusTargetIds(graph, selectedNodeId, focusMode);
  const activeTargetId = resolveCanvasActiveTargetId(targetIds, selectedNodeId);
  const focusAnchorId = resolveCanvasFocusAnchorId(targetIds, selectedNodeId, focusMode);

  if (focusMode === "issues") {
    return {
      targetIds,
      activeTargetId,
      focusAnchorId,
      label: canvasFocusStatusLabel(focusMode),
      badge: `问题节点 ${targetIds.length}`,
      note:
        targetIds.length > 0
          ? "将视口收拢到当前正在阻塞校验的节点。"
          : "当前没有激活的节点级问题，因此画布保持完整结构视图。"
    };
  }

  if (focusMode === "recent") {
    return {
      targetIds,
      activeTargetId,
      focusAnchorId,
      label: canvasFocusStatusLabel(focusMode),
      badge: `最近编辑 ${targetIds.length}`,
      note:
        targetIds.length > 0
          ? "跟踪最近新建、移动或重新接线的节点，同时不丢失其余结构上下文。"
          : "当前还没有最近结构编辑记录。"
    };
  }

  return {
    targetIds,
    activeTargetId,
    focusAnchorId,
    label: canvasFocusStatusLabel(focusMode),
    badge: selectedNodeId ? "已选节点 1" : "已选节点 0",
    note: selectedNodeId
      ? "调整配置时，让视口持续围绕当前节点。"
      : "选择一个节点即可进入单节点聚焦。"
  };
}

export default function StrategyCanvas({
  focusMode: controlledFocusMode = null,
  onFocusModeChange = null,
  workspaceContext = null,
  recommendationStateOverride = null
}) {
  const graph = useGraphStore((state) => state.graph);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const setSelectedNode = useGraphStore((state) => state.setSelectedNode);
  const [localFocusMode, setLocalFocusMode] = useState("selected");
  const focusMode = controlledFocusMode || localFocusMode;
  const handleFocusModeChange = onFocusModeChange || setLocalFocusMode;
  const nodeIssues = graph.validation_state?.node_issues || {};

  const focusState = useMemo(
    () => canvasFocusState(graph, selectedNodeId, focusMode),
    [focusMode, graph, selectedNodeId]
  );

  const selectedNode = useMemo(
    () => (selectedNodeId ? graph.nodes.find((node) => node.id === selectedNodeId) || null : null),
    [graph.nodes, selectedNodeId]
  );
  const recommendationState = useMemo(
    () =>
      recommendationStateOverride ||
      resolveCanvasRecommendations(graph, selectedNodeId, workspaceContext?.laneId || null),
    [graph, recommendationStateOverride, selectedNodeId, workspaceContext?.laneId]
  );

  const laneSummary = useMemo(() => {
    const focusTargetSet = new Set(focusState.targetIds);

    return CANVAS_LANE_ORDER.map((lane) => {
      const laneNodes = graph.nodes.filter((node) => node.type === lane);
      return {
        lane,
        count: laneNodes.length,
        issueCount: laneNodes.filter(
          (node) => Array.isArray(nodeIssues[node.id]) && nodeIssues[node.id].length > 0
        ).length,
        focusCount: laneNodes.filter((node) => focusTargetSet.has(node.id)).length
      };
    });
  }, [focusState.targetIds, graph.nodes, nodeIssues]);

  const selectedNodeHealth = selectedNode
    ? Array.isArray(nodeIssues[selectedNode.id]) && nodeIssues[selectedNode.id].length > 0
      ? nodeIssues[selectedNode.id][0]?.message || "当前节点存在校验问题。"
      : "当前节点没有校验阻塞。"
    : "选择一个节点，让检查器与画布保持对齐。";

  const focusTargets = useMemo(() => {
    const nodeMap = new Map(graph.nodes.map((node) => [node.id, node]));

    return focusState.targetIds
      .map((targetId) => {
        const node = nodeMap.get(targetId);
        if (!node) return null;

        return {
          id: targetId,
          name: node.name || targetId,
          type: node.type,
          issue: nodeIssues[targetId]?.[0]?.message || null
        };
      })
      .filter(Boolean);
  }, [focusState.targetIds, graph.nodes, nodeIssues]);

  const activeFocusIndex = useMemo(
    () => focusTargets.findIndex((target) => target.id === focusState.activeTargetId),
    [focusState.activeTargetId, focusTargets]
  );
  const recommendedTargets = useMemo(() => {
    const nodeMap = new Map(graph.nodes.map((node) => [node.id, node]));
    return recommendationState.recommendedNodeIds
      .map((nodeId) => {
        const node = nodeMap.get(nodeId);
        if (!node) return null;
        return {
          id: node.id,
          name: node.name || node.id,
          type: node.type,
          issue: nodeIssues[node.id]?.[0]?.message || null
        };
      })
      .filter(Boolean);
  }, [graph.nodes, nodeIssues, recommendationState.recommendedNodeIds]);
  const repairPathTargets = useMemo(() => {
    const nodeMap = new Map(graph.nodes.map((node) => [node.id, node]));
    return recommendationState.pathNodeIds
      .map((nodeId) => {
        const node = nodeMap.get(nodeId);
        if (!node) return null;
        return {
          id: node.id,
          name: node.name || node.id,
          type: node.type,
          issue: nodeIssues[node.id]?.[0]?.message || null
        };
      })
      .filter(Boolean);
  }, [graph.nodes, nodeIssues, recommendationState.pathNodeIds]);
  const recommendationSummary = useMemo(() => {
    if (!workspaceContext?.laneId || !selectedNode) return null;
    if (workspaceContext.laneId === "diagnostics") {
      return {
        title: `${workspaceContext.laneLabel}推荐节点`,
        note: `在${workspaceContext.laneLabel}激活期间，让 ${selectedNode.type} 选中项始终和下一批修复候选节点保持连贯。`,
        pathTitle: "建议修复路径",
        pathNote:
          repairPathTargets.length > 1
            ? "按从左到右的顺序沿路径推进，从当前信号逐步进入下一个阻塞检查点。"
            : "选择一个推荐节点，让画布对齐到下一个修复目标。"
      };
    }
    return null;
  }, [repairPathTargets.length, selectedNode, workspaceContext]);

  const handleCycleFocusTarget = useCallback(
    (direction) => {
      const nextTargetId = cycleCanvasFocusTarget(
        focusState.targetIds,
        focusState.activeTargetId,
        direction
      );
      if (nextTargetId) {
        setSelectedNode(nextTargetId);
      }
    },
    [focusState.activeTargetId, focusState.targetIds, setSelectedNode]
  );
  const handleRecommendationClick = useCallback(
    (nodeId) => {
      setSelectedNode(nodeId);
      if (workspaceContext?.laneId === "diagnostics" && focusMode !== "issues") {
        handleFocusModeChange("issues");
      }
    },
    [focusMode, handleFocusModeChange, setSelectedNode, workspaceContext?.laneId]
  );

  return (
    <section className="strategy-canvas-shell">
      <div className="canvas-header">
        <div className="canvas-header-main">
          <div>
            <div className="panel-title">结构画布</div>
            <div className="canvas-subtitle">
              用这块图形画布进行结构编辑、问题路由与定向视口操作。
            </div>
          </div>
          <div className="canvas-stage-pill">工作台</div>
        </div>

        <div className="canvas-focus-toolbar" role="tablist" aria-label="画布聚焦模式">
          {CANVAS_FOCUS_MODES.map((mode) => {
            const label =
              mode === "issues" ? "聚焦问题" : mode === "recent" ? "聚焦最近编辑" : "聚焦节点";

            return (
              <button
                key={mode}
                type="button"
                role="tab"
                aria-selected={focusMode === mode}
                className={`canvas-focus-toolbar__tab ${
                  focusMode === mode ? "canvas-focus-toolbar__tab--active" : ""
                }`}
                data-testid={`canvas-focus-tab-${mode}`}
                onClick={() => handleFocusModeChange(mode)}
              >
                {label}
              </button>
            );
          })}
        </div>

        <div className="canvas-focus-summary">
          <span className="status-pill info">{focusState.badge}</span>
          <span className="canvas-focus-summary__note">{focusState.note}</span>
        </div>

        {recommendationSummary && recommendedTargets.length > 0 ? (
          <div className="canvas-recommendation-panel" data-testid="canvas-recommendation-panel">
            <div className="canvas-recommendation-panel__header">
              <div>
                <div className="canvas-recommendation-panel__title">
                  {recommendationSummary.title}
                </div>
                <div className="canvas-recommendation-panel__note">
                  {recommendationSummary.note}
                </div>
              </div>
              <span className="status-pill info">{`${recommendedTargets.length} 个节点`}</span>
            </div>

            <div
              className="canvas-recommendation-panel__targets"
              data-testid="canvas-recommendation-targets"
            >
              {recommendedTargets.map((target) => (
                <button
                  key={target.id}
                  type="button"
                  className={`canvas-recommendation-target ${
                    target.id === selectedNodeId ? "canvas-recommendation-target--active" : ""
                  }`}
                  data-testid={`canvas-recommendation-target-${target.id}`}
                  onClick={() => handleRecommendationClick(target.id)}
                >
                  <span className="canvas-recommendation-target__name">{target.name}</span>
                  <span className="canvas-recommendation-target__meta">{target.type}</span>
                  {target.issue ? (
                    <span className="canvas-recommendation-target__note">{target.issue}</span>
                  ) : null}
                </button>
              ))}
            </div>

            {repairPathTargets.length > 1 ? (
              <div className="canvas-repair-path" data-testid="canvas-repair-path">
                <div className="canvas-repair-path__header">
                  <div className="canvas-repair-path__title">{recommendationSummary.pathTitle}</div>
                  <div className="canvas-repair-path__note">{recommendationSummary.pathNote}</div>
                </div>
                <div className="canvas-repair-path__steps" data-testid="canvas-repair-path-steps">
                  {repairPathTargets.map((target, index) => (
                    <div key={target.id} className="canvas-repair-path__step">
                      <button
                        type="button"
                        className={`canvas-repair-path__node ${
                          target.id === selectedNodeId ? "canvas-repair-path__node--active" : ""
                        }`}
                        data-testid={`canvas-repair-path-node-${target.id}`}
                        onClick={() => handleRecommendationClick(target.id)}
                      >
                        <span className="canvas-repair-path__name">{target.name}</span>
                        <span className="canvas-repair-path__meta">{target.type}</span>
                      </button>
                      {index < repairPathTargets.length - 1 ? (
                        <span className="canvas-repair-path__arrow" aria-hidden="true">
                          →
                        </span>
                      ) : null}
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        ) : null}

        {focusMode !== "selected" && focusTargets.length > 0 ? (
          <div className="canvas-focus-nav" data-testid="canvas-focus-nav">
            <div className="canvas-focus-nav__header">
              <div className="canvas-focus-nav__title">
                {focusMode === "issues" ? "问题导航" : "最近编辑导航"}
              </div>
              <div className="canvas-focus-nav__actions">
                <button
                  type="button"
                  className="ghost-btn compact-btn"
                  onClick={() => handleCycleFocusTarget(-1)}
                  disabled={focusTargets.length <= 1}
                >
                  上一个
                </button>
                <span className="canvas-focus-nav__index">
                  {activeFocusIndex >= 0 ? activeFocusIndex + 1 : 1} / {focusTargets.length}
                </span>
                <button
                  type="button"
                  className="ghost-btn compact-btn"
                  onClick={() => handleCycleFocusTarget(1)}
                  disabled={focusTargets.length <= 1}
                >
                  下一个
                </button>
              </div>
            </div>

            <div className="canvas-focus-nav__targets" data-testid="canvas-focus-targets">
              {focusTargets.map((target) => (
                <button
                  key={target.id}
                  type="button"
                  className={`canvas-focus-target ${
                    target.id === focusState.activeTargetId ? "canvas-focus-target--active" : ""
                  }`}
                  data-testid={`canvas-focus-target-${target.id}`}
                  onClick={() => setSelectedNode(target.id)}
                >
                  <span className="canvas-focus-target__name">{target.name}</span>
                  <span className="canvas-focus-target__meta">{target.type}</span>
                  {focusMode === "issues" && target.issue ? (
                    <span className="canvas-focus-target__note">{target.issue}</span>
                  ) : null}
                </button>
              ))}
            </div>
          </div>
        ) : null}
      </div>

      {(graph.validation_state?.graph_issues || []).length > 0 ? (
        <div className="validation-banner">{graph.validation_state.graph_issues[0].message}</div>
      ) : null}

      <div className="canvas-lanes">
        {laneSummary.map((lane) => (
          <div
            key={lane.lane}
            className={`canvas-lane-chip${
              selectedNode?.type === lane.lane ? " canvas-lane-chip--active" : ""
            }${lane.issueCount > 0 ? " canvas-lane-chip--warning" : ""}`}
          >
            <span>{CANVAS_LANE_LABELS[lane.lane]}</span>
            <strong>{lane.count}</strong>
            <small>
              {lane.focusCount > 0
                ? `${lane.focusCount} 个聚焦中`
                : lane.issueCount > 0
                  ? `${lane.issueCount} 个问题`
                  : "正常"}
            </small>
          </div>
        ))}
      </div>

      <div className="strategy-canvas">
        <ReactFlowProvider>
          <FlowInner
            focusMode={focusMode}
            focusTargetIds={focusState.targetIds}
            focusAnchorId={focusState.focusAnchorId}
            recommendedNodeIds={recommendationState.recommendedNodeIds}
            repairPathNodeIds={recommendationState.pathNodeIds}
            repairPathEdgeIds={recommendationState.pathEdgeIds}
          />
        </ReactFlowProvider>
      </div>
    </section>
  );
}
