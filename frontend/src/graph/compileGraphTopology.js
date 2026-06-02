export function buildTopology(graph) {
  const indegree = new Map(graph.nodes.map((node) => [node.id, 0]));
  const outgoing = new Map(graph.nodes.map((node) => [node.id, []]));

  graph.edges.forEach((edge) => {
    if (!indegree.has(edge.target_node_id) || !outgoing.has(edge.source_node_id)) return;
    indegree.set(edge.target_node_id, indegree.get(edge.target_node_id) + 1);
    outgoing.get(edge.source_node_id).push(edge.target_node_id);
  });

  const queue = graph.nodes
    .filter((node) => indegree.get(node.id) === 0)
    .map((node) => node.id);
  const order = [];

  while (queue.length > 0) {
    const nodeId = queue.shift();
    order.push(nodeId);
    for (const nextId of outgoing.get(nodeId) || []) {
      indegree.set(nextId, indegree.get(nextId) - 1);
      if (indegree.get(nextId) === 0) queue.push(nextId);
    }
  }

  return {
    topologyOrder: order,
    hasCycle: order.length !== graph.nodes.length
  };
}

export function appendGraphCompileDiagnostics({ graph, topology, errors }) {
  if (topology.hasCycle) {
    errors.push("策略图存在循环依赖，无法编译。");
  }

  if ((graph.validation_state?.issue_counts?.error || 0) > 0) {
    errors.push("策略图校验未通过，无法编译。");
  }
}
