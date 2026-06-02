import { useMemo } from "react";
import { CartesianGrid, Legend, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useI18n } from "../../../i18n";

export function resolveBacktestCompareEquityPoints(curveArtifact) {
  if (Array.isArray(curveArtifact)) return curveArtifact;
  if (Array.isArray(curveArtifact?.points)) return curveArtifact.points;
  return [];
}

export function buildBacktestCompareEquityOverlayModel(details = []) {
  const [left, right] = details || [];
  const curveA = resolveBacktestCompareEquityPoints(left?.backtest_artifacts?.equity_curve);
  const curveB = resolveBacktestCompareEquityPoints(right?.backtest_artifacts?.equity_curve);
  const benchmarkCurve = resolveBacktestCompareEquityPoints(
    left?.backtest_artifacts?.benchmark_equity_curve || left?.backtest_artifacts?.equity_curve
  );
  const maxLen = Math.max(curveA.length, curveB.length);

  const rows = Array.from({ length: maxLen }, (_, index) => ({
    cycle: index,
    a: curveA[index]?.equity ?? null,
    b: curveB[index]?.equity ?? null,
    benchmark: benchmarkCurve[index]?.equity ?? null
  })).filter((point) => point.a != null || point.b != null);

  return {
    rows,
    hasBenchmark: benchmarkCurve.some((point) => point.equity != null),
    leftLabel: left?.backtest_id?.slice(0, 8) || "A",
    rightLabel: right?.backtest_id?.slice(0, 8) || "B"
  };
}

export function BacktestCompareEquityOverlayChart({ details = [] }) {
  const { t } = useI18n();
  const chartModel = useMemo(
    () => buildBacktestCompareEquityOverlayModel(details),
    [details]
  );

  if (chartModel.rows.length === 0) {
    return (
      <div className="muted-line" style={{ padding: 20, textAlign: "center" }}>
        {t("无权益曲线数据")}
      </div>
    );
  }

  return (
    <div style={{ width: "100%", height: 280, background: "var(--ad-panel)", borderRadius: "var(--ad-radius-md)", padding: "12px 8px 4px 0" }}>
      <ResponsiveContainer>
        <LineChart data={chartModel.rows} margin={{ top: 4, right: 8, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--ad-border)" />
          <XAxis dataKey="cycle" stroke="var(--ad-text-muted)" fontSize={11} tickLine={false} />
          <YAxis stroke="var(--ad-text-muted)" fontSize={11} tickLine={false} width={60} />
          <Tooltip
            contentStyle={{ background: "var(--ad-card)", border: "1px solid var(--ad-border)", borderRadius: 4, fontSize: 12 }}
            formatter={(value) => [value?.toFixed(2), ""]}
          />
          <Legend />
          <Line name={chartModel.leftLabel} type="monotone" dataKey="a" stroke="var(--ad-chart-line-a)" strokeWidth={1.5} dot={false} connectNulls />
          <Line name={chartModel.rightLabel} type="monotone" dataKey="b" stroke="var(--ad-chart-line-b)" strokeWidth={1.5} dot={false} connectNulls />
          {chartModel.hasBenchmark ? (
            <Line name={t("买入持有基准")} type="monotone" dataKey="benchmark" stroke="var(--ad-text-muted)" strokeWidth={1} strokeDasharray="4 4" dot={false} connectNulls />
          ) : null}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
