import {
  formalDataBindingName,
  formalDataNodes,
  formalDataRuntimeId,
  formalIntentBindingBase,
  formalIntentRuntimeId,
  formalIntentSignalBindingName
} from "./quantscriptFormal";

function setLabelTarget(map, label, target) {
  if (!label || map[label]) return;
  map[label] = target;
}

function nodeTarget(node, label = null) {
  return {
    scope: "node",
    node_id: node.id,
    edge_id: null,
    field: null,
    label: label || node.name || node.id
  };
}

function nodeFieldTarget(node, field, label) {
  return {
    scope: "node",
    node_id: node.id,
    edge_id: null,
    field,
    label
  };
}

export function buildQuantScriptLabelTargets(graph) {
  const targets = {};

  graph.nodes.forEach((node) => {
    const baseTarget = nodeTarget(node);
    setLabelTarget(targets, node.id, baseTarget);
    setLabelTarget(targets, node.name, baseTarget);
    setLabelTarget(targets, `${node.id}.name`, nodeFieldTarget(node, "name", `${node.name}.name`));
    setLabelTarget(targets, `${node.name}.name`, nodeFieldTarget(node, "name", `${node.name}.name`));

    Object.keys(node.config || {}).forEach((field) => {
      const label = `${node.name}.${field}`;
      const target = nodeFieldTarget(node, field, label);
      setLabelTarget(targets, field, target);
      setLabelTarget(targets, `${node.id}.${field}`, target);
      setLabelTarget(targets, `${node.name}.${field}`, target);
    });

    setLabelTarget(targets, formalDataBindingName(node), baseTarget);
    setLabelTarget(targets, formalDataRuntimeId(node), baseTarget);
    const formalBase = formalIntentBindingBase(node);
    [formalBase, formalIntentSignalBindingName(node)].forEach((label) =>
      setLabelTarget(targets, label, baseTarget)
    );
  });

  return targets;
}

// Local runtime targets are approximate; backend compile runtime_targets remain authoritative.
export function buildQuantScriptRuntimeTargets(graph) {
  const runtimeNode = (graph.nodes || []).find((node) => node.type === "runtime");
  const executionNode = (graph.nodes || []).find((node) => node.type === "execution");
  const agentNode = (graph.nodes || []).find((node) => node.type === "agent");
  const riskNode = (graph.nodes || []).find((node) => node.type === "risk");
  const sourceToNode = {};

  formalDataNodes(graph).forEach((node) => {
    sourceToNode[formalDataRuntimeId(node)] = node.id;
  });

  (graph.nodes || [])
    .filter((node) => node.type === "intent")
    .forEach((node) => {
      sourceToNode[formalIntentRuntimeId(node)] = node.id;
    });

  if (agentNode) sourceToNode.agent_script_main = agentNode.id;
  if (riskNode) sourceToNode.risk_script_global = riskNode.id;

  return {
    source_to_node: sourceToNode,
    runtime_node_id: runtimeNode?.id || null,
    execution_node_id: executionNode?.id || null
  };
}
