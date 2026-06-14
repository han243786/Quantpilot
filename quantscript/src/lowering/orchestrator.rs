use crate::script::ScriptModule;
use anyhow::Result;
use qrpc_core::RuntimeProtocolCoreConfig;

use super::context::LoweringContext;

mod entrypoint_runtime_config_assembly;

pub fn lower_script_to_runtime_config(module: &ScriptModule) -> Result<RuntimeProtocolCoreConfig> {
    lower_script_to_runtime_config_with_context(module, &LoweringContext::default())
}

pub fn lower_script_to_runtime_config_with_context(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<RuntimeProtocolCoreConfig> {
    entrypoint_runtime_config_assembly::lower_script_to_runtime_config_with_context(module, context)
}

#[cfg(test)]
mod integration_test_harness;
