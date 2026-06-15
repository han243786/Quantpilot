import { compileGraph } from "../../../src/graph/compileGraph";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../../../src/modules/builtinModules";
import { createModuleRegistry } from "../../../src/modules/moduleRegistry";
import { buildValidatedSampleGraph } from "../../../src/test/fixtures/runtime/buildValidatedSampleGraph";

export const WORKSPACE_V4_RUNTIME_SOURCE = `
v4_strategy strategy.v4.e2e_workspace {
  venue paper-local
  mode paper_simulated
  require capability market

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> risk.guard on bar_closed
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
`;

function buildRegistry() {
  return createModuleRegistry(
    applyCapabilitiesToModules(DEFAULT_CAPABILITIES),
    DEFAULT_CAPABILITIES
  );
}

export function buildWorkspaceGraphFixture() {
  const registry = buildRegistry();
  const validatedGraph = buildValidatedSampleGraph(registry, (graph) => {
    graph.metadata.graph_id = "draft_graph";
    graph.metadata.updated_at = Date.now();
  });
  const compiled = compileGraph(validatedGraph, registry);

  return {
    ...compiled.graph,
    metadata: {
      ...compiled.graph.metadata,
      graph_id: "draft_graph",
      runtime_kind: "v4",
      name: "OKX 双均线趋势策略图",
      runtime_binding: {
        current_run_id: null,
        last_compile_id: null
      },
      artifacts: {
        ...(compiled.graph.metadata.artifacts || {}),
        quantscript: {
          ...(compiled.graph.metadata.artifacts?.quantscript || {}),
          formal_source: WORKSPACE_V4_RUNTIME_SOURCE
        }
      }
    }
  };
}
