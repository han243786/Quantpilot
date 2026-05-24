import { useGraphStore } from "../store/graphStore";

export function useStrategyWorkspaceSharedModel() {
  const graph = useGraphStore((state) => state.graph);
  const runtime = useGraphStore((state) => state.runtime);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const selectedEdgeId = useGraphStore((state) => state.selectedEdgeId);
  const selectedCompileDiagnosticTarget = useGraphStore(
    (state) => state.selectedCompileDiagnosticTarget
  );
  const capabilities = useGraphStore((state) => state.capabilities);
  const capabilityStatus = useGraphStore((state) => state.capabilityStatus);
  const capabilitySource = useGraphStore((state) => state.capabilitySource);
  const capabilityMessage = useGraphStore((state) => state.capabilityMessage);
  const loadGraphById = useGraphStore((state) => state.loadGraphById);

  return {
    graph,
    runtime,
    selectedNodeId,
    selectedEdgeId,
    selectedCompileDiagnosticTarget,
    capabilities,
    capabilityStatus,
    capabilitySource,
    capabilityMessage,
    loadGraphById
  };
}
