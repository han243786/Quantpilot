import { useMemo } from "react";
import { getGlobalLocale } from "../i18n";
import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer
} from "recharts";

function computeDrawdownSeries(equityCurve) {
  if (!equityCurve || equityCurve.length < 2) return [];
  let peak = equityCurve[0].equity;
  return equityCurve.map((point) => {
    if (point.equity > peak) peak = point.equity;
    const drawdown = peak > 0 ? ((peak - point.equity) / peak) * 100 : 0;
    return {
      ts_ms: point.ts_ms,
      time: new Date(point.ts_ms).toLocaleDateString(getGlobalLocale(), { month: "short", day: "numeric" }),
      drawdown: drawdown > 0 ? -drawdown : 0
    };
  });
}

export default function DrawdownChart({ equityCurve, title, height = 260 }) {
  const data = useMemo(() => computeDrawdownSeries(equityCurve), [equityCurve]);

  if (!data.length) {
    return (
      <div className="chart-empty" style={{ height }}>
        <span>无回撤数据</span>
      </div>
    );
  }

  const maxDD = Math.min(...data.map((d) => d.drawdown));

  return (
    <div style={{ width: "100%" }}>
      {title && <div className="chart-title">{title}</div>}
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart data={data} margin={{ top: 8, right: 8, bottom: 0, left: 8 }}>
          <CartesianGrid stroke="var(--ad-border)" strokeDasharray="3 3" />
          <XAxis
            dataKey="time"
            tick={{ fill: "var(--ad-text-muted)", fontSize: 11 }}
            interval="preserveStartEnd"
          />
          <YAxis
            tick={{ fill: "var(--ad-text-muted)", fontSize: 11 }}
            domain={[maxDD * 1.1, 0]}
            tickFormatter={(v) => `${v.toFixed(1)}%`}
          />
          <Tooltip
            contentStyle={{
              background: "var(--ad-panel)",
              border: "1px solid var(--ad-border)",
              borderRadius: 4
            }}
            formatter={(value) => [`${Number(value).toFixed(2)}%`, "回撤"]}
          />
          <Area
            type="monotone"
            dataKey="drawdown"
            stroke="var(--ad-error)"
            fill="var(--ad-error)"
            fillOpacity={0.12}
            strokeWidth={1.5}
            dot={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
