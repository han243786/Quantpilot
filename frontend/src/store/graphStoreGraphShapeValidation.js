import { createModuleRegistry } from "../modules/moduleRegistry";
import {
  DEFAULT_CAPABILITIES,
  applyCapabilitiesToModules,
  createSafeFallbackCapabilities
} from "../modules/builtinModules";
import { createEmptyGraph } from "../graph/createGraph";
import { validateGraph } from "../graph/validation";
import { attachQuantScriptArtifacts } from "../graph/quantscript";
import { sanitizeDisplayText } from "../utils/errorText";
import { normalizeCollaborationMetadata } from "./graphStoreActorCollaboration";

export {
  createSafeFallbackCapabilities,
  DEFAULT_CAPABILITIES as defaultCapabilities
};

const defaultModules = applyCapabilitiesToModules(DEFAULT_CAPABILITIES);
export const defaultRegistry = createModuleRegistry(defaultModules, DEFAULT_CAPABILITIES);

export function hasUsableGraphShape(graph) {
  return Boolean(
    graph &&
    graph.metadata &&
    typeof graph.metadata === "object" &&
    Array.isArray(graph.nodes) &&
    Array.isArray(graph.edges) &&
    graph.nodes.length > 0
  );
}

export function isDeprecatedBuiltinSampleGraph(graph) {
  if (!graph || !Array.isArray(graph.nodes)) return false;
  const isLegacyTrendSample =
    graph.metadata?.graph_id === "dual_ma_trend_paper_v1" &&
    graph.nodes.some(
      (node) =>
        node.module_key === "builtin.data.kline" && node.config?.exchange === "binance"
    );
  return graph.metadata?.name === "示例策略图" || isLegacyTrendSample;
}

function sanitizeText(value, fallback) {
  return sanitizeDisplayText(value, fallback);
}

function normalizeRecentNodeIds(editor, nodes = []) {
  const validNodeIds = new Set(nodes.map((node) => node.id));
  const recentNodeIds = Array.isArray(editor?.recent_node_ids) ? editor.recent_node_ids : [];

  return recentNodeIds.filter((nodeId) => validNodeIds.has(nodeId)).slice(0, 8);
}

export function withRecentNodeIds(graph, recentNodeIds = []) {
  return {
    ...graph,
    metadata: {
      ...(graph.metadata || {}),
      ...graph.metadata,
      editor: {
        ...(graph.metadata?.editor || {}),
        recent_node_ids: normalizeRecentNodeIds(
          {
            ...(graph.metadata?.editor || {}),
            recent_node_ids: recentNodeIds
          },
          graph.nodes || []
        )
      }
    }
  };
}

export function recordRecentNodeIds(graph, nodeIds = []) {
  const nextIds = nodeIds.filter(Boolean);
  if (nextIds.length === 0) return withRecentNodeIds(graph, graph.metadata?.editor?.recent_node_ids);

  const currentIds = Array.isArray(graph.metadata?.editor?.recent_node_ids)
    ? graph.metadata.editor.recent_node_ids
    : [];

  return withRecentNodeIds(graph, [...nextIds, ...currentIds]);
}

function normalizeEdges(rawEdges) {
  if (!Array.isArray(rawEdges)) return [];
  return rawEdges.map((edge) => ({
    ...edge,
    source_node_id: edge.source_node_id || edge.source,
    target_node_id: edge.target_node_id || edge.target,
    source_port: edge.source_port || edge.sourceHandle,
    target_port: edge.target_port || edge.targetHandle,
    edge_type: edge.edge_type || `${edge.source_node_id || edge.source}-${edge.target_node_id || edge.target}`
  }));
}

export function normalizeGraphShape(graph) {
  if (!graph || typeof graph !== "object") {
    return createEmptyGraph(defaultRegistry);
  }

  const normalizedNodes = Array.isArray(graph.nodes)
    ? graph.nodes.map((node) => {
        // v1.0.1: 仅支持新格式, 旧 QS 兼容 fallback 已移除
        const moduleKey = node.module_key;
        const moduleDef = defaultRegistry.getByKey(moduleKey);
        const inputPorts = node.input_ports;
        const outputPorts = node.output_ports;
        const config = node.config;
        return {
          ...node,
          name: sanitizeText(node.name, moduleDef?.node?.default_name || node.id || "节点"),
          module_key: moduleKey || node.module_key,
          config: { ...config } || {},
          input_ports: Array.isArray(inputPorts) ? inputPorts : moduleDef?.ports?.inputs || [],
          output_ports: Array.isArray(outputPorts) ? outputPorts : moduleDef?.ports?.outputs || [],
          ui_state: {
            collapsed: Boolean(node.ui_state?.collapsed)
          },
          runtime_state: {
            status: node.runtime_state?.status || "idle",
            last_event_type: node.runtime_state?.last_event_type || null,
            last_event_time: node.runtime_state?.last_event_time || null,
            last_message: sanitizeText(node.runtime_state?.last_message, ""),
            metrics: node.runtime_state?.metrics || {},
            error: sanitizeText(node.runtime_state?.error, null)
          }
        };
      })
    : [];

  return {
    metadata: {
      graph_id: sanitizeText(graph.metadata?.graph_id, "draft_graph"),
      name: sanitizeText(graph.metadata?.name, "未命名策略图"),
      description: sanitizeText(graph.metadata?.description, ""),
      version: graph.metadata?.version || "1.0.0",
      created_at: graph.metadata?.created_at || Date.now(),
      updated_at: graph.metadata?.updated_at || Date.now(),
      template_id: sanitizeText(graph.metadata?.template_id, ""),
      template_label: sanitizeText(graph.metadata?.template_label, ""),
      version_label: sanitizeText(graph.metadata?.version_label, ""),
      save_note: sanitizeText(graph.metadata?.save_note, ""),
      runtime_binding: {
        current_run_id: graph.metadata?.runtime_binding?.current_run_id || null,
        last_compile_id: graph.metadata?.runtime_binding?.last_compile_id || null
      },
      collaboration: normalizeCollaborationMetadata(graph.metadata?.collaboration),
      editor: {
        viewport: graph.metadata?.editor?.viewport || { x: 0, y: 0, zoom: 0.8 },
        recent_node_ids: normalizeRecentNodeIds(graph.metadata?.editor, normalizedNodes)
      },
      source_mode: graph.metadata?.source_mode || "graph",
      artifacts: graph.metadata?.artifacts || {}
    },
    nodes: normalizedNodes,
    edges: normalizeEdges(graph.edges),
    validation_state: graph.validation_state || {},
    compile_summary: graph.compile_summary || {}
  };
}

export function buildRegistryFromCapabilities(capabilities) {
  return createModuleRegistry(
    applyCapabilitiesToModules(capabilities),
    capabilities || DEFAULT_CAPABILITIES
  );
}

export function attachValidationWithRegistry(graph, registry) {
  const normalized = attachQuantScriptArtifacts(normalizeGraphShape(graph));
  const validation = validateGraph(normalized, registry);
  return { ...normalized, validation_state: validation };
}

export function fallbackRunnableGraph(registry = defaultRegistry) {
  return attachValidationWithRegistry(createEmptyGraph(registry), registry);
}

export function resolveLoadedGraph(graph) {
  return resolveLoadedGraphWithRegistry(graph, defaultRegistry);
}

export function resolveLoadedGraphWithRegistry(graph, registry) {
  if (!hasUsableGraphShape(graph)) return null;
  return attachValidationWithRegistry(
    isDeprecatedBuiltinSampleGraph(graph) ? createEmptyGraph(registry) : graph,
    registry
  );
}
