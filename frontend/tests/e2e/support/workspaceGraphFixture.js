import { compileGraph } from "../../../src/graph/compileGraph";
import { DEFAULT_CAPABILITIES, applyCapabilitiesToModules } from "../../../src/modules/builtinModules";
import { createModuleRegistry } from "../../../src/modules/moduleRegistry";
import { buildValidatedSampleGraph } from "../../../src/test/fixtures/runtime/buildValidatedSampleGraph";

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
      name: "OKX 双均线趋势策略图",
      runtime_binding: {
        current_run_id: null,
        last_compile_id: null
      }
    }
  };
}
