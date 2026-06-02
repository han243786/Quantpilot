import { describe, expect, it } from "vitest";
import {
  agentUsesPortfolioRebalance,
  buildLocalCompileDiagnostics,
  capabilityEntryStatus,
  capabilityReason,
  capabilitySet,
  jsonValue,
  normalizeRebalanceAllocationKind,
  normalizeRebalanceRankMethod,
  normalizeRebalanceSchedule,
  normalizeRebalanceScoreNormalize,
  parseCsvNumbers,
  parseCsvStrings,
  supportMap
} from "./compileGraphSupport";

describe("compileGraphSupport", () => {
  it("builds capability support indexes and status fallbacks", () => {
    const fallback = capabilitySet([], ["paper"]);
    const map = supportMap([{ key: "paper", status: "unsupported", reason: "disabled" }]);

    expect(fallback.has("paper")).toBe(true);
    expect(map.get("paper").status).toBe("unsupported");
    expect(capabilityEntryStatus(map.get("paper"), fallback, "paper")).toBe(false);
    expect(capabilityEntryStatus(null, fallback, "paper")).toBe(true);
    expect(capabilityReason(map.get("paper"), "")).toBe("disabled");
  });

  it("normalizes optional CSV and rebalance config values", () => {
    expect(parseCsvStrings(" BTCUSDT, ETHUSDT ,, SOLUSDT ")).toEqual([
      "BTCUSDT",
      "ETHUSDT",
      "SOLUSDT"
    ]);
    expect(parseCsvNumbers("0.5, bad, 0.25")).toEqual([0.5, 0.25]);
    expect(jsonValue(undefined)).toBeNull();
    expect(jsonValue(0)).toBe(0);

    expect(normalizeRebalanceSchedule(" weekly ")).toBe("weekly");
    expect(normalizeRebalanceSchedule("never")).toBe("__invalid__");
    expect(normalizeRebalanceAllocationKind("fixed_weights")).toBe("fixed_weights");
    expect(normalizeRebalanceRankMethod("inverse_rank")).toBe("inverse_rank");
    expect(normalizeRebalanceScoreNormalize("sum")).toBe("sum");
    expect(agentUsesPortfolioRebalance({ rebalance_symbols: "BTCUSDT" })).toBe(true);
    expect(agentUsesPortfolioRebalance({})).toBe(false);
  });

  it("creates deterministic local compile diagnostics", () => {
    expect(buildLocalCompileDiagnostics(["broken"], ["heads up"])).toEqual([
      {
        source: "graph",
        code: "GRAPH_COMPILE_ERROR",
        severity: "error",
        message: "broken",
        target: null,
        hint: null
      },
      {
        source: "graph",
        code: "GRAPH_COMPILE_WARNING",
        severity: "warning",
        message: "heads up",
        target: null,
        hint: null
      }
    ]);
  });
});
