import { describe, expect, it } from "vitest";
import {
  deriveConfigureCardOrder,
  derivePriorityFieldGroups,
  resolveConfigureIssueTargetCard
} from "./configureFieldPriority";

describe("derivePriorityFieldGroups", () => {
  it("pins issue-specific fields before the broader node defaults", () => {
    const moduleDef = {
      node: {
        quick_fields: ["fast_period", "slow_period"],
        summary_fields: ["fast_period", "slow_period", "entry_ratio"]
      },
      config_schema: {
        fields: [
          { key: "fast_period", label: "Fast period" },
          { key: "slow_period", label: "Slow period" },
          { key: "entry_ratio", label: "Entry ratio" }
        ]
      }
    };

    const groups = derivePriorityFieldGroups({
      moduleDef,
      nodeType: "intent",
      prioritizePathFields: true,
      nodeIssues: [
        {
          code: "FIELD_REQUIRED",
          message: "You must fill Fast period."
        }
      ]
    });

    expect(groups[0].title).toBe("优先字段");
    expect(groups[0].summary).toContain("当前节点问题");
    expect(groups[0].fields.map((field) => field.key)).toEqual([
      "fast_period",
      "slow_period",
      "entry_ratio"
    ]);
  });

  it("uses issue-code mappings for data capability failures", () => {
    const moduleDef = {
      node: {
        quick_fields: ["instrument", "timeframe"],
        summary_fields: ["exchange", "instrument", "timeframe", "window_size"]
      },
      config_schema: {
        fields: [
          { key: "exchange", label: "Exchange" },
          { key: "instrument", label: "Instrument" },
          { key: "timeframe", label: "Timeframe" },
          { key: "window_size", label: "Window size" }
        ]
      }
    };

    const groups = derivePriorityFieldGroups({
      moduleDef,
      nodeType: "data",
      prioritizePathFields: true,
      nodeIssues: [
        {
          code: "UNSUPPORTED_SYMBOL",
          message: "The selected instrument is not supported."
        }
      ]
    });

    expect(groups).toHaveLength(1);
    expect(groups[0].fields[0].key).toBe("instrument");
    expect(groups[0].summary).toContain("当前节点问题");
  });

  it("keeps a single all-settings group when path prioritization is disabled", () => {
    const moduleDef = {
      node: {
        quick_fields: ["mode"],
        summary_fields: ["mode", "initial_cash"]
      },
      config_schema: {
        fields: [
          { key: "mode", label: "Mode" },
          { key: "initial_cash", label: "Initial cash" }
        ]
      }
    };

    const groups = derivePriorityFieldGroups({
      moduleDef,
      nodeType: "runtime",
      prioritizePathFields: false,
      nodeIssues: [
        {
          code: "UNSUPPORTED_RUNTIME_MODE",
          message: "Runtime mode is not supported."
        }
      ]
    });

    expect(groups).toHaveLength(1);
    expect(groups[0].title).toBe("全部设置");
    expect(groups[0].fields.map((field) => field.key)).toEqual(["mode", "initial_cash"]);
  });
});

describe("deriveConfigureCardOrder", () => {
  it("pushes connections ahead of config when wiring issues dominate", () => {
    const order = deriveConfigureCardOrder({
      prioritizePathFields: true,
      nodeIssues: [
        { code: "AGENT_NO_OUTPUT" },
        { code: "AGENT_NO_INPUT" }
      ]
    });

    expect(order).toEqual(["connections", "validation", "config"]);
  });

  it("keeps config ahead when field issues dominate", () => {
    const order = deriveConfigureCardOrder({
      prioritizePathFields: true,
      nodeIssues: [
        { code: "FIELD_REQUIRED" },
        { code: "FIELD_COMPARE" }
      ]
    });

    expect(order).toEqual(["config", "validation", "connections"]);
  });

  it("falls back to validation-first when issues are neither config nor wiring", () => {
    const order = deriveConfigureCardOrder({
      prioritizePathFields: true,
      nodeIssues: [{ code: "INTENT_CONFIG" }]
    });

    expect(order).toEqual(["validation", "config", "connections"]);
  });
});

describe("resolveConfigureIssueTargetCard", () => {
  it("routes wiring issues into the connections card", () => {
    expect(resolveConfigureIssueTargetCard({ code: "AGENT_NO_OUTPUT" })).toBe("connections");
  });

  it("routes field issues into the config card", () => {
    expect(resolveConfigureIssueTargetCard({ code: "FIELD_REQUIRED" })).toBe("config");
  });

  it("keeps unmatched issues in the validation card", () => {
    expect(resolveConfigureIssueTargetCard({ code: "INTENT_CONFIG" })).toBe("validation");
  });
});
