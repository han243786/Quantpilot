import { useMemo } from "react";
import { useGraphStore } from "../store/graphStore";
import { strategyIrSourceFromGraph } from "./propertyPanelShared";

export function usePropertyPanelSelectors() {
  const graph = useGraphStore((state) => state.graph);
  const registry = useGraphStore((state) => state.registry);
  const compileResult = useGraphStore((state) => state.compileResult);
  const runtime = useGraphStore((state) => state.runtime);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const selectedEdgeId = useGraphStore((state) => state.selectedEdgeId);
  const selectedCompileDiagnosticTarget = useGraphStore(
    (state) => state.selectedCompileDiagnosticTarget
  );
  const quantScriptDraft = useGraphStore((state) => state.quantScriptDraft);
  const formalQuantScriptDraft = useGraphStore((state) => state.formalQuantScriptDraft);
  const formalQuantScriptOverride = useGraphStore((state) => state.formalQuantScriptOverride);
  const strategyIrDraft = useGraphStore((state) => state.strategyIrDraft);

  const selectedNode = useMemo(
    () => graph.nodes.find((node) => node.id === selectedNodeId) || null,
    [graph.nodes, selectedNodeId]
  );
  const selectedEdge = useMemo(
    () => graph.edges.find((edge) => edge.id === selectedEdgeId) || null,
    [graph.edges, selectedEdgeId]
  );

  const compileSummary = graph.compile_summary || { compilable: false, errors: [], warnings: [] };
  const graphSource = quantScriptDraft || graph.metadata?.artifacts?.quantscript?.graph_source || "";
  const graphFormalSource = graph.metadata?.artifacts?.quantscript?.formal_source || "";
  const formalQuantScriptSource = formalQuantScriptDraft ?? graphFormalSource;
  const strategyIrSource = strategyIrDraft || strategyIrSourceFromGraph(graph);
  const successfulAuthoringView =
    compileResult?.backend_compile?.artifacts?.strategy?.metadata?.quantscript_authoring_view || null;
  const partialAuthoringView =
    compileResult?.backend_compile_error?.partial_artifacts?.quantscript_authoring_view || null;
  const authoringView = successfulAuthoringView || partialAuthoringView || null;
  const authoringViewState = successfulAuthoringView
    ? {
        mode: "compiled",
        error: null
      }
    : partialAuthoringView
      ? {
          mode: "partial",
          error: compileResult?.backend_compile_error || null
        }
      : {
          mode: "missing",
          error: null
        };
  const nodeSource = selectedNode
    ? graph.metadata?.artifacts?.quantscript?.node_sources?.[selectedNode.id] || ""
    : "";
  const moduleDef = selectedNode ? registry.getByKey(selectedNode.module_key) : null;
  const nodeIssues = selectedNode ? graph.validation_state?.node_issues?.[selectedNode.id] || [] : [];
  const nodeMetrics = selectedNode ? Object.entries(selectedNode.runtime_state?.metrics || {}) : [];
  const sourceNode = selectedEdge
    ? graph.nodes.find((node) => node.id === selectedEdge.source_node_id) || null
    : null;
  const targetNode = selectedEdge
    ? graph.nodes.find((node) => node.id === selectedEdge.target_node_id) || null
    : null;

  return {
    graph,
    registry,
    compileResult,
    runtime,
    authoringView,
    authoringViewState,
    formalQuantScriptSource,
    formalQuantScriptOverrideActive: formalQuantScriptOverride !== null,
    selectedNodeId,
    selectedEdgeId,
    selectedCompileDiagnosticTarget,
    selectedNode,
    selectedEdge,
    sourceNode,
    targetNode,
    moduleDef,
    compileSummary,
    graphSource,
    strategyIrSource,
    nodeSource,
    nodeIssues,
    nodeMetrics
  };
}
