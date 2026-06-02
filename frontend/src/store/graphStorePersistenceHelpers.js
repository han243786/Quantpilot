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
import { sanitizeDisplayText } from "../utils/errorText";
import {
  DEFAULT_LOCAL_ACTOR,
  normalizeActorIdentity,
  normalizeCollaborationMetadata
} from "./graphStoreActorCollaboration";
import { fetchJson } from "./graphStorePersistenceTransport";

export {
  API_BASE,
  deleteJson,
  fetchJson,
  postJson,
  unwrapPage
} from "./graphStorePersistenceTransport";

export {
  CAPABILITY_CACHE_KEY,
  STORAGE_KEY,
  loadCapabilitiesFromCache,
  loadGraphFromStorage,
  saveCapabilitiesToCache,
  saveGraphToStorage
} from "./graphStorePersistenceStorage";

export {
  resolveGraphActor,
  withGraphActorMetadata
} from "./graphStoreActorCollaboration";

const defaultModules = applyCapabilitiesToModules(DEFAULT_CAPABILITIES);
const defaultRegistry = createModuleRegistry(defaultModules, DEFAULT_CAPABILITIES);

// v1.0.5: API_BASE 来自 src/api/client.js (统一来源)
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
  } catch (e) {
    console.warn("graphStorePersistenceHelpers: resolveGraphForDetail failed", e);
    return fallbackGraph;
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

  const normalizeStrategyConfigDiff = (diff) => {
    if (!diff || typeof diff !== "object") return null;
    return {
      schema_version: sanitizeText(diff?.schema_version, ""),
      left_artifact_id: sanitizeText(diff?.left_artifact_id, ""),
      right_artifact_id: sanitizeText(diff?.right_artifact_id, ""),
      source_digest_changes: Array.isArray(diff?.source_digest_changes)
        ? diff.source_digest_changes.map((change) => ({
            field: sanitizeText(change?.field, ""),
            before: sanitizeText(change?.before, ""),
            after: sanitizeText(change?.after, "")
          }))
        : [],
      domain_changes: Array.isArray(diff?.domain_changes)
        ? diff.domain_changes.map((change) => ({
            domain_id: sanitizeText(change?.domain_id, ""),
            lifecycle_changed: Boolean(change?.lifecycle_changed),
            readiness_changed: Boolean(change?.readiness_changed),
            source_refs_changed: Boolean(change?.source_refs_changed),
            findings_changed: Boolean(change?.findings_changed)
          }))
        : [],
      runtime_boundary_changed: Boolean(diff?.runtime_boundary_changed),
      changed: Boolean(diff?.changed)
    };
  };

  const normalizeEvidenceCountChange = (change) => ({
    key: sanitizeText(change?.key, ""),
    left_count: typeof change?.left_count === "number" ? change.left_count : 0,
    right_count: typeof change?.right_count === "number" ? change.right_count : 0
  });

  const normalizeEvidenceFirstDivergence = (divergence) => {
    if (!divergence || typeof divergence !== "object") return null;
    return {
      index: typeof divergence?.index === "number" ? divergence.index : 0,
      left: sanitizeText(divergence?.left, ""),
      right: sanitizeText(divergence?.right, "")
    };
  };

  const normalizeStrategyConfigEvidenceDiff = (diff) => {
    if (!diff || typeof diff !== "object") return null;
    return {
      schema_version: sanitizeText(diff?.schema_version, ""),
      left_backtest_id: sanitizeText(diff?.left_backtest_id, ""),
      right_backtest_id: sanitizeText(diff?.right_backtest_id, ""),
      status: sanitizeText(diff?.status, "missing"),
      changed: Boolean(diff?.changed),
      diagnostics: Array.isArray(diff?.diagnostics)
        ? diff.diagnostics.map((finding) => ({
            severity: sanitizeText(finding?.severity, "info"),
            code: sanitizeText(finding?.code, ""),
            message: sanitizeText(finding?.message, "")
          }))
        : [],
      machine_trajectory: diff?.machine_trajectory
        ? {
            status: sanitizeText(diff.machine_trajectory?.status, "missing"),
            left_point_count: typeof diff.machine_trajectory?.left_point_count === "number" ? diff.machine_trajectory.left_point_count : 0,
            right_point_count: typeof diff.machine_trajectory?.right_point_count === "number" ? diff.machine_trajectory.right_point_count : 0,
            left_visited_states: Array.isArray(diff.machine_trajectory?.left_visited_states)
              ? diff.machine_trajectory.left_visited_states.map((value) => sanitizeText(value, "")).filter(Boolean)
              : [],
            right_visited_states: Array.isArray(diff.machine_trajectory?.right_visited_states)
              ? diff.machine_trajectory.right_visited_states.map((value) => sanitizeText(value, "")).filter(Boolean)
              : [],
            transition_hit_changes: Array.isArray(diff.machine_trajectory?.transition_hit_changes)
              ? diff.machine_trajectory.transition_hit_changes.map(normalizeEvidenceCountChange)
              : [],
            left_terminal_state: sanitizeText(diff.machine_trajectory?.left_terminal_state, ""),
            right_terminal_state: sanitizeText(diff.machine_trajectory?.right_terminal_state, ""),
            first_divergence: normalizeEvidenceFirstDivergence(diff.machine_trajectory?.first_divergence)
          }
        : null,
      risk_plane: diff?.risk_plane
        ? {
            status: sanitizeText(diff.risk_plane?.status, "missing"),
            left_decision_count: typeof diff.risk_plane?.left_decision_count === "number" ? diff.risk_plane.left_decision_count : 0,
            right_decision_count: typeof diff.risk_plane?.right_decision_count === "number" ? diff.risk_plane.right_decision_count : 0,
            left_approved_count: typeof diff.risk_plane?.left_approved_count === "number" ? diff.risk_plane.left_approved_count : 0,
            right_approved_count: typeof diff.risk_plane?.right_approved_count === "number" ? diff.risk_plane.right_approved_count : 0,
            left_rejected_count: typeof diff.risk_plane?.left_rejected_count === "number" ? diff.risk_plane.left_rejected_count : 0,
            right_rejected_count: typeof diff.risk_plane?.right_rejected_count === "number" ? diff.risk_plane.right_rejected_count : 0,
            action_count_changes: Array.isArray(diff.risk_plane?.action_count_changes)
              ? diff.risk_plane.action_count_changes.map(normalizeEvidenceCountChange)
              : [],
            reason_count_changes: Array.isArray(diff.risk_plane?.reason_count_changes)
              ? diff.risk_plane.reason_count_changes.map(normalizeEvidenceCountChange)
              : [],
            first_divergence: normalizeEvidenceFirstDivergence(diff.risk_plane?.first_divergence)
          }
        : null,
      execution_capability: diff?.execution_capability
        ? {
            status: sanitizeText(diff.execution_capability?.status, "missing"),
            left_source_count: typeof diff.execution_capability?.left_source_count === "number" ? diff.execution_capability.left_source_count : 0,
            right_source_count: typeof diff.execution_capability?.right_source_count === "number" ? diff.execution_capability.right_source_count : 0,
            left_accepted_count: typeof diff.execution_capability?.left_accepted_count === "number" ? diff.execution_capability.left_accepted_count : 0,
            right_accepted_count: typeof diff.execution_capability?.right_accepted_count === "number" ? diff.execution_capability.right_accepted_count : 0,
            left_rejected_count: typeof diff.execution_capability?.left_rejected_count === "number" ? diff.execution_capability.left_rejected_count : 0,
            right_rejected_count: typeof diff.execution_capability?.right_rejected_count === "number" ? diff.execution_capability.right_rejected_count : 0,
            runtime_mode_changes: Array.isArray(diff.execution_capability?.runtime_mode_changes)
              ? diff.execution_capability.runtime_mode_changes.map(normalizeEvidenceCountChange)
              : [],
            capability_kind_changes: Array.isArray(diff.execution_capability?.capability_kind_changes)
              ? diff.execution_capability.capability_kind_changes.map(normalizeEvidenceCountChange)
              : [],
            capability_source_changes: Array.isArray(diff.execution_capability?.capability_source_changes)
              ? diff.execution_capability.capability_source_changes.map(normalizeEvidenceCountChange)
              : [],
            status_changes: Array.isArray(diff.execution_capability?.status_changes)
              ? diff.execution_capability.status_changes.map(normalizeEvidenceCountChange)
              : [],
            first_divergence: normalizeEvidenceFirstDivergence(diff.execution_capability?.first_divergence)
          }
        : null,
      metrics: diff?.metrics
        ? {
            status: sanitizeText(diff.metrics?.status, "missing"),
            fields: Array.isArray(diff.metrics?.fields)
              ? diff.metrics.fields.map((field) => ({
                  key: sanitizeText(field?.key, ""),
                  status: sanitizeText(field?.status, "same"),
                  left_value: sanitizeText(field?.left_value, ""),
                  right_value: sanitizeText(field?.right_value, "")
                }))
              : []
          }
        : null
    };
  };

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
    strategy_config_diff: normalizeStrategyConfigDiff(compare?.strategy_config_diff),
    strategy_config_evidence_diff: normalizeStrategyConfigEvidenceDiff(compare?.strategy_config_evidence_diff),
    has_changes: Boolean(compare?.has_changes)
  };
}

function graphExistsInIndex(graph, graphIndex = []) {
  const graphId = graph?.metadata?.graph_id;
  if (!graphId) return false;
  return graphIndex.some((entry) => entry.graph_id === graphId);
}


export {
  createSafeFallbackCapabilities,
  DEFAULT_CAPABILITIES as defaultCapabilities,
  defaultRegistry,
  fallbackRunnableGraph,
  graphExistsInIndex,
  hasUsableGraphShape,
  isDeprecatedBuiltinSampleGraph,
  normalizeGraphIndex,
  normalizeGraphAuditHistory,
  normalizeGraphVersionCompare,
  normalizeGraphVersions,
  normalizeGraphShape,
  recordRecentNodeIds,
  resolveGraphForDetail,
  resolveLoadedGraph,
  resolveLoadedGraphWithRegistry,
  withRecentNodeIds,
  attachValidationWithRegistry,
  buildRegistryFromCapabilities
};
