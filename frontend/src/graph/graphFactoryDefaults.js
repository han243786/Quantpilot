export function createGraphEdge(sourceNode, sourcePort, targetNode, targetPort) {
  return {
    id: `edge_${sourceNode.id}_${targetNode.id}_${sourcePort}_${targetPort}`,
    source_node_id: sourceNode.id,
    source_port: sourcePort,
    target_node_id: targetNode.id,
    target_port: targetPort,
    edge_type: `${sourceNode.type}_to_${targetNode.type}`
  };
}

export function createInitialValidationState() {
  return {
    is_valid: false,
    is_runnable: false,
    node_issues: {},
    edge_issues: {},
    graph_issues: [],
    issue_counts: { error: 0, warning: 0, info: 0 },
    last_validated_at: null
  };
}

export function createInitialCompileSummary() {
  return {
    compilable: false,
    last_compile_id: null,
    last_compile_at: null,
    topology_order: [],
    outputs: {
      data_sources: 0,
      intent_generators: 0,
      agents: 0,
      risk_controls: 0,
      executions: 0
    },
    warnings: [],
    errors: []
  };
}
