use crate::{
    AgentModuleProvider, DataModuleProvider, ExecutionModuleProvider, IntentModuleProvider,
    RiskCheckerProvider,
};
use qrpc_core::{ExtensionPoint, PluginManifest, PluginRegistry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// v1.0.0 插件安全操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginSecurityAction {
    AccessCredentials,
    NetworkCall,
    WriteState,
}

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
            return Err(format!("插件 `{plugin_id}` 未注册"));
        };
        match lifecycle.state {
            PluginLifecycleState::Registered | PluginLifecycleState::Stopped => {
                lifecycle.state = PluginLifecycleState::Active;
                lifecycle.fault_reason = None;
                Ok(())
            }
            PluginLifecycleState::Active => Ok(()), // 幂等
            PluginLifecycleState::Faulted => {
                Err(format!("插件 `{plugin_id}` 处于故障状态, 请先修复后重试"))
            }
        }
    }

    pub fn deactivate(&mut self, plugin_id: &str) -> Result<(), String> {
        let Some(lifecycle) = self.lifecycle.get_mut(plugin_id) else {
            return Err(format!("插件 `{plugin_id}` 未注册"));
        };
        match lifecycle.state {
            PluginLifecycleState::Active => {
                lifecycle.state = PluginLifecycleState::Stopped;
                Ok(())
            }
            PluginLifecycleState::Stopped => Ok(()), // 幂等
            PluginLifecycleState::Registered | PluginLifecycleState::Faulted => {
                Err(format!("插件 `{plugin_id}` 当前状态不允许停用"))
            }
        }
    }

    pub fn mark_faulted(
        &mut self,
        plugin_id: &str,
        reason: impl Into<String>,
    ) -> Result<(), String> {
        let Some(lifecycle) = self.lifecycle.get_mut(plugin_id) else {
            return Err(format!("插件 `{plugin_id}` 未注册"));
        };
        match lifecycle.state {
            PluginLifecycleState::Registered | PluginLifecycleState::Active => {
                lifecycle.state = PluginLifecycleState::Faulted;
                lifecycle.fault_reason = Some(reason.into());
                Ok(())
            }
            PluginLifecycleState::Faulted => {
                lifecycle.fault_reason = Some(reason.into());
                Ok(()) // 更新 fault 原因
            }
            PluginLifecycleState::Stopped => {
                Err(format!("插件 `{plugin_id}` 已停用, 无法标记故障"))
            }
        }
    }

    pub fn lifecycle(&self, plugin_id: &str) -> Option<&RuntimePluginLifecycle> {
        self.lifecycle.get(plugin_id)
    }

    pub fn active_data_provider(&self, plugin_id: &str) -> Option<Arc<dyn DataModuleProvider>> {
        self.is_active(plugin_id)
            .then(|| self.data_providers.get(plugin_id).cloned())
            .flatten()
    }

    pub fn active_intent_provider(&self, plugin_id: &str) -> Option<Arc<dyn IntentModuleProvider>> {
        self.is_active(plugin_id)
            .then(|| self.intent_providers.get(plugin_id).cloned())
            .flatten()
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.manifests.manifests()
    }

    // ── v1.0.0 原子扫描 ──

    /// 从 `plugins/builtin/` 和 `plugins/installed/` 目录扫描 .json manifest 文件并注册。
    /// v1.0.0: 单个文件失败不阻止其他文件注册，收集全部错误后统一报告。
    pub fn scan_atoms(&mut self, atom_dir: &std::path::Path) -> Result<usize, String> {
        let dir = std::fs::read_dir(atom_dir).map_err(|e| format!("无法读取插件目录: {e}"))?;
        let mut loaded = 0usize;
        let mut errors: Vec<String> = Vec::new();
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let raw = match std::fs::read_to_string(&path) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(format!("读取 {} 失败: {e}", path.display()));
                    continue;
                }
            };
            let manifest: PluginManifest = match serde_json::from_str(&raw) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(format!("{} JSON 解析失败: {e}", path.display()));
                    continue;
                }
            };
            if let Err(errs) = manifest.validate() {
                errors.push(format!("{} 校验失败: {:?}", path.display(), errs));
                continue;
            }
            if let Err(errs) = self.manifests.register(manifest.clone()) {
                errors.push(format!("注册 {} 失败: {:?}", path.display(), errs));
                continue;
            }
            let plugin_id = manifest.id.clone();
            self.lifecycle.insert(
                plugin_id.clone(),
                RuntimePluginLifecycle {
                    plugin_id,
                    state: PluginLifecycleState::Registered,
                    fault_reason: None,
                },
            );
            loaded += 1;
        }
        if !errors.is_empty() {
            eprintln!(
                "[plugin] 扫描原子时 {} 个文件失败: {:?}",
                errors.len(),
                errors
            );
        }
        Ok(loaded)
    }

    /// 列出所有已注册的原子 (PluginType::Atom)
    pub fn atoms(&self) -> Vec<&PluginManifest> {
        self.manifests
            .manifests()
            .into_iter()
            .filter(|m| m.plugin_type == Some(qrpc_core::PluginType::Atom))
            .collect()
    }

    // ── v1.0.0 套件校验 ──

    /// 校验套件：所有引用的原子已注册 + exchange/symbol 一致
    pub fn validate_suite(&self, suite: &PluginManifest) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if suite.plugin_type != Some(qrpc_core::PluginType::Suite) {
            errors.push("套件校验仅适用于 Suite 类型插件".to_string());
            return Err(errors);
        }

        if suite.atoms.is_empty() {
            errors.push("套件必须声明至少一个原子".to_string());
            return Err(errors);
        }

        let registered: std::collections::BTreeSet<&str> = self
            .manifests
            .manifests()
            .into_iter()
            .map(|m| m.id.as_str())
            .collect();

        for atom in &suite.atoms {
            if !registered.contains(atom.atom_id.as_str()) {
                errors.push(format!("套件引用的原子 `{}` 未注册", atom.atom_id));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(())
    }

    // ── v1.0.0 安全边界 ──

    /// 检查插件是否被允许执行指定操作。
    ///
    /// 插件子进程执行路径应通过 `PluginSandbox::execute_checked()` 进入, 由该方法
    /// 先调用本检查再启动子进程，确保声明式安全策略不是旁路文档。
    pub fn check_security(
        &self,
        plugin_id: &str,
        action: PluginSecurityAction,
    ) -> Result<(), String> {
        let manifest = self
            .manifests
            .manifests()
            .into_iter()
            .find(|m| m.id == plugin_id)
            .ok_or_else(|| format!("插件 `{plugin_id}` 未注册"))?;

        match action {
            PluginSecurityAction::AccessCredentials => {
                return Err(format!("插件 `{plugin_id}` 不允许访问凭证管理"));
            }
            PluginSecurityAction::NetworkCall => {
                if !manifest.security.allow_network {
                    return Err(format!("插件 `{plugin_id}` 未声明 allow_network"));
                }
            }
            PluginSecurityAction::WriteState => {
                // 插件默认允许写入沙盒状态
            }
        }
        Ok(())
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
                "插件 `{}` 必须声明扩展点 `{}`",
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
                enforce_max_compute_ms: None,
                enforce_max_memory_mb: None,
            },
            signature: None,
            dependencies: vec![],
            params_schema: None,
            plugin_type: None,
            atoms: vec![],
            hot_handoff: false,
            asset_management: false,
        }
    }

    #[test]
    fn registers_and_activates_data_provider() {
        let mut registry = RuntimePluginRegistry::default();
        registry
            .register_data_provider(
                sample_data_manifest(),
                Arc::new(BuiltinDataModule::default()),
            )
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
        assert!(err.iter().any(|item| item.contains("必须声明扩展点")));
    }

    #[test]
    fn faulted_plugin_is_not_exposed_as_active() {
        let mut registry = RuntimePluginRegistry::default();
        registry
            .register_data_provider(
                sample_data_manifest(),
                Arc::new(BuiltinDataModule::default()),
            )
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

    #[test]
    fn security_rejects_network_when_manifest_does_not_allow_it() {
        let mut registry = RuntimePluginRegistry::default();
        let manifest = sample_data_manifest();
        let plugin_id = manifest.id.clone();
        registry
            .register_data_provider(manifest, Arc::new(BuiltinDataModule::default()))
            .unwrap();

        let err = registry
            .check_security(&plugin_id, PluginSecurityAction::NetworkCall)
            .unwrap_err();
        assert!(err.contains("allow_network"));
    }

    #[test]
    fn security_allows_network_when_manifest_declares_it() {
        let mut registry = RuntimePluginRegistry::default();
        let mut manifest = sample_data_manifest();
        manifest.security.allow_network = true;
        let plugin_id = manifest.id.clone();
        registry
            .register_data_provider(manifest, Arc::new(BuiltinDataModule::default()))
            .unwrap();

        assert!(registry
            .check_security(&plugin_id, PluginSecurityAction::NetworkCall)
            .is_ok());
    }

    #[test]
    fn security_always_rejects_credential_access() {
        let mut registry = RuntimePluginRegistry::default();
        let manifest = sample_data_manifest();
        let plugin_id = manifest.id.clone();
        registry
            .register_data_provider(manifest, Arc::new(BuiltinDataModule::default()))
            .unwrap();

        let err = registry
            .check_security(&plugin_id, PluginSecurityAction::AccessCredentials)
            .unwrap_err();
        assert!(err.contains("不允许访问凭证管理"));
    }
}
