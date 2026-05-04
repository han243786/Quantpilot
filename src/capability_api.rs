use super::*;

#[derive(Debug, Clone, Serialize)]
pub(super) struct CapabilityContract {
    pub(super) api_version: &'static str,
    pub(super) schema_version: &'static str,
    pub(super) chain_stages: Vec<&'static str>,
    pub(super) declared_indicator_kinds: Vec<IndicatorKind>,
    pub(super) supported_indicator_kinds: Vec<IndicatorKind>,
    pub(super) runtime_modes: Vec<&'static str>,
    pub(super) execution_module_keys: Vec<&'static str>,
    pub(super) supported_exchanges: Vec<&'static str>,
    pub(super) supported_symbols: Vec<&'static str>,
    pub(super) declared_module_keys: Vec<&'static str>,
    pub(super) supported_module_keys: Vec<&'static str>,
    pub(super) unsupported_module_reasons: BTreeMap<&'static str, &'static str>,
    pub(super) versioning: CapabilityVersioningSummary,
    pub(super) permission_boundary: CapabilityPermissionBoundarySummary,
}

pub(super) async fn get_capabilities() -> impl IntoResponse {
    Json(build_capability_response())
}

pub(super) fn build_capability_response() -> CapabilityResponse {
    let contract = build_capability_contract();
    let supported_module_set = contract
        .supported_module_keys
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let schema_hash = capability_contract_hash(&contract);

    CapabilityResponse {
        api_version: contract.api_version,
        schema_version: contract.schema_version,
        schema_hash,
        chain_stages: contract.chain_stages,
        strategy_ir: StrategyIrCapabilitySummary {
            declared_indicator_kinds: contract.declared_indicator_kinds.clone(),
            supported_indicator_kinds: contract.supported_indicator_kinds.clone(),
            indicator_support: contract
                .declared_indicator_kinds
                .iter()
                .copied()
                .map(|kind| IndicatorCapabilityEntry {
                    status: if contract.supported_indicator_kinds.contains(&kind) {
                        CapabilitySupportStatus::Supported
                    } else {
                        CapabilitySupportStatus::DeclaredOnly
                    },
                    kind,
                    reason: indicator_declared_only_reason(kind),
                })
                .collect(),
        },
        runtime: RuntimeCapabilitySummary {
            supported_modes: contract.runtime_modes.clone(),
            supported_execution_modules: contract.execution_module_keys.clone(),
            mode_support: contract
                .runtime_modes
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
            execution_module_support: contract
                .execution_module_keys
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
        },
        market_data: MarketDataCapabilitySummary {
            supported_exchanges: contract.supported_exchanges.clone(),
            supported_symbols: contract.supported_symbols.clone(),
            exchange_support: contract
                .supported_exchanges
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
            symbol_support: contract
                .supported_symbols
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
        },
        frontend: FrontendCapabilitySummary {
            declared_module_keys: contract.declared_module_keys.clone(),
            supported_module_keys: contract.supported_module_keys.clone(),
            unsupported_module_reasons: contract.unsupported_module_reasons.clone(),
            module_support: contract
                .declared_module_keys
                .iter()
                .copied()
                .map(|module_key| ModuleCapabilityEntry {
                    module_key,
                    status: if supported_module_set.contains(module_key) {
                        CapabilitySupportStatus::Supported
                    } else {
                        CapabilitySupportStatus::DeclaredOnly
                    },
                    reason: contract.unsupported_module_reasons.get(module_key).copied(),
                })
                .collect(),
        },
        versioning: contract.versioning,
        permission_boundary: contract.permission_boundary,
    }
}

#[cfg(test)]
pub(super) fn current_capability_hash() -> String {
    capability_contract_hash(&build_capability_contract())
}

pub(super) fn current_capability_context() -> FrontendCapabilityContext {
    let contract = build_capability_contract();
    let permission = contract.permission_boundary;

    FrontendCapabilityContext {
        schema_hash: capability_contract_hash(&contract),
        permission_boundary: PermissionBoundarySnapshot {
            model_version: permission.model_version.to_string(),
            execution_owner_module: permission.execution_owner_module.to_string(),
            live_execution_allowed: permission.live_execution_allowed,
            ai_write_policy: permission.ai_write_policy.as_str().to_string(),
            plugin_network_default: permission.plugin_network_default.as_str().to_string(),
            non_execution_order_access: permission.non_execution_order_access.as_str().to_string(),
        },
    }
}

pub(super) fn build_capability_contract() -> CapabilityContract {
    let runtime_boundary = runtime_support_boundary();
    CapabilityContract {
        api_version: CAPABILITY_API_VERSION,
        schema_version: CAPABILITY_SCHEMA_VERSION,
        chain_stages: RUNTIME_CHAIN_STAGES.to_vec(),
        declared_indicator_kinds: declared_indicator_kinds().to_vec(),
        supported_indicator_kinds: supported_indicator_kinds().to_vec(),
        runtime_modes: runtime_boundary.runtime_modes.to_vec(),
        execution_module_keys: runtime_boundary.execution_module_keys.to_vec(),
        supported_exchanges: SUPPORTED_EXCHANGES.to_vec(),
        supported_symbols: SUPPORTED_SYMBOLS.to_vec(),
        declared_module_keys: DECLARED_FRONTEND_MODULE_KEYS.to_vec(),
        supported_module_keys: SUPPORTED_FRONTEND_MODULE_KEYS.to_vec(),
        unsupported_module_reasons: unsupported_frontend_module_reasons(),
        versioning: capability_versioning_summary(),
        permission_boundary: capability_permission_boundary_summary(),
    }
}

pub(super) fn capability_contract_hash(contract: &CapabilityContract) -> String {
    canonical_sha256_hash(contract)
}

pub(super) fn capability_versioning_summary() -> CapabilityVersioningSummary {
    CapabilityVersioningSummary {
        model_version: CAPABILITY_VERSIONING_MODEL_VERSION,
        strategy_version_source: "frontend_runtime_config.metadata.version",
        parameter_version_policy: "immutable_generation_pointer",
        deployment_revision_policy: "strategy_version_plus_compile_id_plus_capability_hash",
    }
}

pub(super) fn capability_permission_boundary_summary() -> CapabilityPermissionBoundarySummary {
    CapabilityPermissionBoundarySummary {
        model_version: CAPABILITY_PERMISSION_MODEL_VERSION,
        execution_owner_module: "builtin.execution.paper",
        live_execution_allowed: false,
        ai_write_policy: AiWritePolicy::ProposalOnly,
        plugin_network_default: BoundaryAccessPolicy::Deny,
        non_execution_order_access: BoundaryAccessPolicy::Deny,
    }
}

pub(super) fn runtime_governance_snapshot(
    metadata: &FrontendMetadata,
    parameter_fingerprint: Option<&str>,
) -> RuntimeGovernanceSnapshot {
    let contract = build_capability_contract();
    let capability_hash = capability_contract_hash(&contract);
    let parameter_version = parameter_fingerprint
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("config:{value}"))
        .unwrap_or_else(|| format!("compile:{}", metadata.compile_id));
    let deployment_revision = stable_json_hash(&json!({
        "graph_id": metadata.graph_id,
        "compile_id": metadata.compile_id,
        "strategy_version": metadata.version,
        "parameter_version": parameter_version,
        "capability_hash": capability_hash,
    }));
    let permission = contract.permission_boundary;

    RuntimeGovernanceSnapshot {
        schema_version: RUNTIME_GOVERNANCE_SCHEMA_VERSION.to_string(),
        governance_source: "current_runtime".to_string(),
        capability_hash,
        strategy_version: metadata.version.clone(),
        parameter_version,
        deployment_revision,
        permission_boundary: PermissionBoundarySnapshot {
            model_version: permission.model_version.to_string(),
            execution_owner_module: permission.execution_owner_module.to_string(),
            live_execution_allowed: permission.live_execution_allowed,
            ai_write_policy: permission.ai_write_policy.as_str().to_string(),
            plugin_network_default: permission.plugin_network_default.as_str().to_string(),
            non_execution_order_access: permission.non_execution_order_access.as_str().to_string(),
        },
    }
}

fn stable_json_hash(value: &Value) -> String {
    canonical_sha256_hash(value)
}

fn canonical_sha256_hash(value: &impl Serialize) -> String {
    let digest = canonical_json_sha256_digest(value)
        .expect("capability governance payloads must serialize for canonical hashing");
    format!("sha256:{}", digest.value)
}

pub(super) fn supported_named_capability(key: &'static str) -> NamedCapabilityEntry {
    NamedCapabilityEntry {
        key,
        status: CapabilitySupportStatus::Supported,
        reason: None,
    }
}

pub(super) fn indicator_declared_only_reason(kind: IndicatorKind) -> Option<&'static str> {
    let _ = kind;
    None
}

pub(super) fn unsupported_frontend_module_reasons() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::new()
}
