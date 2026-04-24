import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import DiagnosticsPanel from "./DiagnosticsPanel";
import { useGraphStore } from "../store/graphStore";

describe("DiagnosticsPanel", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        focusCompileDiagnostic: vi.fn()
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders structured compile diagnostics through stable row hooks", () => {
    render(
      <DiagnosticsPanel
        compileSummary={{
          diagnostics: [
            {
              code: "QPWARM001",
              source: "strategy_ir",
              severity: "warning",
              message: "intent `Trend Entry` needs at least 50 bars",
              target: {
                scope: "node",
                node_id: "data_feed",
                field: "window_size",
                label: "Price Feed.window_size"
              },
              hint: "Increase `Price Feed` window_size to >= 50."
            }
          ]
        }}
      />
    );

    expect(screen.getByTestId("diagnostics-panel")).toBeInTheDocument();
    expect(screen.getByTestId("diagnostics-panel-title")).toBeInTheDocument();
    const row = screen.getByTestId("diagnostics-row-QPWARM001");
    expect(screen.getByTestId("diagnostics-meta-QPWARM001")).toHaveTextContent("QPWARM001");
    expect(screen.getByTestId("diagnostics-meta-QPWARM001")).toHaveTextContent("Price Feed.window_size");
    expect(screen.getByTestId("diagnostics-message-QPWARM001")).toHaveTextContent(/needs at least 50 bars/i);
    expect(row).toHaveTextContent(/Increase `Price Feed` window_size/i);
  });

  it("routes actionable diagnostics without depending on visible severity copy", () => {
    const focusCompileDiagnostic = vi.fn();
    const onRouteDiagnostic = vi.fn();
    act(() => {
      useGraphStore.setState({
        focusCompileDiagnostic
      });
    });

    render(
      <DiagnosticsPanel
        compileSummary={{
          diagnostics: [
            {
              code: "QPBLOCK001",
              source: "runtime",
              severity: "error",
              message: "Runtime compile rejected the execution module.",
              target: {
                scope: "node",
                node_id: "execution_1",
                label: "Execution node"
              }
            }
          ]
        }}
        onRouteDiagnostic={onRouteDiagnostic}
      />
    );

    fireEvent.click(screen.getByTestId("diagnostics-row-QPBLOCK001"));

    expect(focusCompileDiagnostic).toHaveBeenCalledWith({
      scope: "node",
      node_id: "execution_1",
      label: "Execution node"
    });
    expect(onRouteDiagnostic).toHaveBeenCalledWith({
      code: "QPBLOCK001",
      source: "runtime",
      severity: "error",
      message: "Runtime compile rejected the execution module.",
      target: {
        scope: "node",
        node_id: "execution_1",
        label: "Execution node"
      }
    });
  });

  it("shows repair-path notes through stable note hooks", () => {
    render(
      <DiagnosticsPanel
        compileSummary={{
          diagnostics: [
            {
              code: "QPBLOCK002",
              source: "runtime",
              severity: "error",
              message: "Execution guard is still missing on the active path.",
              target: {
                scope: "node",
                node_id: "execution_1",
                label: "Paper execution"
              }
            }
          ]
        }}
        graph={{
          nodes: [
            { id: "intent_1", name: "Signal" },
            { id: "execution_1", name: "Paper" }
          ],
          edges: []
        }}
        repairPathState={{
          pathNodeIds: ["intent_1", "execution_1"],
          pathEdgeIds: []
        }}
      />
    );

    expect(screen.getByTestId("diagnostics-meta-QPBLOCK002")).toHaveTextContent("Signal -> Paper");
  });
});
