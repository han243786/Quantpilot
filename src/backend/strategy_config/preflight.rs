use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backend::strategy_config::artifact::{
    build_strategy_config_artifact, finding, ConfigDomainReadiness, StrategyConfigArtifact,
    StrategyConfigArtifactRequest, StrategyConfigFinding,
};
use crate::{current_time_ms, AppState};

pub const MODULE_ID: &str = "backend.strategy_config.preflight";

const STRATEGY_CONFIG_PREFLIGHT_SCHEMA: &str = "quantpilot/v4-strategy-config-preflight/v1";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router.route(
        "/api/v1/strategy-config/preflight",
        post(preflight_strategy_config),
    )
}

async fn preflight_strategy_config(
    Json(request): Json<StrategyConfigArtifactRequest>,
) -> Result<Json<StrategyConfigPreflightReport>, (StatusCode, String)> {
    let artifact = build_strategy_config_artifact(request, current_time_ms())?;
    Ok(Json(build_preflight_report(artifact)))
}

pub(crate) fn build_strategy_config_preflight_value(
    request: StrategyConfigArtifactRequest,
) -> Result<Value, (StatusCode, String)> {
    let artifact = build_strategy_config_artifact(request, current_time_ms())?;
    serde_json::to_value(build_preflight_report(artifact)).map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("strategy config preflight serialization failed: {}", error),
        )
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StrategyConfigPreflightReport {
    pub(crate) schema_version: String,
    pub(crate) artifact: StrategyConfigArtifact,
    pub(crate) decision: PreflightDecision,
    pub(crate) can_compile: bool,
    pub(crate) can_paper_simulated: bool,
    pub(crate) can_backtest: bool,
    pub(crate) can_paper_actual_demo: bool,
    pub(crate) can_live_execution: bool,
    pub(crate) allowed_actions: Vec<String>,
    pub(crate) blocked_actions: Vec<PreflightBlockedAction>,
    pub(crate) findings: Vec<StrategyConfigFinding>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PreflightDecision {
    Ready,
    Restricted,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PreflightBlockedAction {
    pub(crate) action: String,
    pub(crate) reason: String,
}

pub(crate) fn build_preflight_report(
    artifact: StrategyConfigArtifact,
) -> StrategyConfigPreflightReport {
    let has_source = artifact.source.graph_digest.is_some()
        || artifact.source.runtime_config_digest.is_some()
        || artifact.source.qs_digest.is_some()
        || artifact.source.core_ir_digest.is_some()
        || artifact.source.v4_graph_digest.is_some();
    let unsupported_execution = artifact
        .runtime_boundary
        .execution_capability_sources
        .iter()
        .any(|source| source == "unsupported");
    let capability_current = artifact.capability.capability_snapshot_status == "current";
    let ai_binding_incomplete = artifact.proposal_bindings.iter().any(|binding| {
        binding.before_digest.is_none()
            || binding.after_digest.is_none()
            || binding.sandbox_status == "failed"
    });
    let can_compile = has_source && !unsupported_execution;
    let can_paper_simulated = can_compile && capability_current;
    let can_backtest = can_compile && capability_current;
    let can_paper_actual_demo = can_compile
        && capability_current
        && artifact.runtime_boundary.mode_label == "PaperActual"
        && artifact.runtime_boundary.provider_order_submission_allowed;

    let mut findings = artifact
        .config_domains
        .iter()
        .flat_map(|domain| domain.findings.clone())
        .collect::<Vec<_>>();
    let mut blocked_actions = Vec::new();
    if !has_source {
        findings.push(finding(
            "error",
            "strategy_config_source_missing",
            "策略配置缺少 graph、QS、Core IR 或 v4 graph 来源，不能继续运行前核验。",
        ));
        blocked_actions.push(blocked("compile", "缺少策略配置来源"));
    }
    if unsupported_execution {
        blocked_actions.push(blocked(
            "start_runtime",
            "当前策略需要 unsupported 执行能力",
        ));
        blocked_actions.push(blocked(
            "activate_proposal",
            "unsupported 执行能力不能进入激活路径",
        ));
    }
    if !capability_current {
        findings.push(finding(
            "warning",
            "strategy_config_stale_capability",
            "当前策略配置使用的 capability 快照不是后端当前快照，请刷新能力后重新核验。",
        ));
        blocked_actions.push(blocked("start_runtime", "capability 快照不是后端当前快照"));
        blocked_actions.push(blocked("run_backtest", "capability 快照不是后端当前快照"));
        blocked_actions.push(blocked(
            "activate_proposal",
            "capability 快照不是后端当前快照",
        ));
    }
    if ai_binding_incomplete {
        blocked_actions.push(blocked(
            "activate_proposal",
            "AI 提案缺少配置域 digest 或沙箱未通过",
        ));
    }
    blocked_actions.push(blocked(
        "live_execution",
        "live_execution_allowed=false，当前不开放真实资金自动交易",
    ));

    let decision = if !has_source || unsupported_execution {
        PreflightDecision::Blocked
    } else if artifact
        .config_domains
        .iter()
        .any(|domain| domain.readiness == ConfigDomainReadiness::Restricted)
        || !capability_current
        || ai_binding_incomplete
    {
        PreflightDecision::Restricted
    } else {
        PreflightDecision::Ready
    };
    let mut allowed_actions = Vec::new();
    if can_compile {
        allowed_actions.push("compile".to_string());
    }
    if can_paper_simulated {
        allowed_actions.push("start_paper_simulated".to_string());
    }
    if can_backtest {
        allowed_actions.push("run_backtest".to_string());
    }
    if can_paper_actual_demo {
        allowed_actions.push("start_paper_actual_demo".to_string());
    }

    StrategyConfigPreflightReport {
        schema_version: STRATEGY_CONFIG_PREFLIGHT_SCHEMA.to_string(),
        artifact,
        decision,
        can_compile,
        can_paper_simulated,
        can_backtest,
        can_paper_actual_demo,
        can_live_execution: false,
        allowed_actions,
        blocked_actions,
        findings,
    }
}

fn blocked(action: impl Into<String>, reason: impl Into<String>) -> PreflightBlockedAction {
    PreflightBlockedAction {
        action: action.into(),
        reason: reason.into(),
    }
}
