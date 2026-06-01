import {
  CAPABILITY_ACTION_MAP,
  DECLARED_INDICATOR_KINDS,
  SUPPORTED_EXCHANGES,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_RUNTIME_EXECUTION_MODULES,
  SUPPORTED_RUNTIME_MODES,
  SUPPORTED_SYMBOLS,
  SUPPORT_MATRIX,
  WORKSPACE_SURFACE_MAP
} from "./supportMatrix.js";

import {
  CAPABILITY_CLASSES,
  CAPABILITY_OWNER_ROLES,
  DEFAULT_TEXT_GATE_ALLOWED_CONTEXT_PATTERN,
  buildCapabilityGovernanceEntry
} from "./capabilityGovernanceCore.js";

const runtimeModeEntries = SUPPORTED_RUNTIME_MODES.map((mode) =>
  buildCapabilityGovernanceEntry({
    id: `runtime.mode.${mode}`,
    family: "runtime_mode",
    value: mode,
    className: CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_runtime_owner,
    reviewResponsibility: "backend contract, compile/runtime checks",
    sourceOfTruth: "backend:/api/capabilities.runtime.supported_modes"
  })
);

const executionModuleEntries = SUPPORTED_RUNTIME_EXECUTION_MODULES.map((moduleKey) =>
  buildCapabilityGovernanceEntry({
    id: `execution.module.${moduleKey}`,
    family: "execution_module",
    value: moduleKey,
    className: CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_runtime_owner,
    reviewResponsibility: "execution semantics, capability response",
    sourceOfTruth: "backend:/api/capabilities.runtime.supported_execution_modules"
  })
);

const exchangeEntries = SUPPORTED_EXCHANGES.map((exchange) =>
  buildCapabilityGovernanceEntry({
    id: `market.exchange.${exchange}`,
    family: "exchange",
    value: exchange,
    className: CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_market_data_owner,
    reviewResponsibility: "market boundary, fixtures, wording",
    sourceOfTruth: "backend:/api/capabilities.market_data.supported_exchanges"
  })
);

const symbolEntries = SUPPORTED_SYMBOLS.map((symbol) =>
  buildCapabilityGovernanceEntry({
    id: `market.symbol.${symbol}`,
    family: "symbol",
    value: symbol,
    className: CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_market_data_owner,
    reviewResponsibility: "market boundary, fixtures, wording",
    sourceOfTruth: "backend:/api/capabilities.market_data.supported_symbols"
  })
);

const indicatorClassMap = {
  spread: CAPABILITY_CLASSES.restricted,
  custom: CAPABILITY_CLASSES.restricted
};

const indicatorNotesMap = {
  spread: [
    "Spread exists in the beta compile/runtime path but must not be marketed as research-grade spread strategy support."
  ],
  custom: [
    "Custom is limited to the restricted Strategy IR expression path that lowers into Core IR."
  ]
};

const indicatorEntries = DECLARED_INDICATOR_KINDS.map((kind) =>
  buildCapabilityGovernanceEntry({
    id: `strategy_ir.indicator.${kind}`,
    family: "strategy_ir_indicator_kind",
    value: kind,
    className: indicatorClassMap[kind] || CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_compile_owner,
    reviewResponsibility: "lowering boundary, diagnostics",
    sourceOfTruth: "backend:/api/capabilities.strategy_ir.declared_indicator_kinds",
    notes: indicatorNotesMap[kind] || []
  })
);

const frontendModuleClassMap = {
  "builtin.intent.spread_observer": CAPABILITY_CLASSES.restricted,
  "builtin.agent.arbitrage": CAPABILITY_CLASSES.trace_only
};

const frontendModuleNotesMap = {
  "builtin.intent.spread_observer": [
    "Spread-related module exposure is beta-only and must carry explicit boundary notes."
  ],
  "builtin.agent.arbitrage": [
    "该模块键可能在 Beta 代码路径中保持可见，但这并不代表真正的套利平台支持。"
  ]
};

const frontendModuleEntries = SUPPORTED_FRONTEND_MODULE_KEYS.map((moduleKey) =>
  buildCapabilityGovernanceEntry({
    id: `frontend.module.${moduleKey}`,
    family: "frontend_module",
    value: moduleKey,
    className: frontendModuleClassMap[moduleKey] || CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.frontend_editor_owner,
    reviewResponsibility: "sidebar exposure, disabled reasons, UX",
    sourceOfTruth: "backend:/api/capabilities.frontend.module_support",
    notes: frontendModuleNotesMap[moduleKey] || []
  })
);

const actionEntries = Object.entries(CAPABILITY_ACTION_MAP).map(([actionKey, action]) =>
  buildCapabilityGovernanceEntry({
    id: `ui.action.${actionKey}`,
    family: "ui_action",
    value: actionKey,
    className: CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.frontend_editor_owner,
    reviewResponsibility: "action gating, reason text, E2E",
    sourceOfTruth: "backend:/api/capabilities.ui_actions.actions",
    notes: action.notes || []
  })
);

const workspaceSurfaceClassMap = {
  parameter_sweep: CAPABILITY_CLASSES.restricted
};

const workspaceSurfaceEntries = Object.entries(WORKSPACE_SURFACE_MAP).map(([surfaceKey, surface]) =>
  buildCapabilityGovernanceEntry({
    id: `workspace.surface.${surfaceKey}`,
    family: "workspace_surface",
    value: surfaceKey,
    className: workspaceSurfaceClassMap[surfaceKey] || CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.frontend_editor_owner,
    reviewResponsibility: "workspace exposure, backend route honesty, closeout audit",
    sourceOfTruth: "backend:/api/capabilities.workspace.surfaces",
    notes: surface.notes || []
  })
);

const compileBoundaryEntries = [
  buildCapabilityGovernanceEntry({
    id: "compile.strategy_ir_preflight",
    family: "compile_boundary",
    value: "strategy_ir",
    className: CAPABILITY_CLASSES.restricted,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_compile_owner,
    reviewResponsibility: "lowering boundary, diagnostics",
    sourceOfTruth: "frontend:support-matrix.compile.preflightArtifact",
    notes: ["Semantic preflight only. It does not decide runnable output."]
  }),
  buildCapabilityGovernanceEntry({
    id: "compile.formal_quantscript_lowering",
    family: "compile_boundary",
    value: "quantscript.formal_source",
    className: CAPABILITY_CLASSES.restricted,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_compile_owner,
    reviewResponsibility: "lowering boundary, diagnostics",
    sourceOfTruth: "frontend:support-matrix.compile.boundaryNotes",
    notes: ["Owns runtime lowering when present, but runtime compile still decides runnable output."]
  }),
  buildCapabilityGovernanceEntry({
    id: "compile.runtime_source_of_truth",
    family: "compile_boundary",
    value: SUPPORT_MATRIX.compile.runtimeSourceOfTruth,
    className: CAPABILITY_CLASSES.supported,
    ownerRole: CAPABILITY_OWNER_ROLES.backend_compile_owner,
    reviewResponsibility: "backend contract, compile/runtime checks",
    sourceOfTruth: "frontend:support-matrix.compile.runtimeSourceOfTruth",
    notes: ["When artifacts disagree, runtime behavior follows this source of truth."]
  })
];

const claimEntries = [
  ...SUPPORT_MATRIX.userFacingGuardrails.allowedClaims.map((claimText) =>
    buildCapabilityGovernanceEntry({
      id: `claim.allowed.${claimText.replace(/\s+/g, "_")}`,
      family: "user_facing_claim",
      value: claimText,
      className: CAPABILITY_CLASSES.supported,
      ownerRole: CAPABILITY_OWNER_ROLES.docs_and_qa_owner,
      reviewResponsibility: "README, markdown, UI copy, text gates",
      sourceOfTruth: "frontend:support-matrix.userFacingGuardrails.allowedClaims",
      textGate: {
        approvedPhrase: claimText
      }
    })
  ),
  buildCapabilityGovernanceEntry({
    id: "claim.disallowed.claiming_research-grade_backtest_support",
    family: "user_facing_claim",
    value: SUPPORT_MATRIX.userFacingGuardrails.disallowedClaims[0],
    className: CAPABILITY_CLASSES.disallowed_claim,
    ownerRole: CAPABILITY_OWNER_ROLES.docs_and_qa_owner,
    reviewResponsibility: "README, markdown, UI copy, text gates",
    sourceOfTruth: "frontend:support-matrix.userFacingGuardrails.disallowedClaims",
    textGate: {
      forbiddenPattern: "research-grade backtest is supported",
      detail: "Do not describe research-grade backtest as a currently supported capability.",
      allowedContextPattern: DEFAULT_TEXT_GATE_ALLOWED_CONTEXT_PATTERN
    }
  }),
  buildCapabilityGovernanceEntry({
    id: "claim.disallowed.claiming_live_trading_support",
    family: "user_facing_claim",
    value: SUPPORT_MATRIX.userFacingGuardrails.disallowedClaims[1],
    className: CAPABILITY_CLASSES.disallowed_claim,
    ownerRole: CAPABILITY_OWNER_ROLES.docs_and_qa_owner,
    reviewResponsibility: "README, markdown, UI copy, text gates",
    sourceOfTruth: "frontend:support-matrix.userFacingGuardrails.disallowedClaims",
    textGate: {
      forbiddenPattern: "live trading is supported",
      detail: "Do not describe live trading as a currently supported capability.",
      allowedContextPattern: DEFAULT_TEXT_GATE_ALLOWED_CONTEXT_PATTERN
    }
  }),
  buildCapabilityGovernanceEntry({
    id: "claim.disallowed.claiming_true_arbitrage_agent_support",
    family: "user_facing_claim",
    value: SUPPORT_MATRIX.userFacingGuardrails.disallowedClaims[2],
    className: CAPABILITY_CLASSES.disallowed_claim,
    ownerRole: CAPABILITY_OWNER_ROLES.docs_and_qa_owner,
    reviewResponsibility: "README, markdown, UI copy, text gates",
    sourceOfTruth: "frontend:support-matrix.userFacingGuardrails.disallowedClaims",
    textGate: {
      forbiddenPattern: "true arbitrage agent support is supported",
      detail: "Do not describe true arbitrage agent support as a currently supported capability.",
      allowedContextPattern: DEFAULT_TEXT_GATE_ALLOWED_CONTEXT_PATTERN
    }
  }),
  buildCapabilityGovernanceEntry({
    id: "claim.disallowed.claiming_third-party_plugin_marketplace_support",
    family: "user_facing_claim",
    value: SUPPORT_MATRIX.userFacingGuardrails.disallowedClaims[3],
    className: CAPABILITY_CLASSES.disallowed_claim,
    ownerRole: CAPABILITY_OWNER_ROLES.docs_and_qa_owner,
    reviewResponsibility: "README, markdown, UI copy, text gates",
    sourceOfTruth: "frontend:support-matrix.userFacingGuardrails.disallowedClaims",
    textGate: {
      forbiddenPattern: "third-party plugin marketplace is supported",
      detail: "Do not describe a third-party plugin marketplace as a currently supported capability.",
      allowedContextPattern: DEFAULT_TEXT_GATE_ALLOWED_CONTEXT_PATTERN
    }
  })
];

export const CAPABILITY_GOVERNANCE_REGISTRY = [
  ...runtimeModeEntries,
  ...executionModuleEntries,
  ...exchangeEntries,
  ...symbolEntries,
  ...indicatorEntries,
  ...frontendModuleEntries,
  ...actionEntries,
  ...workspaceSurfaceEntries,
  ...compileBoundaryEntries,
  ...claimEntries
];
