import { createModuleRegistry } from "../modules/moduleRegistry";
import {
  DEFAULT_CAPABILITIES,
  applyCapabilitiesToModules,
  createSafeFallbackCapabilities
} from "../modules/builtinModules";
import { createEmptyGraph } from "../graph/createGraph";
import { validateGraph } from "../graph/validation";
import { attachQuantScriptArtifacts } from "../graph/quantscript";
import { buildActionFailureMessage } from "../utils/actionFailure";
import { humanizeErrorText, sanitizeDisplayText } from "../utils/errorText";
import { fetchWithTimeout } from "../utils/api";

const STORAGE_KEY = "quantpilot_frontend_graph";
const CAPABILITY_CACHE_KEY = "quantpilot_capabilities_cache";
const defaultModules = applyCapabilitiesToModules(DEFAULT_CAPABILITIES);
const defaultRegistry = createModuleRegistry(defaultModules, DEFAULT_CAPABILITIES);

function resolveApiBase() {
  const explicitBase = import.meta.env.VITE_API_BASE_URL?.trim();
  if (explicitBase) {
    return explicitBase.replace(/\/+$/, "");
  }

  if (typeof window === "undefined") {
    return "http://127.0.0.1:3000/api";
  }

  return "/api";
}

const API_BASE = resolveApiBase();
const DEFAULT_LOCAL_ACTOR = {
  actor_id: "local_operator",
  display_name: "Local operator"
};

function saveGraphToStorage(graph) {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(graph));
}

function loadGraphFromStorage() {
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

function hasUsableGraphShape(graph) {
  return Boolean(
    graph &&
    graph.metadata &&
    typeof graph.metadata === "object" &&
    Array.isArray(graph.nodes) &&
    Array.isArray(graph.edges) &&
    graph.nodes.length > 0
  );
}

function isDeprecatedBuiltinSampleGraph(graph) {
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

function withRecentNodeIds(graph, recentNodeIds = []) {
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

function recordRecentNodeIds(graph, nodeIds = []) {
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

function normalizeGraphShape(graph) {
  if (!graph || typeof graph !== "object") {
    return createEmptyGraph(defaultRegistry);
  }

  const normalizedNodes = Array.isArray(graph.nodes)
    ? graph.nodes.map((node) => {
        // Fallback: QS-generated nodes store module_key in data.subtitle
        const moduleKey = node.module_key || node.data?.subtitle;
        const moduleDef = defaultRegistry.getByKey(moduleKey);
        // Fallback: QS-generated nodes store ports in data.inputPorts / data.outputPorts
        const inputPorts = node.input_ports || node.data?.inputPorts;
        const outputPorts = node.output_ports || node.data?.outputPorts;
        // Fallback: QS-generated nodes store config in data.config
        const config = node.config || node.data?.config;
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


export async function fetchJson(path) {
  const response = await fetchWithTimeout(`${API_BASE}${path}`);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(humanizeErrorText(text, `Request failed with status ${response.status}.`));
  }
  return response.json();
}

async function postJson(path, body) {
  const response = await fetchWithTimeout(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });

  if (!response.ok) {
    const text = await response.text();
    let payload = null;
    try {
      payload = JSON.parse(text);
    } catch {
    }

    const error = new Error(
      humanizeErrorText(
        payload?.message || text,
        `Request failed with status ${response.status}.`
      )
    );
    error.status = response.status;
    error.error = payload?.error || null;
    error.details = Array.isArray(payload?.details) ? payload.details : [];
    error.partial_artifacts = payload?.partial_artifacts || null;
    throw error;
  }

  return response.json();
}

async function deleteJson(path) {
  const response = await fetchWithTimeout(`${API_BASE}${path}`, {
    method: "DELETE"
  });

  if (!response.ok) {
    const text = await response.text();
    let payload = null;
    try {
      payload = JSON.parse(text);
    } catch {
    }

    const error = new Error(
      humanizeErrorText(
        payload?.message || text,
        `Request failed with status ${response.status}.`
      )
    );
    error.status = response.status;
    error.error = payload?.error || null;
    error.details = Array.isArray(payload?.details) ? payload.details : [];
    throw error;
  }

  return response.json();
}

function buildRegistryFromCapabilities(capabilities) {
  return createModuleRegistry(
    applyCapabilitiesToModules(capabilities),
    capabilities || DEFAULT_CAPABILITIES
  );
}

function attachValidation(graph) {
  return attachValidationWithRegistry(graph, defaultRegistry);
}

function attachValidationWithRegistry(graph, registry) {
  const normalized = attachQuantScriptArtifacts(normalizeGraphShape(graph));
  const validation = validateGraph(normalized, registry);
  return { ...normalized, validation_state: validation };
}

function fallbackRunnableGraph(registry = defaultRegistry) {
  return attachValidationWithRegistry(createEmptyGraph(registry), registry);
}

async function resolveGraphForDetail(graphId, fallbackGraph, registry = defaultRegistry) {
  if (!graphId || fallbackGraph?.metadata?.graph_id === graphId) {
    return fallbackGraph;
  }

  try {
    const loaded = resolveLoadedGraphWithRegistry(await fetchJson(`/graphs/${graphId}`), registry);
    return loaded || fallbackGraph;
  } catch {
    return fallbackGraph;
  }
}

function saveCapabilitiesToCache(capabilities) {
  window.localStorage.setItem(CAPABILITY_CACHE_KEY, JSON.stringify(capabilities));
}

function loadCapabilitiesFromCache() {
  const raw = window.localStorage.getItem(CAPABILITY_CACHE_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : null;
  } catch {
    return null;
  }
}

function resolveLoadedGraph(graph) {
  return resolveLoadedGraphWithRegistry(graph, defaultRegistry);
}

function resolveLoadedGraphWithRegistry(graph, registry) {
  if (!hasUsableGraphShape(graph)) return null;
  return attachValidationWithRegistry(
    isDeprecatedBuiltinSampleGraph(graph) ? createEmptyGraph(registry) : graph,
    registry
  );
}

function normalizeGraphIndex(entries) {
  if (!Array.isArray(entries)) return [];

  return entries
    .map((entry) => ({
      graph_id: sanitizeText(entry?.graph_id, ""),
      name: sanitizeText(entry?.name, ""),
      updated_at: typeof entry?.updated_at === "number" ? entry.updated_at : 0,
      path: sanitizeText(entry?.path, "")
    }))
    .filter((entry) => entry.graph_id);
}

function normalizeActorIdentity(actor, fallback = DEFAULT_LOCAL_ACTOR) {
  const actorId = sanitizeText(actor?.actor_id, fallback.actor_id);
  const displayName = sanitizeText(actor?.display_name, fallback.display_name || actorId);
  return {
    actor_id: actorId || fallback.actor_id,
    display_name: displayName || fallback.display_name
  };
}

function normalizeCollaborationMetadata(collaboration) {
  return {
    owner:
      collaboration?.owner && typeof collaboration.owner === "object"
        ? normalizeActorIdentity(collaboration.owner)
        : null,
    editors: Array.isArray(collaboration?.editors)
      ? collaboration.editors.map((actor) => normalizeActorIdentity(actor)).filter((actor) => actor.actor_id)
      : [],
    last_saved_by:
      collaboration?.last_saved_by && typeof collaboration.last_saved_by === "object"
        ? normalizeActorIdentity(collaboration.last_saved_by)
        : null,
    last_run_actor:
      collaboration?.last_run_actor && typeof collaboration.last_run_actor === "object"
        ? normalizeActorIdentity(collaboration.last_run_actor)
        : null
  };
}

function resolveGraphActor(graph) {
  const collaboration = normalizeCollaborationMetadata(graph?.metadata?.collaboration);
  return collaboration.owner || collaboration.editors[0] || DEFAULT_LOCAL_ACTOR;
}

function withGraphActorMetadata(graph, actor = resolveGraphActor(graph)) {
  const collaboration = normalizeCollaborationMetadata(graph?.metadata?.collaboration);
  if (!collaboration.owner) {
    collaboration.owner = normalizeActorIdentity(actor);
  }
  collaboration.last_saved_by = normalizeActorIdentity(actor);
  return {
    ...graph,
    metadata: {
      ...(graph?.metadata || {}),
      collaboration
    }
  };
}

function normalizeGraphAuditHistory(entries) {
  if (!Array.isArray(entries)) return [];
  return entries
    .map((entry) => ({
      audit_id: sanitizeText(entry?.audit_id, ""),
      graph_id: sanitizeText(entry?.graph_id, ""),
      action: sanitizeText(entry?.action, ""),
      created_at_ms: typeof entry?.created_at_ms === "number" ? entry.created_at_ms : 0,
      actor: normalizeActorIdentity(entry?.actor, DEFAULT_LOCAL_ACTOR),
      target_id: sanitizeText(entry?.target_id, ""),
      summary: sanitizeText(entry?.summary, "")
    }))
    .filter((entry) => entry.audit_id && entry.graph_id);
}

function normalizeGraphVersions(entries) {
  if (!Array.isArray(entries)) return [];

  return entries
    .map((entry) => ({
      graph_id: sanitizeText(entry?.graph_id, ""),
      version_id: sanitizeText(entry?.version_id, ""),
      name: sanitizeText(entry?.name, ""),
      updated_at: typeof entry?.updated_at === "number" ? entry.updated_at : 0,
      version_label: sanitizeText(entry?.version_label, ""),
      save_note: sanitizeText(entry?.save_note, ""),
      node_count: typeof entry?.node_count === "number" ? entry.node_count : 0,
      edge_count: typeof entry?.edge_count === "number" ? entry.edge_count : 0,
      path: sanitizeText(entry?.path, ""),
      quantscript_path: sanitizeText(entry?.quantscript_path, ""),
      is_latest: Boolean(entry?.is_latest)
    }))
    .filter((entry) => entry.graph_id && entry.version_id);
}

function normalizeGraphVersionCompare(compare) {
  if (!compare || typeof compare !== "object") return null;

  const normalizeEntry = (entry) => ({
    graph_id: sanitizeText(entry?.graph_id, ""),
    version_id: sanitizeText(entry?.version_id, ""),
    name: sanitizeText(entry?.name, ""),
    updated_at: typeof entry?.updated_at === "number" ? entry.updated_at : 0,
    version_label: sanitizeText(entry?.version_label, ""),
    save_note: sanitizeText(entry?.save_note, ""),
    node_count: typeof entry?.node_count === "number" ? entry.node_count : 0,
    edge_count: typeof entry?.edge_count === "number" ? entry.edge_count : 0,
    is_latest: Boolean(entry?.is_latest)
  });

  const normalizeDiffRow = (row) => ({
    key: sanitizeText(row?.key, ""),
    label: sanitizeText(row?.label, ""),
    status: sanitizeText(row?.status, "same"),
    left_value: sanitizeText(row?.left_value, ""),
    right_value: sanitizeText(row?.right_value, "")
  });

  const normalizeCollectionDiff = (diff) => ({
    left_count: typeof diff?.left_count === "number" ? diff.left_count : 0,
    right_count: typeof diff?.right_count === "number" ? diff.right_count : 0,
    added_ids: Array.isArray(diff?.added_ids) ? diff.added_ids.map((value) => sanitizeText(value, "")).filter(Boolean) : [],
    removed_ids: Array.isArray(diff?.removed_ids)
      ? diff.removed_ids.map((value) => sanitizeText(value, "")).filter(Boolean)
      : [],
    changed_ids: Array.isArray(diff?.changed_ids)
      ? diff.changed_ids.map((value) => sanitizeText(value, "")).filter(Boolean)
      : []
  });

  return {
    graph_id: sanitizeText(compare?.graph_id, ""),
    left: normalizeEntry(compare?.left),
    right: normalizeEntry(compare?.right),
    metadata_rows: Array.isArray(compare?.metadata_rows)
      ? compare.metadata_rows.map(normalizeDiffRow)
      : [],
    node_diff: normalizeCollectionDiff(compare?.node_diff),
    edge_diff: normalizeCollectionDiff(compare?.edge_diff),
    config_diffs: Array.isArray(compare?.config_diffs)
      ? compare.config_diffs.map((row) => ({
          node_id: sanitizeText(row?.node_id, ""),
          node_name: sanitizeText(row?.node_name, ""),
          field_path: sanitizeText(row?.field_path, ""),
          status: sanitizeText(row?.status, "same"),
          left_value: sanitizeText(row?.left_value, ""),
          right_value: sanitizeText(row?.right_value, "")
        }))
      : [],
    has_changes: Boolean(compare?.has_changes)
  };
}

function graphExistsInIndex(graph, graphIndex = []) {
  const graphId = graph?.metadata?.graph_id;
  if (!graphId) return false;
  return graphIndex.some((entry) => entry.graph_id === graphId);
}


export {
  API_BASE,
  CAPABILITY_CACHE_KEY,
  STORAGE_KEY,
  createSafeFallbackCapabilities,
  deleteJson,
  defaultModules,
  DEFAULT_CAPABILITIES as defaultCapabilities,
  defaultRegistry,
  fallbackRunnableGraph,
  graphExistsInIndex,
  hasUsableGraphShape,
  isDeprecatedBuiltinSampleGraph,
  loadCapabilitiesFromCache,
  loadGraphFromStorage,
  normalizeGraphIndex,
  normalizeGraphAuditHistory,
  normalizeGraphVersionCompare,
  normalizeGraphVersions,
  normalizeGraphShape,
  postJson,
  recordRecentNodeIds,
  resolveGraphActor,
  resolveGraphForDetail,
  resolveLoadedGraph,
  resolveLoadedGraphWithRegistry,
  saveCapabilitiesToCache,
  saveGraphToStorage,
  withGraphActorMetadata,
  withRecentNodeIds,
  attachValidation,
  attachValidationWithRegistry,
  buildRegistryFromCapabilities
};
