function formatScalar(value) {
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (value === null || value === undefined) return "null";
  return JSON.stringify(value);
}

function formatConfig(config) {
  return Object.entries(config || {})
    .map(([key, value]) => `    ${key}: ${formatScalar(value)}`)
    .join("\n");
}

function normalizeNodeKind(node) {
  if (node.type === "runtime") return "runtime";
  if (node.type === "execution") return "execution";
  return "plugin";
}

function incomingConnections(graph, nodeId) {
  return graph.edges
    .filter((edge) => edge.target_node_id === nodeId)
    .map((edge) => ({
      sourceNode: graph.nodes.find((node) => node.id === edge.source_node_id),
      sourcePort: edge.source_port,
      targetPort: edge.target_port
    }))
    .filter((item) => item.sourceNode);
}

export function generateNodeQuantScript(node, graph) {
  const kind = normalizeNodeKind(node);
  const inputs = incomingConnections(graph, node.id);
  const configBlock = formatConfig(node.config);
  const lines = [
    `${kind} ${node.id} uses ${node.module_key}`,
    `  name: ${JSON.stringify(node.name)}`,
    `  category: ${JSON.stringify(node.type)}`
  ];

  if (configBlock) {
    lines.push("  config:");
    lines.push(configBlock);
  }

  if (inputs.length > 0) {
    lines.push("  inputs:");
    inputs.forEach((input) => {
      lines.push(`    - from: ${input.sourceNode.id}.${input.sourcePort}`);
      lines.push(`      to: ${node.id}.${input.targetPort}`);
    });
  }

  return lines.join("\n");
}

export function generateGraphQuantScript(graph) {
  const lines = [
    `strategy_graph ${graph.metadata.graph_id} {`,
    `  name: ${JSON.stringify(graph.metadata.name)}`,
    `  version: ${JSON.stringify(graph.metadata.version)}`,
    `  mode: ${JSON.stringify(graph.nodes.find((node) => node.type === "runtime")?.config?.mode || "paper")}`,
    "",
    "  nodes:"
  ];

  graph.nodes.forEach((node) => {
    const nodeScript = generateNodeQuantScript(node, graph)
      .split("\n")
      .map((line) => `    ${line}`);
    lines.push(...nodeScript);
    lines.push("");
  });

  lines.push("  graph:");
  if (graph.edges.length === 0) {
    lines.push("    # no connections");
  } else {
    graph.edges.forEach((edge) => {
      lines.push(`    connect ${edge.source_node_id}.${edge.source_port} -> ${edge.target_node_id}.${edge.target_port}`);
    });
  }
  lines.push("}");
  return lines.join("\n").replace(/\n{3,}/g, "\n\n");
}
