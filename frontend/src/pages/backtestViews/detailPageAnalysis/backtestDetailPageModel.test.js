import { describe, expect, it } from "vitest";
import {
  buildBacktestDetailPageModel,
  previewEquityCurve,
  previewTrades
} from "./backtestDetailPageModel";

describe("backtestDetailPageModel", () => {
  it("normalizes runtime artifacts and route identity for the detail page", () => {
    const points = Array.from({ length: 10 }, (_, index) => ({
      ts_ms: index + 1,
      equity: 100 + index
    }));
    const trades = Array.from({ length: 10 }, (_, index) => ({
      fill_id: `fill_${index + 1}`
    }));
    const model = buildBacktestDetailPageModel({
      backtestId: "bt_fallback",
      runtime: {
        selectedBacktestId: "bt_selected",
        backtestHistory: [{ backtest_id: "bt_selected", graph_id: "strategy_from_history" }],
        backtestArtifacts: {
          graph_id: "strategy_from_artifact",
          metrics: {
            artifact_id: "metrics_1",
            started_at_ms: 10,
            ended_at_ms: 20,
            summary: { total_return_ratio: 0.1 }
          },
          manifest: {
            output_artifacts: [{ artifact_id: "out_1" }]
          },
          equity_curve: { points },
          trade_ledger: { trades },
          v4_artifact: {
            schema_version: "v4",
            microstructure_metrics: { fill_rate: 0.5 }
          }
        },
        timeline: [{ event_id: "evt_1" }],
        events: [{ id: "event_1" }],
        retainedKeyEventIndex: { retained_event_count: 1 },
        compactEvidence: { retained_event_count: 1 }
      }
    });

    expect(model.selectedBacktestId).toBe("bt_selected");
    expect(model.selectedSummary?.graph_id).toBe("strategy_from_history");
    expect(model.resolvedStrategyId).toBe("strategy_from_history");
    expect(model.metrics?.artifact_id).toBe("metrics_1");
    expect(model.outputArtifacts).toHaveLength(1);
    expect(model.v4MicroMetrics?.fill_rate).toBe(0.5);
    expect(model.curvePreview.map((point) => point.ts_ms)).toEqual([1, 2, 3, 4, 7, 8, 9, 10]);
    expect(model.tradePreview.map((trade) => trade.fill_id)).toEqual([
      "fill_1",
      "fill_2",
      "fill_3",
      "fill_4",
      "fill_5",
      "fill_6",
      "fill_7",
      "fill_8"
    ]);
    expect(model.timelineSource).toEqual({
      timeline: [{ event_id: "evt_1" }],
      events: [{ id: "event_1" }],
      retained_key_event_index: { retained_event_count: 1 },
      compact_evidence: { retained_event_count: 1 }
    });
  });

  it("falls back to route and empty projections when runtime detail is missing", () => {
    const model = buildBacktestDetailPageModel({ backtestId: "bt_route" });

    expect(model.selectedBacktestId).toBe("bt_route");
    expect(model.selectedSummary).toBeNull();
    expect(model.metrics).toBeNull();
    expect(model.manifest).toBeNull();
    expect(model.summary).toBeNull();
    expect(model.resolvedStrategyId).toBe("");
    expect(model.curvePreview).toEqual([]);
    expect(model.tradePreview).toEqual([]);
  });

  it("keeps small equity curves whole and previews larger curves by head and tail", () => {
    expect(previewEquityCurve([{ ts_ms: 1 }, { ts_ms: 2 }])).toEqual([
      { ts_ms: 1 },
      { ts_ms: 2 }
    ]);
    expect(previewEquityCurve(Array.from({ length: 9 }, (_, index) => ({ ts_ms: index + 1 })))).toEqual([
      { ts_ms: 1 },
      { ts_ms: 2 },
      { ts_ms: 3 },
      { ts_ms: 4 },
      { ts_ms: 6 },
      { ts_ms: 7 },
      { ts_ms: 8 },
      { ts_ms: 9 }
    ]);
    expect(previewTrades([{ id: 1 }, { id: 2 }], 1)).toEqual([{ id: 1 }]);
  });
});
