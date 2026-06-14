import { describe, expect, it } from "vitest";
import { createNodeFromModule } from "./createNode";

const moduleDef = {
  category: "data",
  module_key: "builtin.data.test",
  node: {
    default_name: "Test Data"
  },
  config_schema: {
    fields: [
      { key: "exchange", default: "okx" },
      { key: "instrument", default: "BTCUSDT" }
    ]
  },
  ports: {
    inputs: [],
    outputs: [{ key: "market_data_out", provides: "market_data" }]
  }
};

describe("createNodeFromModule", () => {
  it("creates graph nodes from module definitions with default config and runtime state", () => {
    const node = createNodeFromModule(moduleDef);

    expect(node.id).toMatch(/^node_data_\d+$/);
    expect(node.type).toBe("data");
    expect(node.module_key).toBe("builtin.data.test");
    expect(node.name).toBe("Test Data");
    expect(node.config).toEqual({
      exchange: "okx",
      instrument: "BTCUSDT"
    });
    expect(node.output_ports).toEqual([{ key: "market_data_out", provides: "market_data" }]);
    expect(node.runtime_state).toEqual({
      status: "idle",
      last_event_type: null,
      last_event_time: null,
      last_message: "",
      metrics: {},
      error: null
    });
  });
});
