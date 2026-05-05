import { postJson } from "./graphStorePersistenceHelpers";

export function requestStrategyIrCompile(graphId, compileId, strategyIr) {
  return postJson("/strategy-ir/compile", {
    graph_id: graphId,
    compile_id: compileId,
    strategy_ir: strategyIr
  });
}

export function requestFormalQuantScriptCompile({
  graphId,
  compileId,
  source,
  runtimeTemplate,
  runtimeTargets
}) {
  return postJson("/quantscript/formal/compile", {
    graph_id: graphId,
    compile_id: compileId,
    source,
    runtime_template: runtimeTemplate,
    runtime_targets: runtimeTargets
  });
}

export function requestRuntimeCompile(runtimeConfig, graphJson) {
  return postJson("/runtime/compile", {
    runtime_config: runtimeConfig,
    graph_json: graphJson
  });
}
