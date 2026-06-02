import {
  formalDataBindingName,
  formalDataNodes,
  formalDataRuntimeId,
  formalIntentBindingBase,
  formalIntentRuntimeId,
  formalIntentSignalBindingName,
  generateFormalQuantScript
} from "./quantscriptFormal";
import { generateGraphQuantScript, generateNodeQuantScript } from "./quantscriptGraphSource";

export { generateFormalQuantScript } from "./quantscriptFormal";
export { generateGraphQuantScript, generateNodeQuantScript } from "./quantscriptGraphSource";

function parseScalar(value) {
  const trimmed = value.trim();
  if (trimmed === "true") return true;
  if (trimmed === "false") return false;
  if (trimmed === "null") return null;
  if ((trimmed.startsWith("\"") && trimmed.endsWith("\"")) || (trimmed.startsWith("'") && trimmed.endsWith("'"))) {
    try {
      return JSON.parse(trimmed.replace(/^'/, '"').replace(/'$/, '"'));
    } catch (e) {
      console.warn("quantscript: parseScalar failed", e);
      return trimmed.slice(1, -1);
    }
  }
  if (/^-?\d+(\.\d+)?$/.test(trimmed)) return Number(trimmed);
  return trimmed;
}

function moduleKeyFromKind(kind, nodeType, registry) {
  if (kind === "runtime") return "builtin.runtime.control";
  if (kind === "execution") return "builtin.execution.paper";
  if (nodeType && registry.getByCategory(nodeType).length === 1) {
    return registry.getByCategory(nodeType)[0].module_key;
  }
  return null;
}

function getPortForNode(node, direction, fallbackIndex = 0) {
  const ports = direction === "input" ? node.input_ports || [] : node.output_ports || [];
  return ports[fallbackIndex]?.key || (direction === "input" ? "input" : "output");
}

function cloneJson(value, fallback) {
  if (value === undefined) return fallback;
  try {
    return JSON.parse(JSON.stringify(value));
  } catch (e) {
    console.warn("quantscript: cloneJson failed", e);
    return fallback;
  }
}

function defaultViewport() {
  return { x: 0, y: 0, zoom: 0.8 };
}

function defaultRuntimeState() {
  return {
    status: "idle",
    last_event_type: null,
    last_event_time: null,
    last_message: "",
    metrics: {},
    error: null
  };
}

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

function buildQuantScriptLabelTargets(graph) {
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

// v2.4.0 G7: 此函数仅为前端本地近似, 编译后以 backendCompile.runtime_targets 为准。
// formalNodeIdentifier 的 sanitize 逻辑可能与后端 sanitize_quantscript_runtime_id 略有差异,
// 运行时事件映射统一使用后端返回的 CompileRuntimeTargets。
function buildQuantScriptRuntimeTargets(graph) {
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

function fallbackPosition(index, type) {
  const laneX = {
    runtime: 40,
    data: 120,
    intent: 420,
    agent: 720,
    risk: 1020,
    execution: 1320
  };
  return {
    x: laneX[type] || 120,
    y: type === "runtime" ? 24 + index * 160 : 120 + index * 160
  };
}

function buildPreviousNodeIndexes(previousGraph) {
  const byId = new Map();
  const bySignature = new Map();

  (previousGraph?.nodes || []).forEach((node, index) => {
    const entry = { node, index };
    byId.set(node.id, entry);
    const signature = JSON.stringify([node.module_key, node.type, node.name]);
    const bucket = bySignature.get(signature) || [];
    bucket.push(entry);
    bySignature.set(signature, bucket);
  });

  return { byId, bySignature };
}

function takePreviousNode(nodeData, indexes, usedIds, usedSignatures) {
  const direct = indexes.byId.get(nodeData.id);
  if (direct && !usedIds.has(direct.node.id)) {
    usedIds.add(direct.node.id);
    return direct;
  }

  const signature = JSON.stringify([nodeData.module_key, nodeData.category || nodeData.type, nodeData.name]);
  const bucket = indexes.bySignature.get(signature) || [];
  const next = bucket.find((entry) => !usedSignatures.has(entry.node.id));
  if (!next) return null;

  usedSignatures.add(next.node.id);
  usedIds.add(next.node.id);
  return next;
}

function mergeGraphMetadata(previousGraph, metadata, sourceMode = "quantscript") {
  const previousMetadata = previousGraph?.metadata || {};
  return {
    graph_id: metadata.graph_id,
    name: metadata.name,
    description: previousMetadata.description || metadata.description,
    version: metadata.version,
    created_at: previousMetadata.created_at || metadata.created_at,
    updated_at: metadata.updated_at,
    runtime_binding: cloneJson(previousMetadata.runtime_binding, {
      current_run_id: null,
      last_compile_id: null
    }),
    editor: {
      ...(cloneJson(previousMetadata.editor, {}) || {}),
      viewport: cloneJson(previousMetadata.editor?.viewport, defaultViewport())
    },
    source_mode: sourceMode,
    artifacts: cloneJson(previousMetadata.artifacts, {}) || {}
  };
}

export function attachQuantScriptArtifacts(graph) {
  const nodeScripts = Object.fromEntries(
    graph.nodes.map((node) => [node.id, generateNodeQuantScript(node, graph)])
  );
  const graphScript = generateGraphQuantScript(graph);
  const formalScript = generateFormalQuantScript(graph);
  const labelTargets = buildQuantScriptLabelTargets(graph);
  const runtimeTargets = buildQuantScriptRuntimeTargets(graph);
  return {
    ...graph,
    metadata: {
      ...graph.metadata,
      source_mode: graph.metadata?.source_mode || "graph",
      artifacts: {
        ...(graph.metadata?.artifacts || {}),
        quantscript: {
          graph_source: graphScript,
          formal_source: formalScript,
          node_sources: nodeScripts,
          label_targets: labelTargets,
          runtime_targets: runtimeTargets,
          generated_at: Date.now()
        }
      }
    }
  };
}

export function parseGraphQuantScript(source, registry, previousGraph = null) {
  const lines = source
    .split(/\r?\n/)
    .map((line) => line.replace(/\t/g, "    "))
    .filter((line) => line.trim() !== "" && !line.trim().startsWith("#"));

  if (!lines.length || !lines[0].startsWith("strategy_graph ")) {
    throw new Error("策略图源码必须以 `strategy_graph` 开头。");
  }

  const headerMatch = lines[0].match(/^strategy_graph\s+(\S+)\s+\{$/);
  if (!headerMatch) throw new Error("策略图源码头部格式无效。");

  const graphId = headerMatch[1];
  const metadata = {
    graph_id: graphId,
    name: "Imported Strategy",
    description: "Imported from strategy_graph source",
    version: "1.0.0",
    created_at: Date.now(),
    updated_at: Date.now(),
    runtime_binding: { current_run_id: null, last_compile_id: null },
    editor: { viewport: defaultViewport() },
    source_mode: "graph",
    artifacts: {}
  };

  const nodes = [];
  const edges = [];
  const previousIndexes = buildPreviousNodeIndexes(previousGraph);
  const usedPreviousIds = new Set();
  const usedPreviousSignatures = new Set();
  let index = 1;
  let mode = "paper";

  while (index < lines.length) {
    const line = lines[index].trim();
    if (line === "nodes:") {
      index += 1;
      break;
    }
    if (line.startsWith("name:")) metadata.name = parseScalar(line.slice(5)).toString();
    if (line.startsWith("version:")) metadata.version = parseScalar(line.slice(8)).toString();
    if (line.startsWith("mode:")) mode = parseScalar(line.slice(5)).toString();
    index += 1;
  }

  while (index < lines.length) {
    const raw = lines[index];
    const line = raw.trim();
    if (line === "graph:") {
      index += 1;
      break;
    }
    const nodeMatch = line.match(/^(runtime|execution|plugin)\s+(\S+)\s+uses\s+(\S+)$/);
    if (!nodeMatch) {
      index += 1;
      continue;
    }
    const [, kind, nodeId, explicitModuleKey] = nodeMatch;
    const nodeData = {
      id: nodeId,
      kind,
      module_key: explicitModuleKey,
      type: kind === "plugin" ? null : kind,
      name: nodeId,
      category: null,
      config: {},
      inputs: []
    };
    index += 1;
    while (index < lines.length) {
      const nextRaw = lines[index];
      const nextLine = nextRaw.trim();
      const nextIndent = nextRaw.match(/^\s*/)[0].length;
      if (nextLine === "graph:" || nextLine.match(/^(runtime|execution|plugin)\s+\S+\s+uses\s+\S+$/) || nextIndent <= 2) {
        break;
      }
      if (nextLine.startsWith("name:")) nodeData.name = parseScalar(nextLine.slice(5)).toString();
      else if (nextLine.startsWith("category:")) nodeData.category = parseScalar(nextLine.slice(9)).toString();
      else if (nextLine === "config:") {
        index += 1;
        while (index < lines.length) {
          const configRaw = lines[index];
          const configTrim = configRaw.trim();
          const configIndent = configRaw.match(/^\s*/)[0].length;
          if (!configTrim || configIndent <= 4) break;
          const split = configTrim.split(/:\s+/, 2);
          if (split.length === 2) nodeData.config[split[0]] = parseScalar(split[1]);
          index += 1;
        }
        continue;
      } else if (nextLine === "inputs:") {
        index += 1;
        while (index < lines.length) {
          const inputRaw = lines[index];
          const inputTrim = inputRaw.trim();
          const inputIndent = inputRaw.match(/^\s*/)[0].length;
          if (!inputTrim || inputIndent <= 4) break;
          if (inputTrim.startsWith("- from:")) {
            const fromRef = inputTrim.replace(/^\-\s*from:\s*/, "").trim();
            const toTrim = (lines[index + 1] || "").trim();
            const toRef = toTrim.startsWith("to:") ? toTrim.replace(/^to:\s*/, "").trim() : `${nodeId}.${getPortForNode({ input_ports: [] }, "input")}`;
            nodeData.inputs.push({ fromRef, toRef });
            if (toTrim.startsWith("to:")) index += 1;
          }
          index += 1;
        }
        continue;
      }
      index += 1;
    }

    const nodeType = nodeData.category || nodeData.type || "data";
    const moduleKey = nodeData.module_key || moduleKeyFromKind(kind, nodeType, registry);
    const moduleDef = registry.getByKey(moduleKey) || registry.getByCategory(nodeType)[0];
    if (!moduleDef) throw new Error(`节点 ${nodeId} 使用了未知模块。`);

    const previousEntry = takePreviousNode(nodeData, previousIndexes, usedPreviousIds, usedPreviousSignatures);
    const previousNode = previousEntry?.node;
    const node = {
      id: nodeId,
      type: nodeType,
      module_key: moduleDef.module_key,
      name: nodeData.name,
      position: cloneJson(previousNode?.position, fallbackPosition(previousEntry?.index ?? nodes.length, nodeType)),
      config: {
        ...Object.fromEntries((moduleDef.config_schema?.fields || []).map((field) => [field.key, field.default])),
        ...(cloneJson(previousNode?.config, {}) || {}),
        ...nodeData.config,
        ...(nodeType === "runtime" ? { mode } : {})
      },
      input_ports: moduleDef.ports.inputs || [],
      output_ports: moduleDef.ports.outputs || [],
      ui_state: {
        ...(cloneJson(previousNode?.ui_state, {}) || {}),
        collapsed: Boolean(previousNode?.ui_state?.collapsed)
      },
      runtime_state: cloneJson(previousNode?.runtime_state, defaultRuntimeState())
    };
    nodes.push(node);
  }

  while (index < lines.length) {
    const line = lines[index].trim();
    if (line === "}" || line === "# no connections") {
      index += 1;
      continue;
    }
    const match = line.match(/^connect\s+(\S+)\.(\S+)\s+->\s+(\S+)\.(\S+)$/);
    if (match) {
      const [, sourceNodeId, sourcePort, targetNodeId, targetPort] = match;
      edges.push({
        id: `edge_${sourceNodeId}_${targetNodeId}_${sourcePort}_${targetPort}`,
        source_node_id: sourceNodeId,
        source_port: sourcePort,
        target_node_id: targetNodeId,
        target_port: targetPort,
        edge_type: `${sourceNodeId}-${targetNodeId}`
      });
    }
    index += 1;
  }

  const graph = {
    metadata: mergeGraphMetadata(previousGraph, metadata),
    nodes,
    edges,
    validation_state: cloneJson(previousGraph?.validation_state, {
      is_valid: false,
      is_runnable: false,
      node_issues: {},
      edge_issues: {},
      graph_issues: [],
      issue_counts: { error: 0, warning: 0, info: 0 },
      last_validated_at: null
    }),
    compile_summary: cloneJson(previousGraph?.compile_summary, {
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
    })
  };

  return attachQuantScriptArtifacts(graph);
}
