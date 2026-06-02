export function buildGraphEdgeIndex(edges = []) {
  const edgesByTarget = new Map();
  const edgesBySource = new Map();

  edges.forEach((edge) => {
    if (!edgesByTarget.has(edge.target_node_id)) edgesByTarget.set(edge.target_node_id, []);
    edgesByTarget.get(edge.target_node_id).push(edge);

    if (!edgesBySource.has(edge.source_node_id)) edgesBySource.set(edge.source_node_id, []);
    edgesBySource.get(edge.source_node_id).push(edge);
  });

  return { edgesByTarget, edgesBySource };
}

export function resolveNodeEdges(edgeIndex, nodeId) {
  return {
    incoming: edgeIndex.edgesByTarget.get(nodeId) || [],
    outgoing: edgeIndex.edgesBySource.get(nodeId) || []
  };
}

export function summarizeGraphNodeTypes(nodes = []) {
  return nodes.reduce(
    (summary, node) => {
      if (node.type === "runtime") summary.runtimeCount += 1;
      if (node.type === "execution") {
        summary.hasExecution = true;
        summary.executionCount += 1;
      }
      if (node.type === "risk") summary.hasRisk = true;
      if (node.type === "agent") summary.hasAgent = true;
      if (node.type === "intent") summary.hasIntent = true;
      if (node.type === "data") summary.hasData = true;
      return summary;
    },
    {
      runtimeCount: 0,
      hasExecution: false,
      hasRisk: false,
      hasAgent: false,
      hasIntent: false,
      hasData: false,
      executionCount: 0
    }
  );
}
