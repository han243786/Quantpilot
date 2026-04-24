use super::*;

pub(super) async fn get_capabilities() -> impl IntoResponse {
    Json(build_capability_response())
}

pub(super) fn build_capability_response() -> CapabilityResponse {
    let runtime_boundary = runtime_support_boundary();
    let supported_indicator_kind_list = supported_indicator_kinds().to_vec();
    let declared_module_keys = DECLARED_FRONTEND_MODULE_KEYS.to_vec();
    let supported_module_keys = SUPPORTED_FRONTEND_MODULE_KEYS.to_vec();
    let supported_module_set = supported_module_keys
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let unsupported_module_reasons = unsupported_frontend_module_reasons();

    CapabilityResponse {
        api_version: CAPABILITY_API_VERSION,
        strategy_ir: StrategyIrCapabilitySummary {
            declared_indicator_kinds: declared_indicator_kinds().to_vec(),
            supported_indicator_kinds: supported_indicator_kind_list.clone(),
            indicator_support: declared_indicator_kinds()
                .iter()
                .copied()
                .map(|kind| IndicatorCapabilityEntry {
                    kind,
                    status: if supported_indicator_kind_list.contains(&kind) {
                        CapabilitySupportStatus::Supported
                    } else {
                        CapabilitySupportStatus::DeclaredOnly
                    },
                    reason: indicator_declared_only_reason(kind),
                })
                .collect(),
        },
        runtime: RuntimeCapabilitySummary {
            supported_modes: runtime_boundary.runtime_modes.to_vec(),
            supported_execution_modules: runtime_boundary.execution_module_keys.to_vec(),
            mode_support: runtime_boundary
                .runtime_modes
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
            execution_module_support: runtime_boundary
                .execution_module_keys
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
        },
        market_data: MarketDataCapabilitySummary {
            supported_exchanges: SUPPORTED_EXCHANGES.to_vec(),
            supported_symbols: SUPPORTED_SYMBOLS.to_vec(),
            exchange_support: SUPPORTED_EXCHANGES
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
            symbol_support: SUPPORTED_SYMBOLS
                .iter()
                .copied()
                .map(supported_named_capability)
                .collect(),
        },
        frontend: FrontendCapabilitySummary {
            declared_module_keys,
            supported_module_keys,
            unsupported_module_reasons: unsupported_module_reasons.clone(),
            module_support: DECLARED_FRONTEND_MODULE_KEYS
                .iter()
                .copied()
                .map(|module_key| ModuleCapabilityEntry {
                    module_key,
                    status: if supported_module_set.contains(module_key) {
                        CapabilitySupportStatus::Supported
                    } else {
                        CapabilitySupportStatus::DeclaredOnly
                    },
                    reason: unsupported_module_reasons.get(module_key).copied(),
                })
                .collect(),
        },
    }
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
