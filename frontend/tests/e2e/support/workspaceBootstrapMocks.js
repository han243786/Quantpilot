import { buildWorkspaceGraphFixture } from "./workspaceGraphFixture";

function buildGraphIndexFixture(graphFixture) {
  return [
    {
      graph_id: graphFixture.metadata?.graph_id || "draft_graph",
      name: graphFixture.metadata?.name || "Draft graph",
      updated_at: graphFixture.metadata?.updated_at || Date.now(),
      path: `storage/graphs/${graphFixture.metadata?.graph_id || "draft_graph"}.json`
    }
  ];
}

function buildStrategyConfigPreflightFixture(request = {}) {
  const strategyId = request.strategy_id || "draft_graph";
  const runtimeMode = request.runtime_mode || "PaperSimulated";
  const capabilitySnapshotHash = request.capability_snapshot_hash || "safe-fallback";
  const proposalBindings = Array.isArray(request.proposal_bindings)
    ? request.proposal_bindings
    : [];
  const evidenceAnchors = Array.isArray(request.evidence_anchors)
    ? request.evidence_anchors
    : [];

  const artifact = {
    schema_version: "quantpilot/v4-strategy-config-artifact/v1",
    artifact_id: `artifact_${strategyId}`,
    strategy_id: strategyId,
    artifact_digest: `sha256:e2e_${strategyId}`,
    generated_at_ms: 1_700_000_000_000,
    source: {
      graph_digest: `sha256:e2e_graph_${strategyId}`,
      runtime_config_digest: null,
      qs_digest: null,
      core_ir_digest: null,
      v4_graph_digest: null
    },
    capability: {
      capability_snapshot_hash: capabilitySnapshotHash,
      capability_snapshot_status: "current",
      capability_source: request.capability_source || "frontend_snapshot"
    },
    runtime_boundary: {
      mode_label: runtimeMode,
      live_execution_allowed: false,
      provider_order_submission_allowed: false,
      execution_capability_sources:
        request.required_execution_capability_sources || ["runtime_simulated"],
      rejection_reasons: []
    },
    config_domains: [
      {
        domain_id: "state_machine",
        lifecycle: "implemented",
        readiness: "ready",
        source_refs: [
          {
            source_kind: "v4_graph",
            source_id: strategyId,
            digest: `sha256:e2e_state_machine_${strategyId}`
          }
        ],
        findings: []
      },
      {
        domain_id: "risk",
        lifecycle: "implemented",
        readiness: "ready",
        source_refs: [],
        findings: [],
        primary_action: "preflight"
      }
    ],
    evidence_anchors: evidenceAnchors,
    proposal_bindings: proposalBindings
  };

  return {
    schema_version: "quantpilot/v4-strategy-config-preflight/v1",
    artifact,
    decision: "ready",
    can_compile: true,
    can_paper_simulated: true,
    can_backtest: true,
    can_paper_actual_demo: false,
    can_live_execution: false,
    allowed_actions: ["compile", "start_paper_simulated", "run_backtest"],
    blocked_actions: [
      {
        action: "live_execution",
        reason: "live_execution_allowed=false"
      }
    ],
    findings: []
  };
}

export async function installWorkspaceBootstrapMocks(
  api,
  {
    graphFixture = buildWorkspaceGraphFixture(),
    latestGraphResponse = graphFixture,
    runHistory = [],
    backtestHistory = [],
    experiments = []
  } = {}
) {
  const graphId = graphFixture.metadata?.graph_id || "draft_graph";

  if (
    latestGraphResponse &&
    typeof latestGraphResponse === "object" &&
    ("status" in latestGraphResponse || "body" in latestGraphResponse)
  ) {
    await api.fulfill("**/api/graphs/latest", latestGraphResponse);
  } else {
    await api.json("**/api/graphs/latest", latestGraphResponse);
  }
  await api.json("**/api/graphs", buildGraphIndexFixture(graphFixture));
  await api.json(`**/api/graphs/${graphId}`, graphFixture);
  await api.json("**/api/runtime/runs", runHistory);
  await api.json("**/api/runtime/backtests", backtestHistory);
  await api.json("**/api/runtime/mutations**", []);
  await api.json("**/api/runtime/reports**", []);
  await api.json("**/api/runtime/experiments", experiments);
  await api.json("**/api/runtime/experiments/*", {
    experiment_id: "",
    graph_id: graphId,
    variants: []
  });
  await api.json("**/api/graphs/*/versions", []);
  await api.json("**/api/graphs/*/audit", []);
  await api.handle("**/api/v1/strategy-config/preflight", async (route) => {
    let request = {};
    try {
      request = route.request().postDataJSON();
    } catch {
      request = {};
    }
    await route.fulfill({
      status: 200,
      contentType: "application/json; charset=utf-8",
      body: JSON.stringify(buildStrategyConfigPreflightFixture(request))
    });
  });

  return graphFixture;
}
