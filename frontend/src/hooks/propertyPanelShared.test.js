import { describe, expect, it } from "vitest";
import {
  diagnosticSeverityCounts,
  findTargetRangeInSource,
  formatValue,
  strategyIrSourceFromGraph,
  stringifyJson
} from "./propertyPanelShared";

describe("propertyPanelShared", () => {
  it("formats compact scalar values for property rows", () => {
    expect(formatValue(null)).toBe("-");
    expect(formatValue(12)).toBe("12");
    expect(formatValue(12.34567)).toBe("12.3457");
    expect(formatValue("alpha")).toBe("alpha");
  });

  it("resolves strategy IR source from string, source, document, and versioned artifact shapes", () => {
    expect(strategyIrSourceFromGraph({ metadata: { artifacts: {} } })).toBe("");
    expect(
      strategyIrSourceFromGraph({
        metadata: {
          artifacts: {
            strategy_ir: "raw-ir"
          }
        }
      })
    ).toBe("raw-ir");
    expect(
      strategyIrSourceFromGraph({
        metadata: {
          artifacts: {
            strategy_ir: {
              source: "source-ir"
            }
          }
        }
      })
    ).toBe("source-ir");
    expect(
      strategyIrSourceFromGraph({
        metadata: {
          artifacts: {
            strategy_ir: {
              document: {
                name: "demo"
              }
            }
          }
        }
      })
    ).toBe(stringifyJson({ name: "demo" }));
    expect(
      strategyIrSourceFromGraph({
        metadata: {
          artifacts: {
            strategy_ir: {
              ir_version: "1",
              nodes: []
            }
          }
        }
      })
    ).toBe(stringifyJson({ ir_version: "1", nodes: [] }));
  });

  it("finds diagnostic target ranges from ordered search terms before fallbacks", () => {
    const source = '{"strategy":{"risk":{"limit":0.2}}}';

    expect(
      findTargetRangeInSource(source, {
        search_terms: ['"strategy"', '"risk"', '"limit"']
      })
    ).toEqual([21, 28]);
    expect(
      findTargetRangeInSource(source, {
        field: "strategy.risk.limit"
      })
    ).toEqual([21, 28]);
  });

  it("counts diagnostics by blocker, warning, and info severity", () => {
    expect(
      diagnosticSeverityCounts([
        { severity: "error" },
        { severity: "warning" },
        { severity: "info" },
        { severity: "blocker" }
      ])
    ).toEqual({
      blocker: 2,
      warning: 1,
      info: 1
    });
  });
});
