import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import AssetCandlesPanel from "./AssetCandlesPanel";

function buildGraph(mode = "paper") {
  return {
    metadata: {
      graph_id: "asset_graph",
      mode
    },
    nodes: [
      {
        id: "runtime_node",
        type: "runtime",
        config: { mode }
      }
    ]
  };
}

describe("AssetCandlesPanel", () => {
  it("renders chart structure and summary metrics when snapshots exist", () => {
    render(
      <AssetCandlesPanel
        graph={buildGraph("paper")}
        runtime={{
          backtestArtifacts: {
            equity_curve: {
              points: [
                { ts_ms: 1_700_000_000_000, equity: 10_000 },
                { ts_ms: 1_700_000_060_000, equity: 10_250 }
              ]
            }
          },
          events: [],
          history: []
        }}
      />
    );

    expect(screen.getByTestId("asset-candles-panel")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-header")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-title")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-source")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-chart")).toBeInTheDocument();

    expect(screen.getByTestId("asset-candles-current-equity")).toHaveTextContent(/10,250\.00/);
    expect(screen.getByTestId("asset-candles-change")).toHaveTextContent(/250\.00/);
    expect(screen.getByTestId("asset-candles-change")).toHaveTextContent(/\+2\.50%/);
    expect(screen.getByTestId("asset-candles-samples")).toHaveTextContent("2");
  });

  it("renders the empty-state structure when no snapshots are available", () => {
    render(
      <AssetCandlesPanel
        graph={buildGraph("paper")}
        runtime={{
          backtestArtifacts: null,
          events: [],
          history: []
        }}
      />
    );

    expect(screen.getByTestId("asset-candles-panel")).toBeInTheDocument();
    expect(screen.getByTestId("asset-candles-empty")).toBeInTheDocument();
    expect(screen.queryByTestId("asset-candles-chart")).not.toBeInTheDocument();
  });
});
