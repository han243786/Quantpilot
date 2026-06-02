import { attachQuantScriptArtifacts, generateGraphQuantScript } from "./quantscript";
import { buildCoreIr } from "./compileGraphCoreIr";
import { buildRuntimeConfig } from "./compileGraphRuntimeConfig";
import { buildLocalCompileDiagnostics } from "./compileGraphSupport";

function buildTopology(graph) {
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

export function compileGraph(graph, registry = null) {
  const { compileId, output, errors, warnings } = buildRuntimeConfig(graph, registry);
  const topology = buildTopology(graph);
  if (topology.hasCycle) {
    errors.push("策略图存在循环依赖，无法编译。");
  }

  if ((graph.validation_state?.issue_counts?.error || 0) > 0) {
    errors.push("策略图校验未通过，无法编译。");
  }

  const graphWithArtifacts = attachQuantScriptArtifacts(graph);
  const coreIr = buildCoreIr(graph, output);
  graphWithArtifacts.metadata.artifacts = {
    ...(graphWithArtifacts.metadata.artifacts || {}),
    core_ir: coreIr
  };
  const quantscript =
    graphWithArtifacts.metadata.artifacts.quantscript.graph_source ||
    generateGraphQuantScript(graph);
  const compilable = errors.length === 0;
  const diagnostics = buildLocalCompileDiagnostics(errors, warnings);

  return {
    compile_id: compileId,
    runtime_config: output,
    core_ir: coreIr,
    quantscript,
    graph: graphWithArtifacts,
    compile_summary: {
      compilable,
      last_compile_id: compileId,
      last_compile_at: Date.now(),
      topology_order: topology.topologyOrder,
      outputs: {
        data_sources: output.data_sources.length,
        intent_generators: output.intent_generators.length,
        agents: output.agents.length,
        risk_controls: output.risk_controls.length,
        executions: output.executions.length
      },
      diagnostics,
      warnings,
      errors
    }
  };
}
