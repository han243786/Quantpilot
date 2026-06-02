import { describe, expect, it } from "vitest";
import { DEFAULT_CAPABILITIES } from "../modules/builtinModules";
import {
  buildCapabilityIndex,
  buildIssue,
  capabilityEntryStatus,
  capabilityReason,
  capabilitySet,
  compareValues,
  supportMap
} from "./validationSupport";

describe("validationSupport", () => {
  it("builds capability indexes with fallback sets and support maps", () => {
    const index = buildCapabilityIndex({ capabilities: DEFAULT_CAPABILITIES });

    expect(index.supportedRuntimeModes.has("paper")).toBe(true);
    expect(index.supportedExecutionModules.has("builtin.execution.paper")).toBe(true);
    expect(index.supportedSymbols.has("BTCUSDT")).toBe(true);
    expect(index.supportedExchanges.has("okx")).toBe(true);
    expect(index.frontendModuleSupport).toBeInstanceOf(Map);
  });

  it("resolves explicit support entries before fallback sets", () => {
    const fallback = capabilitySet(["paper"], []);

    expect(capabilityEntryStatus({ status: "unsupported" }, fallback, "paper")).toBe(false);
    expect(capabilityEntryStatus(null, fallback, "paper")).toBe(true);
    expect(capabilityEntryStatus(null, fallback, "live")).toBe(false);
    expect(capabilityReason({ reason: "Backend disabled this path." }, "")).toBe(
      "Backend disabled this path."
    );
  });

  it("keeps support maps, value comparisons, and issue records deterministic", () => {
    const map = supportMap([{ module_key: "builtin.execution.paper", status: "supported" }], "module_key");
    const issue = buildIssue("error", "node", "node_1", "FIELD_REQUIRED", "Required.");

    expect(map.get("builtin.execution.paper").status).toBe("supported");
    expect(compareValues(1, "<=", 2)).toBe(true);
    expect(compareValues(2, ">", 3)).toBe(false);
    expect(compareValues(2, "unknown", 3)).toBe(true);
    expect(issue).toEqual({
      id: "node_node_1_FIELD_REQUIRED",
      level: "error",
      scope: "node",
      target_id: "node_1",
      code: "FIELD_REQUIRED",
      message: "Required.",
      hint: ""
    });
  });
});
