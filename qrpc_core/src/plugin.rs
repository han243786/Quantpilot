use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const PLUGIN_MANIFEST_V1_VERSION: &str = "quantpilot/plugin-manifest/v1";
pub const PLUGIN_CAPABILITY_CONTRACT_V1_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    #[serde(rename = "atom")]
    Atom,
    #[serde(rename = "suite")]
    Suite,
}

/// 套件中对原子的引用
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomRef {
    pub atom_id: String,
    pub version: String,
    pub kind: PluginKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub api_version: String,
    pub id: String,
    pub version: String,
    // v1.0.0: 双层模型 — Atom 或 Suite
    #[serde(default)]
    pub plugin_type: Option<PluginType>,
    pub kind: PluginKind,
    pub display: PluginDisplay,
    pub capability_declarations: Vec<PluginCapabilityDeclaration>,
    pub extension_points: Vec<ExtensionPoint>,
    pub execution: PluginExecution,
    pub compatibility: PluginCompatibility,
    pub security: PluginSecurity,
    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,
    #[serde(default)]
    pub params_schema: Option<Value>,
    // v1.0.0: 套件打包 — 仅 Suite 类型使用
    #[serde(default)]
    pub atoms: Vec<AtomRef>,
    // v1.0.0: 热接管声明
    #[serde(default)]
    pub hot_handoff: bool,
    // v1.0.0: 完整资产管理能力 (热接管前提)
    #[serde(default)]
    pub asset_management: bool,
    // v2.0.0: Ed25519 签名 (base64 编码), 用于验证 manifest 完整性
    #[serde(default)]
    pub signature: Option<String>,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.api_version != PLUGIN_MANIFEST_V1_VERSION {
            errors.push(format!(
                "manifest.api_version 必须为 `{}`",
                PLUGIN_MANIFEST_V1_VERSION
            ));
        }
        if self.id.trim().is_empty() {
            errors.push("manifest.id 不能为空".to_string());
        }
        if self.version.trim().is_empty() {
            errors.push("manifest.version 不能为空".to_string());
        }
        if self.display.name.trim().is_empty() {
            errors.push("manifest.display.name 不能为空".to_string());
        }
        if self.extension_points.is_empty() {
            errors.push("manifest.extension_points 必须至少包含一个条目".to_string());
        }

        let mut extension_points = BTreeSet::new();
        for point in &self.extension_points {
            if !extension_points.insert(*point) {
                errors.push(format!("重复的扩展点 `{point:?}`"));
            }
            if !self.kind.supported_extension_points().contains(point) {
                errors.push(format!(
                    "插件类型 `{}` 无法附加到扩展点 `{}`",
                    self.kind.as_str(),
                    point.as_str()
                ));
            }
        }

        let mut capability_ids = BTreeSet::new();
        for capability in &self.capability_declarations {
            if capability.id.trim().is_empty() {
                errors.push("capability_declarations[].id 不能为空".to_string());
                continue;
            }
            if capability.version.trim().is_empty() {
                errors.push(format!(
                    "capability_declarations[`{}`].version 不能为空",
                    capability.id
                ));
            }
            if !capability_ids.insert(capability.id.as_str()) {
                errors.push(format!("重复的能力合约 id `{}`", capability.id));
            }
            match PluginCapabilityContract::parse(&capability.id) {
                Some(contract) => {
                    if !self
                        .kind
                        .supported_capability_contracts()
                        .contains(&contract)
                    {
                        errors.push(format!(
                            "插件类型 `{}` 无法声明能力合约 `{}`",
                            self.kind.as_str(),
                            contract.as_str()
                        ));
                    }
                    if capability.version != PLUGIN_CAPABILITY_CONTRACT_V1_VERSION {
                        errors.push(format!(
                            "能力合约 `{}` 必须使用版本 `{}`",
                            capability.id, PLUGIN_CAPABILITY_CONTRACT_V1_VERSION
                        ));
                    }
                }
                None => errors.push(format!(
                    "不支持的能力合约 `{}`",
                    capability.id
                )),
            }
        }

        let mut dependency_ids = BTreeSet::new();
        for dependency in &self.dependencies {
            if dependency.plugin_id.trim().is_empty() {
                errors.push("dependencies[].plugin_id 不能为空".to_string());
            }
            if !dependency_ids.insert(dependency.plugin_id.as_str()) {
                errors.push(format!("重复的依赖 `{}`", dependency.plugin_id));
            }
        }

        if self.compatibility.core_ir_version.trim().is_empty() {
            errors.push("compatibility.core_ir_version 不能为空".to_string());
        }
        if self.compatibility.capability_api_version.trim().is_empty() {
            errors.push("compatibility.capability_api_version 不能为空".to_string());
        }
        if self.security.max_compute_ms == 0 {
            errors.push("security.max_compute_ms 必须大于 0".to_string());
        }
        if self.security.max_memory_mb == 0 {
            errors.push("security.max_memory_mb 必须大于 0".to_string());
        }
        // v2.1.2: enforce_max_* Some(0) 等同于 None，应拒绝
        if self.security.enforce_max_compute_ms == Some(0) {
            errors.push("security.enforce_max_compute_ms 为 Some(0) 无效，请使用 None 表示不限制".to_string());
        }
        if self.security.enforce_max_memory_mb == Some(0) {
            errors.push("security.enforce_max_memory_mb 为 Some(0) 无效，请使用 None 表示不限制".to_string());
        }

        // v1.0.0: 套件校验 — Suite 类型必须声明 atoms
        if self.plugin_type == Some(PluginType::Suite) && self.atoms.is_empty() {
            errors.push("套件插件必须声明 atoms 列表".to_string());
        }

        // v1.0.0: 热接管声明要求完整资产管理能力
        if self.hot_handoff && !self.asset_management {
            errors.push("声明 hot_handoff 的插件必须同时声明 asset_management".to_string());
        }

        // v2.1.0: entrypoint 校验 (非空 + 无路径遍历)
        if self.execution.entrypoint.trim().is_empty() {
            errors.push("execution.entrypoint 不能为空".to_string());
        } else if self.execution.entrypoint.contains("..")
            || self.execution.entrypoint.contains('/')
            || self.execution.entrypoint.contains('\\')
        {
            errors.push("execution.entrypoint 不能包含路径分隔符".to_string());
        }

        // v2.1.0: 校验每个依赖的 version_req 格式
        for dependency in &self.dependencies {
            if dependency.version_req.trim().is_empty() {
                errors.push(format!(
                    "dependencies[{}].version_req 不能为空",
                    dependency.plugin_id
                ));
            } else if !dependency.version_req.starts_with('>')
                && !dependency.version_req.starts_with('<')
                && !dependency.version_req.starts_with('=')
                && !dependency.version_req.starts_with('^')
                && !dependency.version_req.starts_with('~')
                && !dependency.version_req.chars().next().unwrap_or('.').is_ascii_digit()
            {
                errors.push(format!(
                    "dependencies[{}].version_req 格式无效: '{}'",
                    dependency.plugin_id, dependency.version_req
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Data,
    Intent,
    Agent,
    Risk,
    Execution,
}

impl PluginKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Data => "data",
            Self::Intent => "intent",
            Self::Agent => "agent",
            Self::Risk => "risk",
            Self::Execution => "execution",
        }
    }

    pub fn supported_extension_points(&self) -> &'static [ExtensionPoint] {
        match self {
            Self::Data => &[ExtensionPoint::DataModuleProvider],
            Self::Intent => &[ExtensionPoint::IntentModuleProvider],
            Self::Agent => &[ExtensionPoint::AgentModuleProvider],
            Self::Risk => &[ExtensionPoint::RiskCheckerProvider],
            Self::Execution => &[ExtensionPoint::ExecutionModuleProvider],
        }
    }

    pub fn supported_capability_contracts(&self) -> &'static [PluginCapabilityContract] {
        match self {
            Self::Data => &[PluginCapabilityContract::DataModuleProvider],
            Self::Intent => &[PluginCapabilityContract::IntentModuleProvider],
            Self::Agent => &[PluginCapabilityContract::AgentModuleProvider],
            Self::Risk => &[PluginCapabilityContract::RiskCheckerProvider],
            Self::Execution => &[PluginCapabilityContract::ExecutionModuleProvider],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginDisplay {
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCapabilityDeclaration {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionPoint {
    DataModuleProvider,
    IntentModuleProvider,
    AgentModuleProvider,
    RiskCheckerProvider,
    ExecutionModuleProvider,
}

impl ExtensionPoint {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DataModuleProvider => "data_module_provider",
            Self::IntentModuleProvider => "intent_module_provider",
            Self::AgentModuleProvider => "agent_module_provider",
            Self::RiskCheckerProvider => "risk_checker_provider",
            Self::ExecutionModuleProvider => "execution_module_provider",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapabilityContract {
    DataModuleProvider,
    IntentModuleProvider,
    AgentModuleProvider,
    RiskCheckerProvider,
    ExecutionModuleProvider,
}

impl PluginCapabilityContract {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DataModuleProvider => "quantpilot.capability.data_module_provider",
            Self::IntentModuleProvider => "quantpilot.capability.intent_module_provider",
            Self::AgentModuleProvider => "quantpilot.capability.agent_module_provider",
            Self::RiskCheckerProvider => "quantpilot.capability.risk_checker_provider",
            Self::ExecutionModuleProvider => "quantpilot.capability.execution_module_provider",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "quantpilot.capability.data_module_provider" => Some(Self::DataModuleProvider),
            "quantpilot.capability.intent_module_provider" => Some(Self::IntentModuleProvider),
            "quantpilot.capability.agent_module_provider" => Some(Self::AgentModuleProvider),
            "quantpilot.capability.risk_checker_provider" => Some(Self::RiskCheckerProvider),
            "quantpilot.capability.execution_module_provider" => {
                Some(Self::ExecutionModuleProvider)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginExecution {
    pub engine: PluginExecutionEngine,
    pub entrypoint: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginExecutionEngine {
    Builtin,
    QuantScript,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCompatibility {
    pub core_ir_version: String,
    pub capability_api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginSecurity {
    pub max_compute_ms: u64,
    pub max_memory_mb: u64,
    #[serde(default)]
    pub allow_network: bool,
    // v2.0.0: 硬限制 — 子进程沙箱实际执行, 覆盖 max_compute_ms
    #[serde(default)]
    pub enforce_max_compute_ms: Option<u64>,
    // v2.0.0: 硬限制 — 子进程沙箱实际执行, 覆盖 max_memory_mb
    #[serde(default)]
    pub enforce_max_memory_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_req: String,
}

#[derive(Debug, Default, Clone)]
pub struct PluginRegistry {
    manifests: BTreeMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn register(&mut self, manifest: PluginManifest) -> Result<(), Vec<String>> {
        manifest.validate()?;
        if self.manifests.contains_key(&manifest.id) {
            return Err(vec![format!(
                "插件 `{}` 已注册",
                manifest.id
            )]);
        }
        self.manifests.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn get(&self, plugin_id: &str) -> Option<&PluginManifest> {
        self.manifests.get(plugin_id)
    }

    // v2.1.0: 卸载插件时移除注册信息
    pub fn remove(&mut self, plugin_id: &str) -> Option<PluginManifest> {
        self.manifests.remove(plugin_id)
    }

    pub fn manifests_for_extension_point(
        &self,
        extension_point: ExtensionPoint,
    ) -> Vec<&PluginManifest> {
        self.manifests
            .values()
            .filter(|manifest| manifest.extension_points.contains(&extension_point))
            .collect()
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }
}

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
        assert!(errors
            .iter()
            .any(|item| item.contains("不支持的能力合约")));
    }

    #[test]
    fn rejects_mismatched_extension_point() {
        let mut manifest = sample_manifest();
        manifest.extension_points = vec![ExtensionPoint::ExecutionModuleProvider];
        let errors = manifest.validate().unwrap_err();
        assert!(errors
            .iter()
            .any(|item| item.contains("无法附加到扩展点")));
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
