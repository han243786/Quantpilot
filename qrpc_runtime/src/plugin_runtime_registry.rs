use crate::{
    AgentModuleProvider, DataModuleProvider, ExecutionModuleProvider, IntentModuleProvider,
    RiskCheckerProvider,
};
use qrpc_core::{ExtensionPoint, PluginManifest, PluginRegistry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginLifecycleState {
    Registered,
    Active,
    Stopped,
    Faulted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimePluginLifecycle {
    pub plugin_id: String,
    pub state: PluginLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault_reason: Option<String>,
}

#[derive(Default)]
pub struct RuntimePluginRegistry {
    manifests: PluginRegistry,
    lifecycle: BTreeMap<String, RuntimePluginLifecycle>,
    data_providers: BTreeMap<String, Arc<dyn DataModuleProvider>>,
    intent_providers: BTreeMap<String, Arc<dyn IntentModuleProvider>>,
    agent_providers: BTreeMap<String, Arc<dyn AgentModuleProvider>>,
    risk_providers: BTreeMap<String, Arc<dyn RiskCheckerProvider>>,
    execution_providers: BTreeMap<String, Arc<Mutex<dyn ExecutionModuleProvider>>>,
}

impl std::fmt::Debug for RuntimePluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimePluginRegistry")
            .field("manifests", &self.manifests.manifests().len())
            .field("lifecycle", &self.lifecycle)
            .finish()
    }
}

impl RuntimePluginRegistry {
    pub fn register_data_provider(
        &mut self,
        manifest: PluginManifest,
        provider: Arc<dyn DataModuleProvider>,
    ) -> Result<(), Vec<String>> {
        self.register_manifest_for_point(&manifest, ExtensionPoint::DataModuleProvider)?;
        let plugin_id = manifest.id.clone();
        self.manifests.register(manifest)?;
        self.data_providers.insert(plugin_id.clone(), provider);
        self.lifecycle.insert(
            plugin_id.clone(),
            RuntimePluginLifecycle {
                plugin_id,
                state: PluginLifecycleState::Registered,
                fault_reason: None,
            },
        );
        Ok(())
    }

    pub fn register_intent_provider(
        &mut self,
        manifest: PluginManifest,
        provider: Arc<dyn IntentModuleProvider>,
    ) -> Result<(), Vec<String>> {
        self.register_manifest_for_point(&manifest, ExtensionPoint::IntentModuleProvider)?;
        let plugin_id = manifest.id.clone();
        self.manifests.register(manifest)?;
        self.intent_providers.insert(plugin_id.clone(), provider);
        self.lifecycle.insert(
            plugin_id.clone(),
            RuntimePluginLifecycle {
                plugin_id,
                state: PluginLifecycleState::Registered,
                fault_reason: None,
            },
        );
        Ok(())
    }

    pub fn register_agent_provider(
        &mut self,
        manifest: PluginManifest,
        provider: Arc<dyn AgentModuleProvider>,
    ) -> Result<(), Vec<String>> {
        self.register_manifest_for_point(&manifest, ExtensionPoint::AgentModuleProvider)?;
        let plugin_id = manifest.id.clone();
        self.manifests.register(manifest)?;
        self.agent_providers.insert(plugin_id.clone(), provider);
        self.lifecycle.insert(
            plugin_id.clone(),
            RuntimePluginLifecycle {
                plugin_id,
                state: PluginLifecycleState::Registered,
                fault_reason: None,
            },
        );
        Ok(())
    }

    pub fn register_risk_provider(
        &mut self,
        manifest: PluginManifest,
        provider: Arc<dyn RiskCheckerProvider>,
    ) -> Result<(), Vec<String>> {
        self.register_manifest_for_point(&manifest, ExtensionPoint::RiskCheckerProvider)?;
        let plugin_id = manifest.id.clone();
        self.manifests.register(manifest)?;
        self.risk_providers.insert(plugin_id.clone(), provider);
        self.lifecycle.insert(
            plugin_id.clone(),
            RuntimePluginLifecycle {
                plugin_id,
                state: PluginLifecycleState::Registered,
                fault_reason: None,
            },
        );
        Ok(())
    }

    pub fn register_execution_provider(
        &mut self,
        manifest: PluginManifest,
        provider: Arc<Mutex<dyn ExecutionModuleProvider>>,
    ) -> Result<(), Vec<String>> {
        self.register_manifest_for_point(&manifest, ExtensionPoint::ExecutionModuleProvider)?;
        let plugin_id = manifest.id.clone();
        self.manifests.register(manifest)?;
        self.execution_providers.insert(plugin_id.clone(), provider);
        self.lifecycle.insert(
            plugin_id.clone(),
            RuntimePluginLifecycle {
                plugin_id,
                state: PluginLifecycleState::Registered,
                fault_reason: None,
            },
        );
        Ok(())
    }

    pub fn activate(&mut self, plugin_id: &str) -> Result<(), String> {
        let Some(lifecycle) = self.lifecycle.get_mut(plugin_id) else {
            return Err(format!("plugin `{plugin_id}` is not registered"));
        };
        lifecycle.state = PluginLifecycleState::Active;
        lifecycle.fault_reason = None;
        Ok(())
    }

    pub fn deactivate(&mut self, plugin_id: &str) -> Result<(), String> {
        let Some(lifecycle) = self.lifecycle.get_mut(plugin_id) else {
            return Err(format!("plugin `{plugin_id}` is not registered"));
        };
        lifecycle.state = PluginLifecycleState::Stopped;
        Ok(())
    }

    pub fn mark_faulted(&mut self, plugin_id: &str, reason: impl Into<String>) -> Result<(), String> {
        let Some(lifecycle) = self.lifecycle.get_mut(plugin_id) else {
            return Err(format!("plugin `{plugin_id}` is not registered"));
        };
        lifecycle.state = PluginLifecycleState::Faulted;
        lifecycle.fault_reason = Some(reason.into());
        Ok(())
    }

    pub fn lifecycle(&self, plugin_id: &str) -> Option<&RuntimePluginLifecycle> {
        self.lifecycle.get(plugin_id)
    }

    pub fn active_data_provider(&self, plugin_id: &str) -> Option<Arc<dyn DataModuleProvider>> {
        self.is_active(plugin_id)
            .then(|| self.data_providers.get(plugin_id).cloned())
            .flatten()
    }

    pub fn active_intent_provider(
        &self,
        plugin_id: &str,
    ) -> Option<Arc<dyn IntentModuleProvider>> {
        self.is_active(plugin_id)
            .then(|| self.intent_providers.get(plugin_id).cloned())
            .flatten()
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.manifests.manifests()
    }

    fn is_active(&self, plugin_id: &str) -> bool {
        self.lifecycle
            .get(plugin_id)
            .is_some_and(|lifecycle| lifecycle.state == PluginLifecycleState::Active)
    }

    fn register_manifest_for_point(
        &self,
        manifest: &PluginManifest,
        extension_point: ExtensionPoint,
    ) -> Result<(), Vec<String>> {
        if manifest.extension_points.contains(&extension_point) {
            Ok(())
        } else {
            Err(vec![format!(
                "plugin `{}` must declare extension point `{}`",
                manifest.id,
                extension_point.as_str()
            )])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuiltinDataModule;
    use qrpc_core::{
        PluginCapabilityContract, PluginCapabilityDeclaration, PluginCompatibility, PluginDisplay,
        PluginExecution, PluginExecutionEngine, PluginKind, PluginSecurity,
        PLUGIN_CAPABILITY_CONTRACT_V1_VERSION, PLUGIN_MANIFEST_V1_VERSION,
    };

    fn sample_data_manifest() -> PluginManifest {
        PluginManifest {
            api_version: PLUGIN_MANIFEST_V1_VERSION.into(),
            id: "quantpilot.data.alt_feed".into(),
            version: "0.1.0".into(),
            kind: PluginKind::Data,
            display: PluginDisplay {
                name: "Alt Feed".into(),
                summary: "Alternative feed".into(),
            },
            capability_declarations: vec![PluginCapabilityDeclaration {
                id: PluginCapabilityContract::DataModuleProvider.as_str().into(),
                version: PLUGIN_CAPABILITY_CONTRACT_V1_VERSION.into(),
            }],
            extension_points: vec![ExtensionPoint::DataModuleProvider],
            execution: PluginExecution {
                engine: PluginExecutionEngine::Native,
                entrypoint: "plugin.alt_feed".into(),
            },
            compatibility: PluginCompatibility {
                core_ir_version: "quantpilot/core-ir/v1".into(),
                capability_api_version: "quantpilot-capabilities/v1".into(),
            },
            security: PluginSecurity {
                max_compute_ms: 100,
                max_memory_mb: 64,
                allow_network: false,
            },
            dependencies: vec![],
            params_schema: None,
        }
    }

    #[test]
    fn registers_and_activates_data_provider() {
        let mut registry = RuntimePluginRegistry::default();
        registry
            .register_data_provider(sample_data_manifest(), Arc::new(BuiltinDataModule::default()))
            .unwrap();

        assert_eq!(
            registry
                .lifecycle("quantpilot.data.alt_feed")
                .expect("lifecycle")
                .state,
            PluginLifecycleState::Registered
        );
        assert!(registry
            .active_data_provider("quantpilot.data.alt_feed")
            .is_none());

        registry.activate("quantpilot.data.alt_feed").unwrap();
        assert!(registry
            .active_data_provider("quantpilot.data.alt_feed")
            .is_some());
    }

    #[test]
    fn rejects_mismatched_extension_point_for_data_provider() {
        let mut registry = RuntimePluginRegistry::default();
        let mut manifest = sample_data_manifest();
        manifest.extension_points = vec![ExtensionPoint::IntentModuleProvider];
        let err = registry
            .register_data_provider(manifest, Arc::new(BuiltinDataModule::default()))
            .unwrap_err();
        assert!(err
            .iter()
            .any(|item| item.contains("must declare extension point")));
    }

    #[test]
    fn faulted_plugin_is_not_exposed_as_active() {
        let mut registry = RuntimePluginRegistry::default();
        registry
            .register_data_provider(sample_data_manifest(), Arc::new(BuiltinDataModule::default()))
            .unwrap();
        registry.activate("quantpilot.data.alt_feed").unwrap();
        registry
            .mark_faulted("quantpilot.data.alt_feed", "panic in provider")
            .unwrap();

        let lifecycle = registry.lifecycle("quantpilot.data.alt_feed").unwrap();
        assert_eq!(lifecycle.state, PluginLifecycleState::Faulted);
        assert_eq!(lifecycle.fault_reason.as_deref(), Some("panic in provider"));
        assert!(registry
            .active_data_provider("quantpilot.data.alt_feed")
            .is_none());
    }
}
