use anyhow::{bail, Result};
use qrpc_core::CoreStrategyIr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityVerdict {
    Compatible,
    CompatibleWithWarnings,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub verdict: CompatibilityVerdict,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
    pub migration_required: bool,
    pub migration_steps: Vec<String>,
    pub estimated_downtime_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSurface {
    pub module_key: String,
    pub module_kind: String,
    pub interface_version: String,
    pub core_ir_version: String,
    pub schema_version: String,
    pub input_schema_hash: Option<String>,
    pub output_schema_hash: Option<String>,
    pub state_schema: Option<String>,
    pub capability_hash: Option<String>,
    pub abi_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityContext {
    pub current_core_ir_version: String,
    pub current_capability_hash: String,
    pub runtime_mode: String,
}

#[derive(Default)]
pub struct CompatibilityChecker {
    strict_mode: bool,
}

impl CompatibilityChecker {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    pub fn check(
        &self,
        current: &ModuleSurface,
        candidate: &ModuleSurface,
        context: &CompatibilityContext,
    ) -> CompatibilityReport {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut migration_required = false;
        let mut migration_steps = Vec::new();

        // 1. Module identity check
        if current.module_key != candidate.module_key {
            violations.push(format!(
                "module_key mismatch: '{}' vs '{}'",
                current.module_key, candidate.module_key
            ));
            return CompatibilityReport {
                verdict: CompatibilityVerdict::Incompatible,
                violations,
                warnings,
                migration_required: false,
                migration_steps: Vec::new(),
                estimated_downtime_ms: None,
            };
        }

        if current.module_kind != candidate.module_kind {
            violations.push(format!(
                "module_kind changed from '{}' to '{}' — hot-swap across module kinds is not supported",
                current.module_kind, candidate.module_kind
            ));
        }

        // 2. Interface version compatibility
        if current.interface_version != candidate.interface_version {
            if self.strict_mode {
                violations.push(format!(
                    "interface_version changed from '{}' to '{}'",
                    current.interface_version, candidate.interface_version
                ));
            } else {
                warnings.push(format!(
                    "interface_version changed from '{}' to '{}' — ensure backward compatibility",
                    current.interface_version, candidate.interface_version
                ));
            }
        }

        // 3. Core IR version check
        if candidate.core_ir_version != context.current_core_ir_version {
            violations.push(format!(
                "candidate core_ir_version '{}' does not match runtime '{}'",
                candidate.core_ir_version, context.current_core_ir_version
            ));
        }

        // 4. Schema compatibility
        if let (Some(current_in), Some(candidate_in)) = (
            &current.input_schema_hash,
            &candidate.input_schema_hash,
        ) {
            if current_in != candidate_in {
                warnings.push(
                    "input schema hash changed — verify that new inputs are a superset".to_string(),
                );
            }
        }

        if let (Some(current_out), Some(candidate_out)) = (
            &current.output_schema_hash,
            &candidate.output_schema_hash,
        ) {
            if current_out != candidate_out {
                warnings.push(
                    "output schema hash changed — verify downstream consumers".to_string(),
                );
            }
        }

        // 5. State schema compatibility
        match (&current.state_schema, &candidate.state_schema) {
            (Some(current_state), Some(candidate_state)) if current_state != candidate_state => {
                migration_required = true;
                migration_steps.push(format!(
                    "state schema changed — migration needed from '{}' to '{}'",
                    current_state, candidate_state
                ));
                migration_steps
                    .push("capture pre-migration snapshot before activating".to_string());
                migration_steps.push(
                    "verify state migration with shadow replay before committing".to_string(),
                );
            }
            (None, Some(_)) => {
                migration_required = true;
                migration_steps
                    .push("candidate introduces state schema — initial state will be empty".to_string());
            }
            (Some(_), None) => {
                violations.push(
                    "candidate removes state schema — existing state would be orphaned".to_string(),
                );
            }
            _ => {}
        }

        // 6. Capability hash compatibility
        if let (Some(current_cap), Some(candidate_cap)) =
            (&current.capability_hash, &candidate.capability_hash)
        {
            if current_cap != candidate_cap
                && candidate_cap != context.current_capability_hash.as_str()
            {
                warnings.push(format!(
                    "candidate built against capability hash '{}', runtime is '{}'",
                    candidate_cap, context.current_capability_hash
                ));
            }
        }

        // 7. ABI constraints
        for constraint in &candidate.abi_constraints {
            if !current.abi_constraints.contains(constraint) {
                if constraint.contains("no-dynamic-unload") {
                    warnings.push(format!(
                        "candidate requires ABI constraint '{}' not present in current module",
                        constraint
                    ));
                } else {
                    violations.push(format!(
                        "candidate requires ABI constraint '{}' not present in current module",
                        constraint
                    ));
                }
            }
        }

        // Determine verdict
        let verdict = if !violations.is_empty() {
            CompatibilityVerdict::Incompatible
        } else if !warnings.is_empty() || migration_required {
            CompatibilityVerdict::CompatibleWithWarnings
        } else {
            CompatibilityVerdict::Compatible
        };

        CompatibilityReport {
            verdict,
            violations,
            warnings,
            migration_required,
            migration_steps,
            estimated_downtime_ms: if migration_required {
                Some(5_000)
            } else {
                Some(2_000)
            },
        }
    }

    pub fn validate_module_surface(
        surface: &ModuleSurface,
        core_ir: &CoreStrategyIr,
    ) -> Result<()> {
        if surface.module_key.trim().is_empty() {
            bail!("module_key 不能为空");
        }
        if surface.module_kind.trim().is_empty() {
            bail!("module_kind 不能为空");
        }
        if surface.interface_version.trim().is_empty() {
            bail!("interface_version 不能为空");
        }
        if surface.core_ir_version != core_ir.ir_version {
            bail!(
                "core_ir_version '{}' 与策略 IR 版本 '{}' 不匹配",
                surface.core_ir_version,
                core_ir.ir_version
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_module(kind: &str) -> ModuleSurface {
        ModuleSurface {
            module_key: format!("builtin.{kind}.test"),
            module_kind: kind.to_string(),
            interface_version: "v1".to_string(),
            core_ir_version: "quantpilot/core-ir/v1".to_string(),
            schema_version: "1.0.0".to_string(),
            input_schema_hash: Some("sha256:input-v1".to_string()),
            output_schema_hash: Some("sha256:output-v1".to_string()),
            state_schema: Some("state_v1".to_string()),
            capability_hash: Some("sha256:cap-v1".to_string()),
            abi_constraints: vec!["safe-window-required".to_string()],
        }
    }

    fn sample_context() -> CompatibilityContext {
        CompatibilityContext {
            current_core_ir_version: "quantpilot/core-ir/v1".to_string(),
            current_capability_hash: "sha256:cap-v1".to_string(),
            runtime_mode: "paper".to_string(),
        }
    }

    #[test]
    fn identical_modules_are_compatible() {
        let checker = CompatibilityChecker::new(true);
        let current = sample_module("intent");
        let candidate = sample_module("intent");
        let report = checker.check(&current, &candidate, &sample_context());
        assert_eq!(report.verdict, CompatibilityVerdict::Compatible);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn module_key_mismatch_is_incompatible() {
        let checker = CompatibilityChecker::new(true);
        let mut candidate = sample_module("intent");
        candidate.module_key = "builtin.intent.other".to_string();
        let report = checker.check(&sample_module("intent"), &candidate, &sample_context());
        assert_eq!(report.verdict, CompatibilityVerdict::Incompatible);
    }

    #[test]
    fn module_kind_change_is_incompatible() {
        let checker = CompatibilityChecker::new(true);
        let mut candidate = sample_module("intent");
        candidate.module_kind = "agent".to_string();
        let report = checker.check(&sample_module("intent"), &candidate, &sample_context());
        assert_eq!(report.verdict, CompatibilityVerdict::Incompatible);
    }

    #[test]
    fn state_schema_change_requires_migration() {
        let checker = CompatibilityChecker::new(false);
        let mut candidate = sample_module("intent");
        candidate.state_schema = Some("state_v2".to_string());
        let report = checker.check(&sample_module("intent"), &candidate, &sample_context());
        assert!(report.migration_required);
        assert!(!report.migration_steps.is_empty());
        assert_eq!(report.verdict, CompatibilityVerdict::CompatibleWithWarnings);
    }

    #[test]
    fn core_ir_version_mismatch_is_incompatible() {
        let checker = CompatibilityChecker::new(true);
        let mut candidate = sample_module("intent");
        candidate.core_ir_version = "quantpilot/core-ir/v2".to_string();
        let report = checker.check(&sample_module("intent"), &candidate, &sample_context());
        assert_eq!(report.verdict, CompatibilityVerdict::Incompatible);
    }

    #[test]
    fn interface_version_change_warns_in_non_strict() {
        let checker = CompatibilityChecker::new(false);
        let mut candidate = sample_module("intent");
        candidate.interface_version = "v2".to_string();
        let report = checker.check(&sample_module("intent"), &candidate, &sample_context());
        assert!(!report.warnings.is_empty());
        assert_eq!(report.verdict, CompatibilityVerdict::CompatibleWithWarnings);
    }

    #[test]
    fn removed_state_schema_is_incompatible() {
        let checker = CompatibilityChecker::new(true);
        let current = sample_module("intent");
        let mut candidate = sample_module("intent");
        candidate.state_schema = None;
        let report = checker.check(&current, &candidate, &sample_context());
        assert_eq!(report.verdict, CompatibilityVerdict::Incompatible);
    }

    #[test]
    fn validate_module_surface_rejects_empty_key() {
        let mut surface = sample_module("intent");
        surface.module_key = String::new();
        let core_ir = CoreStrategyIr {
            ir_version: "quantpilot/core-ir/v1".to_string(),
            metadata: qrpc_core_ir::CoreMetadata {
                strategy_id: "test".into(),
                name: "test".into(),
                source_kind: qrpc_core_ir::CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: Vec::new(),
            indicators: Vec::new(),
            signal_rules: Vec::new(),
            agent_policies: Vec::new(),
            risk_policies: Vec::new(),
            execution: qrpc_core_ir::ExecutionRule {
                execution_id: "exec".into(),
                venue_kind: "paper".into(),
                sizing_kind: qrpc_core_ir::ExecutionSizingKind::EquityNotionalRatio,
                slippage_bps: 5.0,
                taker_fee_bps: 10.0,
                total_cost_buffer_bps: 20.0,
                time_in_force: qrpc_core_ir::CoreTimeInForce::Gtc,
                params: BTreeMap::new(),
            },
        };
        assert!(CompatibilityChecker::validate_module_surface(&surface, &core_ir).is_err());
    }
}
