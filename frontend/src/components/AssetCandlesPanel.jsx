import { useMemo } from "react";
import { getGlobalLocale } from "../i18n";

const SVG_WIDTH = 760;
const SVG_HEIGHT = 238;
const PADDING = { top: 18, right: 16, bottom: 42, left: 18 };

const LABELS = {
  live: "\u5b9e\u76d8",
  paper: "\u6d4b\u8bd5",
  currentRun: "\u5f53\u524d\u8fd0\u884c",
  recentRuns: "\u6700\u8fd1\u8fd0\u884c",
  backtestReplay: "\u5386\u53f2\u56de\u653e",
  kicker: "\u8d44\u4ea7\u8ddf\u8e2a",
  titleSuffix: "\u8d44\u4ea7 K \u7ebf",
  sourcePrefix: "\u6765\u6e90\uff1a",
  sourceSuffix: "\u6743\u76ca\u5feb\u7167",
  currentEquity: "\u5f53\u524d\u6743\u76ca",
  change: "\u533a\u95f4\u53d8\u5316",
  samples: "\u91c7\u6837\u70b9",
  waitingTitle: "\u7b49\u5f85\u8d44\u4ea7\u5feb\u7167",
  waitingHint:
    "\u542f\u52a8\u6d4b\u8bd5\u8fd0\u884c\u6216\u6253\u5f00\u4e00\u6761\u8fd0\u884c\u8bb0\u5f55\u540e\uff0c\u8fd9\u91cc\u4f1a\u663e\u793a\u5f53\u524d\u7b56\u7565\u7684\u6743\u76ca\u53d8\u5316 K \u7ebf\u3002",
  aria: "\u8d44\u4ea7\u53d8\u5316 K \u7ebf"
};

function formatMoney(value) {
  if (!Number.isFinite(value)) return "-";
  return value.toLocaleString(getGlobalLocale(), {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  });
}

function formatPercent(value) {
  if (!Number.isFinite(value)) return "-";
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}

function resolveRuntimeMode(graph) {
  const runtimeNode = graph.nodes.find((node) => node.type === "runtime");
  const mode = runtimeNode?.config?.mode || graph.metadata?.mode || "paper";
  return mode === "live" ? LABELS.live : LABELS.paper;
}

function resolveAccountEquity(account) {
  if (!account || typeof account !== "object") return null;
  if (typeof account.cash_balance === "number") {
    const netNotional =
      typeof account.total_net_notional === "number" ? account.total_net_notional : 0;
    const fallbackEquity = account.cash_balance + netNotional;
    if (
      typeof account.equity_estimate === "number" &&
      (Math.abs(account.equity_estimate) > Number.EPSILON ||
        Math.abs(fallbackEquity) <= Number.EPSILON)
    ) {
      return account.equity_estimate;
    }
    return fallbackEquity;
  }
  if (typeof account.equity_estimate === "number") return account.equity_estimate;
  return null;
}

function buildSnapshotsFromEvents(events) {
  return [...(events || [])]
    .filter((event) => event.event_type === "PortfolioUpdated" && event.payload)
    .sort((left, right) => left.event_time_ms - right.event_time_ms)
    .map((event) => {
      const equity = resolveAccountEquity(event.payload);
      if (!Number.isFinite(equity)) return null;
      return {
        ts: event.event_time_ms,
        label: new Date(event.event_time_ms).toLocaleTimeString(getGlobalLocale(), {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit"
        }),
        equity
      };
    })
    .filter(Boolean);
}

function buildSnapshotsFromHistory(history, graphId) {
  return [...(history || [])]
    .filter((run) => !graphId || run.graph_id === graphId)
    .sort((left, right) => left.created_at_ms - right.created_at_ms)
    .map((run) => {
      const equity = resolveAccountEquity(run.account);
      if (!Number.isFinite(equity)) return null;
      return {
        ts: run.created_at_ms,
        label: new Date(run.created_at_ms).toLocaleString(getGlobalLocale(), {
          month: "2-digit",
          day: "2-digit",
          hour: "2-digit",
          minute: "2-digit"
        }),
        equity
      };
    })
    .filter(Boolean);
}

function buildSnapshotsFromBacktestArtifacts(backtestArtifacts) {
  return [...(backtestArtifacts?.equity_curve?.points || [])]
    .filter((point) => Number.isFinite(point?.equity))
    .sort((left, right) => left.ts_ms - right.ts_ms)
    .map((point) => ({
      ts: point.ts_ms,
      label: new Date(point.ts_ms).toLocaleString(getGlobalLocale(), {
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit"
      }),
      equity: point.equity
    }));
}

function buildCandles(snapshots) {
  return snapshots.map((snapshot, index) => {
    const previousClose = index === 0 ? snapshot.equity : snapshots[index - 1].equity;
    const open = previousClose;
    const close = snapshot.equity;
    return {
      ...snapshot,
      open,
      close,
      high: Math.max(open, close),
      low: Math.min(open, close)
    };
  });
}

function labelIndices(length) {
  if (length <= 1) return [0];
  if (length === 2) return [0, 1];
  return [0, Math.floor((length - 1) / 2), length - 1];
}

function sourceTone(sourceLabel) {
  if (sourceLabel === LABELS.backtestReplay) return "info";
  if (sourceLabel === LABELS.currentRun) return "success";
  return "muted";
}

export default function AssetCandlesPanel({ graph, runtime }) {
  const modeLabel = resolveRuntimeMode(graph);

  const model = useMemo(() => {
    const backtestSnapshots = buildSnapshotsFromBacktestArtifacts(runtime.backtestArtifacts);
    const eventSnapshots = buildSnapshotsFromEvents(runtime.events);
    const historySnapshots = buildSnapshotsFromHistory(runtime.history, graph.metadata?.graph_id);
    const snapshots =
      backtestSnapshots.length > 0
        ? backtestSnapshots
        : eventSnapshots.length > 0
          ? eventSnapshots
          : historySnapshots;
    const sourceLabel =
      backtestSnapshots.length > 0
        ? LABELS.backtestReplay
        : eventSnapshots.length > 0
          ? LABELS.currentRun
          : LABELS.recentRuns;
    const candles = buildCandles(snapshots);

    if (!candles.length) {
      return {
        candles: [],
        latest: null,
        delta: null,
        deltaPct: null,
        sourceLabel
      };
    }

    const first = candles[0];
    const latest = candles[candles.length - 1];
    const delta = latest.close - first.open;
    const deltaPct = Math.abs(first.open) > Number.EPSILON ? (delta / first.open) * 100 : 0;
    return {
      candles,
      latest,
      delta,
      deltaPct,
      sourceLabel
    };
  }, [graph.metadata?.graph_id, runtime.backtestArtifacts, runtime.events, runtime.history]);

  const chart = useMemo(() => {
    if (!model.candles.length) return null;

    const innerWidth = SVG_WIDTH - PADDING.left - PADDING.right;
    const innerHeight = SVG_HEIGHT - PADDING.top - PADDING.bottom;
    const lows = model.candles.map((candle) => candle.low);
    const highs = model.candles.map((candle) => candle.high);
    const minValue = Math.min(...lows);
    const maxValue = Math.max(...highs);
    const range = Math.max(maxValue - minValue, Math.abs(maxValue) * 0.02, 1);
    const paddedMin = minValue - range * 0.18;
    const paddedMax = maxValue + range * 0.18;
    const xStep =
      model.candles.length > 1 ? innerWidth / (model.candles.length - 1) : innerWidth / 2;
    const bodyWidth = Math.max(
      10,
      Math.min(26, innerWidth / Math.max(model.candles.length * 1.9, 2))
    );

    const yFor = (value) =>
      PADDING.top + ((paddedMax - value) / Math.max(paddedMax - paddedMin, 1)) * innerHeight;

    const closeLine = model.candles
      .map((candle, index) => {
        const x = PADDING.left + (model.candles.length === 1 ? innerWidth / 2 : index * xStep);
        const y = yFor(candle.close);
        return `${index === 0 ? "M" : "L"} ${x} ${y}`;
      })
      .join(" ");

    const lastIndex = model.candles.length - 1;
    const lastX =
      PADDING.left +
      (model.candles.length === 1 ? innerWidth / 2 : lastIndex * xStep);
    const lastY = yFor(model.candles[lastIndex].close);
    const markerWidth = 106;
    const markerHeight = 28;
    const markerX = Math.max(
      PADDING.left,
      Math.min(lastX - markerWidth / 2, SVG_WIDTH - PADDING.right - markerWidth)
    );
    const markerY = Math.max(PADDING.top + 4, lastY - 38);

    return {
      yFor,
      xStep,
      bodyWidth,
      closeLine,
      lastX,
      lastY,
      markerX,
      markerY,
      labels: labelIndices(model.candles.length),
      gridValues: [paddedMax, paddedMin + (paddedMax - paddedMin) / 2, paddedMin]
    };
  }, [model.candles]);

  return (
    <section className="asset-chart-card" data-testid="asset-candles-panel">
      <div className="asset-chart-header">
        <div className="asset-chart-copy" data-testid="asset-candles-header">
          <div className="panel-subtitle">{LABELS.kicker}</div>
          <div className="panel-title" data-testid="asset-candles-title">{`${modeLabel}${LABELS.titleSuffix}`}</div>
          <div className="asset-chart-subtitle" data-testid="asset-candles-source">
            {`${LABELS.sourcePrefix}${model.sourceLabel}${LABELS.sourceSuffix}`}
          </div>
          <div className="asset-chart-pills">
            <span className="status-pill info">{modeLabel}</span>
            <span className={`status-pill ${sourceTone(model.sourceLabel)}`}>{model.sourceLabel}</span>
          </div>
        </div>

        <div className="asset-chart-stats" data-testid="asset-candles-stats">
          <div className="asset-stat-chip" data-testid="asset-candles-current-equity">
            <span>{LABELS.currentEquity}</span>
            <strong>{formatMoney(model.latest?.close)}</strong>
          </div>
          <div
            className={`asset-stat-chip ${Number(model.delta) >= 0 ? "positive" : "negative"}`}
            data-testid="asset-candles-change"
          >
            <span>{LABELS.change}</span>
            <strong>
              {Number.isFinite(model.delta)
                ? `${model.delta > 0 ? "+" : ""}${formatMoney(model.delta)}`
                : "-"}
            </strong>
            <em>{formatPercent(model.deltaPct)}</em>
          </div>
          <div className="asset-stat-chip" data-testid="asset-candles-samples">
            <span>{LABELS.samples}</span>
            <strong>{model.candles.length}</strong>
          </div>
        </div>
      </div>

      <div className="asset-chart-body">
        {!chart ? (
          <div className="asset-chart-empty" data-testid="asset-candles-empty">
            <div className="empty-state">{LABELS.waitingTitle}</div>
            <div className="muted-line">{LABELS.waitingHint}</div>
          </div>
        ) : (
          <div className="asset-chart-frame">
            <svg
              className="asset-chart-svg"
              viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}
              preserveAspectRatio="none"
              aria-label={LABELS.aria}
              data-testid="asset-candles-chart"
            >
              {chart.gridValues.map((value, index) => {
                const y = chart.yFor(value);
                return (
                  <g key={`${value}-${index}`}>
                    <line
                      className="asset-grid-line"
                      x1={PADDING.left}
                      y1={y}
                      x2={SVG_WIDTH - PADDING.right}
                      y2={y}
                    />
                    <text
                      className="asset-axis-label"
                      x={SVG_WIDTH - PADDING.right}
                      y={y - 4}
                      textAnchor="end"
                    >
                      {formatMoney(value)}
                    </text>
                  </g>
                );
              })}

              <path className="asset-close-line" d={chart.closeLine} />
              <line
                className="asset-current-guide"
                x1={chart.lastX}
                y1={PADDING.top}
                x2={chart.lastX}
                y2={SVG_HEIGHT - PADDING.bottom}
              />
              <circle className="asset-current-point" cx={chart.lastX} cy={chart.lastY} r="4.5" />
              <g className="asset-current-tag" transform={`translate(${chart.markerX} ${chart.markerY})`}>
                <rect width="106" height="28" rx="10" />
                <text x="53" y="18" textAnchor="middle">
                  当前 {formatMoney(model.latest?.close)}
                </text>
              </g>

              {model.candles.map((candle, index) => {
                const x =
                  PADDING.left +
                  (model.candles.length === 1
                    ? (SVG_WIDTH - PADDING.left - PADDING.right) / 2
                    : index * chart.xStep);
                const wickTop = chart.yFor(candle.high);
                const wickBottom = chart.yFor(candle.low);
                const bodyTop = chart.yFor(Math.max(candle.open, candle.close));
                const bodyBottom = chart.yFor(Math.min(candle.open, candle.close));
                const isUp = candle.close >= candle.open;
                const bodyHeight = Math.max(bodyBottom - bodyTop, 2);
                return (
                  <g key={`${candle.ts}-${index}`}>
                    <line
                      className={`asset-candle-wick ${isUp ? "up" : "down"}`}
                      x1={x}
                      y1={wickTop}
                      x2={x}
                      y2={wickBottom}
                    />
                    <rect
                      className={`asset-candle-body ${isUp ? "up" : "down"}`}
                      x={x - chart.bodyWidth / 2}
                      y={bodyTop}
                      width={chart.bodyWidth}
                      height={bodyHeight}
                      rx="3"
                    />
                  </g>
                );
              })}

              {chart.labels.map((index) => {
                const x =
                  PADDING.left +
                  (model.candles.length === 1
                    ? (SVG_WIDTH - PADDING.left - PADDING.right) / 2
                    : index * chart.xStep);
                return (
                  <text
                    key={model.candles[index]?.label || `candle-${index}`}
                    className="asset-axis-label"
                    x={x}
                    y={SVG_HEIGHT - 12}
                    textAnchor="middle"
                  >
                    {model.candles[index]?.label}
                  </text>
                );
              })}
            </svg>
          </div>
        )}
      </div>
    </section>
  );
}
