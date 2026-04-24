import { describe, expect, it } from "vitest";
import {
  CAPABILITY_ACTION_MAP,
  DECLARED_INDICATOR_KINDS,
  SUPPORT_MATRIX,
  SUPPORTED_EXCHANGES,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_INDICATOR_KINDS,
  SUPPORTED_RUNTIME_EXECUTION_MODULES,
  SUPPORTED_RUNTIME_MODES,
  SUPPORTED_SYMBOLS,
  WORKSPACE_SURFACE_MAP,
  getCapabilityActionBlockReason,
  isCapabilitySyncBlocked
} from "./supportMatrix";
import {
  DEFAULT_CAPABILITIES,
  applyCapabilitiesToModules,
  normalizeCapabilities
} from "../modules/builtinModules";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

describe("support matrix truth source", () => {
  it("keeps default frontend capabilities aligned with the support matrix", () => {
    expect(DEFAULT_CAPABILITIES.strategy_ir.declared_indicator_kinds).toEqual(
      DECLARED_INDICATOR_KINDS
    );
    expect(DEFAULT_CAPABILITIES.strategy_ir.supported_indicator_kinds).toEqual(
      SUPPORTED_INDICATOR_KINDS
    );
    expect(DEFAULT_CAPABILITIES.runtime.supported_modes).toEqual(SUPPORTED_RUNTIME_MODES);
    expect(DEFAULT_CAPABILITIES.runtime.supported_execution_modules).toEqual(
      SUPPORTED_RUNTIME_EXECUTION_MODULES
    );
    expect(DEFAULT_CAPABILITIES.market_data.supported_exchanges).toEqual(SUPPORTED_EXCHANGES);
    expect(DEFAULT_CAPABILITIES.market_data.supported_symbols).toEqual(SUPPORTED_SYMBOLS);
    expect(DEFAULT_CAPABILITIES.frontend.supported_module_keys).toEqual(
      SUPPORTED_FRONTEND_MODULE_KEYS
    );
  });

  it("keeps the backend capability fixture aligned with the support matrix", () => {
    const normalized = normalizeCapabilities(backendCapabilitiesFixture);

    expect(normalized.strategy_ir.declared_indicator_kinds).toEqual(
      SUPPORT_MATRIX.strategyIr.declaredIndicatorKinds
    );
    expect(normalized.strategy_ir.supported_indicator_kinds).toEqual(
      SUPPORT_MATRIX.strategyIr.supportedIndicatorKinds
    );
    expect(normalized.runtime.supported_modes).toEqual(SUPPORT_MATRIX.runtime.supportedModes);
    expect(normalized.runtime.supported_execution_modules).toEqual(
      SUPPORT_MATRIX.runtime.supportedExecutionModules
    );
    expect(normalized.market_data.supported_exchanges).toEqual(
      SUPPORT_MATRIX.runtime.marketBoundary.exchanges
    );
    expect(normalized.market_data.supported_symbols).toEqual(
      SUPPORT_MATRIX.runtime.marketBoundary.symbols
    );
    expect(normalized.frontend.supported_module_keys).toEqual(
      SUPPORT_MATRIX.frontend.supportedModuleKeys
    );
  });

  it("threads supported symbols into builtin data-module instrument options", () => {
    const modules = applyCapabilitiesToModules(backendCapabilitiesFixture);
    const dataModules = modules.filter(
      (moduleDef) =>
        moduleDef.module_key === "builtin.data.kline" ||
        moduleDef.module_key === "builtin.data.quote"
    );

    for (const moduleDef of dataModules) {
      const instrumentField = moduleDef.config_schema.fields.find((field) => field.key === "instrument");
      expect(instrumentField.options.map((option) => option.value)).toEqual(SUPPORTED_SYMBOLS);
    }
  });

  it("keeps weighted agent multi-symbol config fields available on the graph config surface", () => {
    const modules = applyCapabilitiesToModules(backendCapabilitiesFixture);
    const weightedAgent = modules.find((moduleDef) => moduleDef.module_key === "builtin.agent.weighted");
    const fieldKeys = weightedAgent.config_schema.fields.map((field) => field.key);

    expect(fieldKeys).toEqual(
      expect.arrayContaining([
        "rebalance_symbols",
        "rebalance_schedule",
        "rebalance_allocation_kind",
        "rebalance_rank_method",
        "rebalance_score_normalize",
        "rebalance_target_weights"
      ])
    );
  });

  it("documents capability-gated actions and API routes", () => {
    expect(CAPABILITY_ACTION_MAP.compile.apiPaths).toEqual([
      "/api/strategy-ir/compile",
      "/api/quantscript/formal/compile",
      "/api/runtime/compile"
    ]);
    expect(CAPABILITY_ACTION_MAP.start_simulation.apiPaths).toContain("/api/runtime/test-run");
    expect(CAPABILITY_ACTION_MAP.run_backtest.apiPaths).toContain("/api/runtime/backtest");
    expect(CAPABILITY_ACTION_MAP.run_parameter_sweep.apiPaths).toContain(
      "/api/runtime/experiments/backtest-sweep"
    );
    expect(CAPABILITY_ACTION_MAP.export_quantscript.apiPaths).toEqual([]);
  });

  it("classifies workspace-visible surfaces against their real source of truth", () => {
    expect(SUPPORT_MATRIX.workspace.surfaces).toEqual(WORKSPACE_SURFACE_MAP);
    expect(WORKSPACE_SURFACE_MAP.template_library.capabilityDriven).toBe(false);
    expect(WORKSPACE_SURFACE_MAP.version_history.capabilityDriven).toBe(false);
    expect(WORKSPACE_SURFACE_MAP.collaboration_audit.capabilityDriven).toBe(false);
    expect(WORKSPACE_SURFACE_MAP.parameter_sweep.capabilityDriven).toBe(true);
    expect(WORKSPACE_SURFACE_MAP.parameter_sweep.apiPaths).toContain(
      "/api/runtime/experiments/backtest-sweep"
    );
  });

  it("blocks risky actions only during capability sync and safe fallback", () => {
    expect(isCapabilitySyncBlocked("loading", "remote")).toBe(true);
    expect(isCapabilitySyncBlocked("error", "safe_fallback")).toBe(true);
    expect(isCapabilitySyncBlocked("degraded", "cache")).toBe(false);
    expect(isCapabilitySyncBlocked("ready", "remote")).toBe(false);
  });

  it("provides human-readable blocked reasons for capability-gated actions", () => {
    expect(
      getCapabilityActionBlockReason({
        actionKey: "compile",
        capabilityStatus: "loading",
        capabilitySource: "remote",
        capabilityMessage: ""
      })
    ).toContain("编译暂时锁定");

    expect(
      getCapabilityActionBlockReason({
        actionKey: "run_parameter_sweep",
        capabilityStatus: "error",
        capabilitySource: "safe_fallback",
        capabilityMessage: "能力校验失败。"
      })
    ).toContain("安全回退模式");

    expect(
      getCapabilityActionBlockReason({
        actionKey: "export_quantscript",
        capabilityStatus: "loading",
        capabilitySource: "remote",
        capabilityMessage: ""
      })
    ).toBe("");
  });
});
