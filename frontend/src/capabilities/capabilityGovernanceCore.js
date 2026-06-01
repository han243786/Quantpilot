export const CAPABILITY_GOVERNANCE_SCHEMA_VERSION = "quantpilot/capability-governance/v1";

export const CAPABILITY_CLASSES = {
  supported: "supported",
  restricted: "restricted",
  trace_only: "trace_only",
  disallowed_claim: "disallowed_claim"
};

export const CAPABILITY_OWNER_ROLES = {
  backend_runtime_owner: "backend runtime owner",
  backend_market_data_owner: "backend market-data owner",
  backend_compile_owner: "backend compile owner",
  frontend_editor_owner: "frontend editor owner",
  docs_and_qa_owner: "docs and QA owner"
};

export const DEFAULT_TEXT_GATE_ALLOWED_CONTEXT_PATTERN =
  "must not|must not appear|disallowedClaims|forbiddenPattern|textGate";

export const CAPABILITY_TEXT_GATES = {
  positiveClaimAudit: {
    scopedPaths: [
      "README.md",
      "frontend/src/components/TopToolbar.jsx",
      "frontend/src/components/PropertyPanel.jsx",
      "frontend/src/components/EventStreamPanel.jsx",
      "frontend/src/components/ModuleSidebar.jsx",
      "frontend/src/pages/EditorPage.jsx",
      "frontend/src/pages/BacktestDetailPage.jsx",
      "frontend/src/pages/BacktestComparePage.jsx"
    ],
    positiveStatementPatterns: [
      "\\b(?:is|are)\\s+(?:currently\\s+)?supported\\b",
      "\\bcurrently\\s+supported\\b",
      "\\bsupports\\b",
      "\\bsupported\\s+(?:runtime|mode|execution|exchange|symbol|indicator|capability|path)\\b",
      "\\b(?:runtime|backtest|execution|market-data|plugin|arbitrage|spread)\\s+support\\b"
    ],
    allowedContextPattern:
      "must not|must not appear|disallowedClaims|allowedClaims|support matrix|capability governance"
  }
};

export function buildCapabilityGovernanceEntry({
  id,
  family,
  value,
  className,
  ownerRole,
  reviewResponsibility,
  sourceOfTruth,
  notes = [],
  textGate = null
}) {
  return {
    id,
    family,
    value,
    class: className,
    ownerRole,
    reviewResponsibility,
    sourceOfTruth,
    notes,
    textGate
  };
}
