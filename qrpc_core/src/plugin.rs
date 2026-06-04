mod capability_contract;
mod execution_security_dependency;
mod manifest_validation;
mod registry;
mod taxonomy_extension;

pub use capability_contract::*;
pub use execution_security_dependency::*;
pub use manifest_validation::*;
pub use registry::*;
pub use taxonomy_extension::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> PluginManifest {
        PluginManifest {
            api_version: PLUGIN_MANIFEST_V1_VERSION.into(),
            id: "quantpilot.intent.custom_expr".into(),
            version: "0.1.0".into(),
            kind: PluginKind::Intent,
            display: PluginDisplay {
                name: "Custom Expr".into(),
                summary: "Restricted custom expression intent".into(),
            },
            capability_declarations: vec![PluginCapabilityDeclaration {
                id: PluginCapabilityContract::IntentModuleProvider
                    .as_str()
                    .into(),
                version: PLUGIN_CAPABILITY_CONTRACT_V1_VERSION.into(),
            }],
            extension_points: vec![ExtensionPoint::IntentModuleProvider],
            execution: PluginExecution {
                engine: PluginExecutionEngine::Builtin,
                entrypoint: "builtin.custom_expr".into(),
            },
            compatibility: PluginCompatibility {
                core_ir_version: "quantpilot/core-ir/v1".into(),
                capability_api_version: "quantpilot-capabilities/v1".into(),
            },
            security: PluginSecurity {
                max_compute_ms: 50,
                max_memory_mb: 64,
                allow_network: false,
                enforce_max_compute_ms: None,
                enforce_max_memory_mb: None,
            },
            dependencies: vec![],
            params_schema: None,
            plugin_type: None,
            atoms: vec![],
            hot_handoff: false,
            asset_management: false,
            signature: None,
        }
    }

    #[test]
    fn validates_minimal_manifest() {
        sample_manifest().validate().unwrap();
    }

    #[test]
    fn rejects_unknown_capability_contract() {
        let mut manifest = sample_manifest();
        manifest.capability_declarations[0].id = "quantpilot.capability.unknown".into();
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|item| item.contains("不支持的能力合约")));
    }

    #[test]
    fn rejects_mismatched_extension_point() {
        let mut manifest = sample_manifest();
        manifest.extension_points = vec![ExtensionPoint::ExecutionModuleProvider];
        let errors = manifest.validate().unwrap_err();
        assert!(errors.iter().any(|item| item.contains("无法附加到扩展点")));
    }

    #[test]
    fn registers_and_filters_manifest() {
        let mut registry = PluginRegistry::default();
        registry.register(sample_manifest()).unwrap();

        assert!(registry.get("quantpilot.intent.custom_expr").is_some());
        assert_eq!(
            registry
                .manifests_for_extension_point(ExtensionPoint::IntentModuleProvider)
                .len(),
            1
        );
    }

    #[test]
    fn rejects_duplicate_plugin_id() {
        let mut registry = PluginRegistry::default();
        registry.register(sample_manifest()).unwrap();
        let err = registry.register(sample_manifest()).unwrap_err();
        assert!(err.iter().any(|item| item.contains("已注册")));
    }
}
