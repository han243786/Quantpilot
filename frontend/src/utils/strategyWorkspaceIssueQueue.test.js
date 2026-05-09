import { describe, expect, it } from "vitest";
import {
  DEFAULT_WORKSPACE_ISSUE_FILTERS,
  buildWorkspaceIssueQueue,
  diagnosticQueueNodeType,
  diagnosticQueueSource,
  filterWorkspaceIssueQueue,
  filterWorkspaceIssueQueueByNodeType,
  filterWorkspaceIssueQueueBySource,
  workspaceIssueFiltersDirty,
  workspaceIssueQueueSeverityLabel,
  workspaceIssueSeverityText
} from "./strategyWorkspaceIssueQueue";

function buildGraph() {
  return {
    nodes: [
      { id: "intent_1", name: "Intent node", type: "intent" },
      { id: "agent_1", name: "Agent node", type: "agent" }
    ],
    validation_state: {
      graph_issues: [{ code: "GRAPH_EMPTY", message: "Graph issue", hint: "Fix graph" }],
      node_issues: {
        intent_1: [
          {
            id: "intent_warning",
            level: "warning",
            code: "INTENT_WARN",
            message: "Intent needs an agent",
            hint: "Wire the intent output"
          }
        ]
      }
    }
  };
}

describe("strategyWorkspaceIssueQueue", () => {
  it("builds a sorted queue with stable fallback labels", () => {
    const queue = buildWorkspaceIssueQueue(buildGraph(), [
      {
        code: "COMPILE_BLOCKER",
        severity: "error",
        source: "runtime",
        message: "Compile failed",
        hint: "Retry compile",
        target: {
          scope: "node",
          node_id: "agent_1",
          label: "Agent target"
        }
      }
    ]);

    expect(queue).toHaveLength(3);
    expect(queue[0]).toMatchObject({
      severity: "error",
      source: "runtime",
      title: "Agent target",
      actionable: true
    });
    expect(queue[1]).toMatchObject({
      severity: "error",
      title: "策略图",
      actionable: false
    });
    expect(queue[2]).toMatchObject({
      severity: "warning",
      title: "Intent node",
      actionable: true
    });
  });

  it("exposes filter and label helpers for the page shell", () => {
    const items = [
      { id: "1", severity: "error", source: "runtime", nodeType: "agent", actionable: true },
      { id: "2", severity: "warning", source: "validation", nodeType: "intent", actionable: false }
    ];

    expect(workspaceIssueQueueSeverityLabel("all", 2)).toBe("全部 2");
    expect(workspaceIssueQueueSeverityLabel("error", 1)).toBe("错误 1");
    expect(workspaceIssueSeverityText("warning")).toBe("警告");
    expect(diagnosticQueueSource({ source: "validation" })).toBe("校验");
    expect(diagnosticQueueNodeType({ nodeType: null })).toBe("策略图");
    expect(filterWorkspaceIssueQueue(items, "error", false)).toHaveLength(1);
    expect(filterWorkspaceIssueQueueBySource(items, "runtime")).toHaveLength(1);
    expect(filterWorkspaceIssueQueueByNodeType(items, "intent")).toHaveLength(1);
    expect(workspaceIssueFiltersDirty(DEFAULT_WORKSPACE_ISSUE_FILTERS)).toBe(false);
    expect(workspaceIssueFiltersDirty({ severityFilter: "warning" })).toBe(true);
  });
});
