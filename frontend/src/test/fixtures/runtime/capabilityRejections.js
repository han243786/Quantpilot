export const backendCompileOkFixture = {
  compile_id: "compile_fixture_ok",
  protocol_name: "quantpilot/runtime-config/v1",
  config_hash: "test_config_hash",
  counts: {
    data_sources: 1,
    intent_generators: 2,
    agents: 1,
    risk_controls: 1,
    executions: 1
  },
  core_ir: {
    ir_version: "quantpilot/core-ir/v1",
    metadata: {
      strategy_id: "fixture_strategy",
      name: "Fixture Strategy",
      source_kind: "frontend_graph"
    }
  }
};

export const capabilityRejectionFixtures = {
  compileExecutionModuleUnsupported: {
    error: "capability_rejected",
    message:
      "Capability rejected: execution module builtin.execution.paper is not available for this backend profile.",
    details: {
      support_kind: "execution_module",
      key: "builtin.execution.paper",
      status: "unsupported"
    }
  },
  runtimeModeUnsupported: {
    error: "capability_rejected",
    message:
      "Capability rejected: runtime mode live is not enabled for this beta backend.",
    details: {
      support_kind: "runtime_mode",
      key: "live",
      status: "unsupported"
    }
  },
  symbolUnsupported: {
    error: "capability_rejected",
    message:
      "Capability rejected: symbol XRPUSDT is outside the current beta market-data profile.",
    details: {
      support_kind: "symbol",
      key: "XRPUSDT",
      status: "unsupported"
    }
  }
};
