import { attachQuantScriptArtifacts, generateGraphQuantScript } from "./quantscript";
import { buildCoreIr } from "./compileGraphCoreIr";
import { buildRuntimeConfig } from "./compileGraphRuntimeConfig";
import { buildLocalCompileDiagnostics } from "./compileGraphSupport";
import { appendGraphCompileDiagnostics, buildTopology } from "./compileGraphTopology";

export function compileGraph(graph, registry = null) {
  const { compileId, output, errors, warnings } = buildRuntimeConfig(graph, registry);
  const topology = buildTopology(graph);
  appendGraphCompileDiagnostics({ graph, topology, errors });

  const graphWithArtifacts = attachQuantScriptArtifacts(graph);
  const coreIr = buildCoreIr(graph, output);
  graphWithArtifacts.metadata.artifacts = {
    ...(graphWithArtifacts.metadata.artifacts || {}),
    core_ir: coreIr
  };
  const quantscript =
    graphWithArtifacts.metadata.artifacts.quantscript.graph_source ||
    generateGraphQuantScript(graph);
  const compilable = errors.length === 0;
  const diagnostics = buildLocalCompileDiagnostics(errors, warnings);

  return {
    compile_id: compileId,
    runtime_config: output,
    core_ir: coreIr,
    quantscript,
    graph: graphWithArtifacts,
    compile_summary: {
      compilable,
      last_compile_id: compileId,
      last_compile_at: Date.now(),
      topology_order: topology.topologyOrder,
      outputs: {
        data_sources: output.data_sources.length,
        intent_generators: output.intent_generators.length,
        agents: output.agents.length,
        risk_controls: output.risk_controls.length,
        executions: output.executions.length
      },
      diagnostics,
      warnings,
      errors
    }
  };
}
