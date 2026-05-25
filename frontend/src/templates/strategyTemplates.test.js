import { describe, expect, it } from "vitest";

import { builtinModules } from "../modules/builtinModules";
import { createModuleRegistry } from "../modules/moduleRegistry";
import { buildStrategyTemplateGraph } from "./strategyTemplates";

describe("strategyTemplates v4 adapter boundary", () => {
  it("keeps v4 runtime templates provider-agnostic outside data and execution edges", () => {
    const registry = createModuleRegistry(builtinModules);
    const graph = buildStrategyTemplateGraph("dual_ma_v4", registry);
    const v4Graph = graph.metadata.artifacts.v4_machine_graph;
    const observation = v4Graph.machines.find((machine) => machine.template === "observation");
    const execution = v4Graph.machines.find((machine) => machine.template === "execution");
    const marketEvent = v4Graph.event_catalog.events.find(
      (event) => event.source_kind === "market_data"
    );

    expect(v4Graph.metadata.default_venue_id).toBe("paper-simulated");
    expect(v4Graph.metadata.market_event_source).toBe("market.data");
    expect(marketEvent.allowed_emitters).toEqual(["market.data"]);
    expect(observation.transitions[0].event.source).toBe("market.data");
    expect(execution.metadata.core_venue_kind).toBe("paper-simulated");

    const serialized = JSON.stringify(v4Graph);
    expect(serialized).not.toContain(["market", "okx"].join("."));
    expect(serialized).not.toContain(["okx", "paper"].join("-"));
  });
});
