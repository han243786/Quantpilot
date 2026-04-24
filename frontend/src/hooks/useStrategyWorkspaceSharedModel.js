import { useGraphStore } from "../store/graphStore";

export function useStrategyWorkspaceSharedModel() {
  const graph = useGraphStore((state) => state.graph);
  const runtime = useGraphStore((state) => state.runtime);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const selectedEdgeId = useGraphStore((state) => state.selectedEdgeId);
  const selectedCompileDiagnosticTarget = useGraphStore(
    (state) => state.selectedCompileDiagnosticTarget
  );
  const loadGraphById = useGraphStore((state) => state.loadGraphById);

  return {
    graph,
    runtime,
    selectedNodeId,
    selectedEdgeId,
    selectedCompileDiagnosticTarget,
    loadGraphById
  };
}
