use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::backend::strategy_config::artifact::{
    build_strategy_config_artifact, version_artifact_request, ConfigDomainId, ConfigDomainStatus,
    StrategyConfigArtifact,
};
use crate::{AppState, GraphVersionEntry};

const STRATEGY_CONFIG_DIFF_SCHEMA: &str = "quantpilot/v4-strategy-config-diff/v1";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    register_strategy_config_diff_route(router)
}

pub(crate) fn register_strategy_config_diff_route(router: Router<AppState>) -> Router<AppState> {
    router.route("/api/v1/strategy-config/diff", post(diff_strategy_config))
}

async fn diff_strategy_config(
    Json(request): Json<StrategyConfigDiffRequest>,
) -> Result<Json<StrategyConfigDiffReport>, (StatusCode, String)> {
    Ok(Json(build_diff_report(request.left, request.right)))
}

pub(crate) fn build_strategy_config_version_diff(
    graph_id: &str,
    left: &GraphVersionEntry,
    left_graph: &Value,
    left_qs_source: Option<String>,
    right: &GraphVersionEntry,
    right_graph: &Value,
    right_qs_source: Option<String>,
) -> Result<StrategyConfigDiffReport, (StatusCode, String)> {
    let left_artifact = build_strategy_config_artifact(
        version_artifact_request(graph_id, left, left_graph, left_qs_source),
        left.updated_at,
    )?;
    let right_artifact = build_strategy_config_artifact(
        version_artifact_request(graph_id, right, right_graph, right_qs_source),
        right.updated_at,
    )?;
    Ok(build_diff_report(left_artifact, right_artifact))
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StrategyConfigDiffRequest {
    pub(crate) left: StrategyConfigArtifact,
    pub(crate) right: StrategyConfigArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigDiffReport {
    pub(crate) schema_version: String,
    pub(crate) left_artifact_id: String,
    pub(crate) right_artifact_id: String,
    pub(crate) source_digest_changes: Vec<StrategyConfigDigestChange>,
    pub(crate) domain_changes: Vec<StrategyConfigDomainChange>,
    pub(crate) runtime_boundary_changed: bool,
    pub(crate) changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigDigestChange {
    pub(crate) field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigDomainChange {
    pub(crate) domain_id: ConfigDomainId,
    pub(crate) lifecycle_changed: bool,
    pub(crate) readiness_changed: bool,
    pub(crate) source_refs_changed: bool,
    pub(crate) findings_changed: bool,
}

fn build_diff_report(
    left: StrategyConfigArtifact,
    right: StrategyConfigArtifact,
) -> StrategyConfigDiffReport {
    let source_digest_changes = [
        (
            "graph_digest",
            left.source.graph_digest.clone(),
            right.source.graph_digest.clone(),
        ),
        (
            "runtime_config_digest",
            left.source.runtime_config_digest.clone(),
            right.source.runtime_config_digest.clone(),
        ),
        (
            "qs_digest",
            left.source.qs_digest.clone(),
            right.source.qs_digest.clone(),
        ),
        (
            "core_ir_digest",
            left.source.core_ir_digest.clone(),
            right.source.core_ir_digest.clone(),
        ),
        (
            "v4_graph_digest",
            left.source.v4_graph_digest.clone(),
            right.source.v4_graph_digest.clone(),
        ),
    ]
    .into_iter()
    .filter_map(|(field, before, after)| {
        if before == after {
            None
        } else {
            Some(StrategyConfigDigestChange {
                field: field.to_string(),
                before,
                after,
            })
        }
    })
    .collect::<Vec<_>>();

    let left_domains = domains_by_id(&left.config_domains);
    let right_domains = domains_by_id(&right.config_domains);
    let mut domain_ids = left_domains
        .keys()
        .chain(right_domains.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let domain_changes = domain_ids
        .iter()
        .filter_map(|domain_id| {
            let left_domain = left_domains.get(domain_id);
            let right_domain = right_domains.get(domain_id);
            let lifecycle_changed =
                left_domain.map(|d| d.lifecycle) != right_domain.map(|d| d.lifecycle);
            let readiness_changed =
                left_domain.map(|d| d.readiness) != right_domain.map(|d| d.readiness);
            let source_refs_changed =
                left_domain.map(|d| &d.source_refs) != right_domain.map(|d| &d.source_refs);
            let findings_changed =
                left_domain.map(|d| &d.findings) != right_domain.map(|d| &d.findings);
            if lifecycle_changed || readiness_changed || source_refs_changed || findings_changed {
                Some(StrategyConfigDomainChange {
                    domain_id: *domain_id,
                    lifecycle_changed,
                    readiness_changed,
                    source_refs_changed,
                    findings_changed,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    domain_ids.clear();

    let runtime_boundary_changed = left.runtime_boundary.mode_label
        != right.runtime_boundary.mode_label
        || left.runtime_boundary.provider_order_submission_attached
            != right.runtime_boundary.provider_order_submission_attached
        || left.runtime_boundary.provider_order_submission_allowed
            != right.runtime_boundary.provider_order_submission_allowed
        || left.runtime_boundary.live_execution_allowed
            != right.runtime_boundary.live_execution_allowed
        || left.runtime_boundary.execution_capability_sources
            != right.runtime_boundary.execution_capability_sources;

    let changed =
        !source_digest_changes.is_empty() || !domain_changes.is_empty() || runtime_boundary_changed;

    StrategyConfigDiffReport {
        schema_version: STRATEGY_CONFIG_DIFF_SCHEMA.to_string(),
        left_artifact_id: left.artifact_id,
        right_artifact_id: right.artifact_id,
        source_digest_changes,
        domain_changes,
        runtime_boundary_changed,
        changed,
    }
}

fn domains_by_id(domains: &[ConfigDomainStatus]) -> BTreeMap<ConfigDomainId, &ConfigDomainStatus> {
    domains
        .iter()
        .map(|domain| (domain.domain_id, domain))
        .collect()
}
