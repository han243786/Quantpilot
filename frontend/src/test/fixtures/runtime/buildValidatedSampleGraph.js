import { createSampleGraph } from "../../../graph/createGraph";
import { validateGraph } from "../../../graph/validation";
import { attachQuantScriptArtifacts } from "../../../graph/quantscript";

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

export function buildValidatedSampleGraph(registry, mutate = null) {
  const graph = clone(createSampleGraph(registry));
  mutate?.(graph);
  const normalized = attachQuantScriptArtifacts(graph);
  return {
    ...normalized,
    validation_state: validateGraph(normalized, registry)
  };
}
