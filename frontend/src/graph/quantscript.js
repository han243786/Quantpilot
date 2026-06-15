import { generateFormalQuantScript } from "./quantscriptFormal";
import { buildQuantScriptLabelTargets, buildQuantScriptRuntimeTargets } from "./quantscriptArtifactTargets";
import { generateGraphQuantScript, generateNodeQuantScript } from "./quantscriptGraphSource";
import { parseGraphQuantScriptSource } from "./quantscriptParser";

export { generateFormalQuantScript } from "./quantscriptFormal";
export { generateGraphQuantScript, generateNodeQuantScript } from "./quantscriptGraphSource";
export { parseGraphQuantScriptSource } from "./quantscriptParser";

export function attachQuantScriptArtifacts(graph) {
  const nodeScripts = Object.fromEntries(
    graph.nodes.map((node) => [node.id, generateNodeQuantScript(node, graph)])
  );
  const graphScript = generateGraphQuantScript(graph);
  const formalScript = generateFormalQuantScript(graph);
  const existingFormalSource = graph.metadata?.artifacts?.quantscript?.formal_source;
  const runtimeFormalSource =
    typeof existingFormalSource === "string" &&
    /^\s*v4_strategy\s+\S+\s*\{/m.test(existingFormalSource)
      ? existingFormalSource
      : formalScript;
  const labelTargets = buildQuantScriptLabelTargets(graph);
  const runtimeTargets = buildQuantScriptRuntimeTargets(graph);
  return {
    ...graph,
    metadata: {
      ...graph.metadata,
      source_mode: graph.metadata?.source_mode || "graph",
      artifacts: {
        ...(graph.metadata?.artifacts || {}),
        quantscript: {
          graph_source: graphScript,
          formal_source: runtimeFormalSource,
          node_sources: nodeScripts,
          label_targets: labelTargets,
          runtime_targets: runtimeTargets,
          generated_at: Date.now()
        }
      }
    }
  };
}

export function parseGraphQuantScript(source, registry, previousGraph = null) {
  return attachQuantScriptArtifacts(parseGraphQuantScriptSource(source, registry, previousGraph));
}
