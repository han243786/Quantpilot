import { describe, expect, it } from "vitest";
import {
  createGraphEdge,
  createInitialCompileSummary,
  createInitialValidationState
} from "./graphFactoryDefaults";

describe("graphFactoryDefaults", () => {
  it("creates edge records from source and target node ports", () => {
    const source = { id: "data_1", type: "data" };
    const target = { id: "intent_1", type: "intent" };

    expect(createGraphEdge(source, "market_data_out", target, "data_input")).toEqual({
      id: "edge_data_1_intent_1_market_data_out_data_input",
      source_node_id: "data_1",
      source_port: "market_data_out",
      target_node_id: "intent_1",
      target_port: "data_input",
      edge_type: "data_to_intent"
    });
  });

  it("creates fresh validation and compile default state objects", () => {
    const validation = createInitialValidationState();
    const secondValidation = createInitialValidationState();
    const compileSummary = createInitialCompileSummary();

    validation.issue_counts.error = 2;

    expect(secondValidation.issue_counts.error).toBe(0);
    expect(compileSummary.outputs).toEqual({
      data_sources: 0,
      intent_generators: 0,
      agents: 0,
      risk_controls: 0,
      executions: 0
    });
  });
});
