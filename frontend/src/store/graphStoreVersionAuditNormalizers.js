import { sanitizeDisplayText } from "../utils/errorText";
import {
  DEFAULT_LOCAL_ACTOR,
  normalizeActorIdentity
} from "./graphStoreActorCollaboration";

function sanitizeText(value, fallback) {
  return sanitizeDisplayText(value, fallback);
}

export function normalizeGraphIndex(entries) {
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

export function normalizeGraphAuditHistory(entries) {
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

export function normalizeGraphVersions(entries) {
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

export function normalizeGraphVersionCompare(compare) {
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

export function graphExistsInIndex(graph, graphIndex = []) {
  const graphId = graph?.metadata?.graph_id;
  if (!graphId) return false;
  return graphIndex.some((entry) => entry.graph_id === graphId);
}
