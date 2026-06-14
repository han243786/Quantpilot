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

  it("prefers live event snapshots over matching run history when no backtest replay exists", () => {
    render(
      <AssetCandlesPanel
        graph={buildGraph("live")}
        runtime={{
          backtestArtifacts: null,
          events: [
            {
              event_type: "PortfolioUpdated",
              event_time_ms: 1_700_000_000_000,
              payload: { equity_estimate: 10_000 }
            },
            {
              event_type: "PortfolioUpdated",
              event_time_ms: 1_700_000_060_000,
              payload: { equity_estimate: 10_900 }
            }
          ],
          history: [
            {
              graph_id: "asset_graph",
              created_at_ms: 1_699_999_000_000,
              account: { equity_estimate: 8_000 }
            }
          ]
        }}
      />
    );

    expect(screen.getByTestId("asset-candles-title")).toHaveTextContent("\u5b9e\u76d8");
    expect(screen.getByTestId("asset-candles-source")).toHaveTextContent("\u5f53\u524d\u8fd0\u884c");
    expect(screen.getByTestId("asset-candles-current-equity")).toHaveTextContent(/10,900\.00/);
    expect(screen.getByTestId("asset-candles-samples")).toHaveTextContent("2");
  });

  it("falls back to graph-matched history snapshots when live snapshots are absent", () => {
    render(
      <AssetCandlesPanel
        graph={buildGraph("paper")}
        runtime={{
          backtestArtifacts: null,
          events: [],
          history: [
            {
              graph_id: "other_graph",
              created_at_ms: 1_700_000_000_000,
              account: { equity_estimate: 9_000 }
            },
            {
              graph_id: "asset_graph",
              created_at_ms: 1_700_000_060_000,
              account: { cash_balance: 10_000, total_net_notional: 500 }
            }
          ]
        }}
      />
    );

    expect(screen.getByTestId("asset-candles-source")).toHaveTextContent("\u6700\u8fd1\u8fd0\u884c");
    expect(screen.getByTestId("asset-candles-current-equity")).toHaveTextContent(/10,500\.00/);
    expect(screen.getByTestId("asset-candles-samples")).toHaveTextContent("1");
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
