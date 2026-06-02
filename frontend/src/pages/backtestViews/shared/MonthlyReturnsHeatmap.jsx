import { useMemo } from "react";

function parsePeriod(period) {
  if (period == null || typeof period !== "string") return { year: "?", month: null, label: "?" };
  const parts = period.split("-");
  if (parts.length === 2) {
    return { year: parts[0], month: parseInt(parts[1], 10), label: "M" };
  }
  return { year: parts[0], month: null, label: "Y" };
}

const MONTH_LABELS = ["1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月"];

function heatColor(returnRatio) {
  const pct = returnRatio * 100;
  if (Math.abs(pct) < 0.001) return "var(--ad-panel)";      // 零收益率 → 中性灰
  if (pct > 10) return "var(--ad-heat-deep-green)";
  if (pct > 5) return "var(--ad-heat-green)";
  if (pct > 0) return "var(--ad-heat-light-green)";
  if (pct > -5) return "var(--ad-heat-light-red)";
  if (pct > -10) return "var(--ad-heat-red)";
  return "var(--ad-heat-deep-red)";
}

function textColor(returnRatio) {
  if (Math.abs(returnRatio) < 0.001) return "var(--ad-text)";
  return Math.abs(returnRatio) > 0.05 ? "#fff" : "var(--ad-text)";
}

export default function MonthlyReturnsHeatmap({ periodReturns, title }) {
  const { years, grid } = useMemo(() => {
    if (!periodReturns || !periodReturns.length) return { years: [], grid: {} };

    const byMonth = {};
    for (const pr of periodReturns) {
      const { year, month } = parsePeriod(pr.period);
      if (month == null) continue; // skip non-monthly
      if (!byMonth[year]) byMonth[year] = Array(12).fill(null);
      byMonth[year][month - 1] = pr.return_ratio;
    }

    const years = Object.keys(byMonth).sort();
    return { years, grid: byMonth };
  }, [periodReturns]);

  if (!years.length) {
    return (
      <div className="chart-empty" style={{ height: 160 }}>
        <span>无月度收益率数据</span>
      </div>
    );
  }

  return (
    <div style={{ width: "100%", overflowX: "auto" }}>
      {title && <div className="chart-title" style={{ marginBottom: 8 }}>{title}</div>}
      <table className="monthly-heatmap">
        <thead>
          <tr>
            <th className="heatmap-year-header">年份</th>
            {MONTH_LABELS.map((m) => (
              <th key={m} className="heatmap-month-header">{m}</th>
            ))}
            <th className="heatmap-year-header">年收益</th>
          </tr>
        </thead>
        <tbody>
          {years.map((year) => {
            const row = grid[year];
            let yearReturn = 1.0;
            for (const r of row) {
              if (r != null) yearReturn *= (1 + r);
            }
            yearReturn = yearReturn - 1;
            return (
              <tr key={year}>
                <td className="heatmap-year-label">{year}</td>
                {row.map((r, i) => (
                  <td
                    key={i}
                    className="heatmap-cell"
                    style={{
                      background: r != null ? heatColor(r) : "var(--ad-panel)",
                      color: r != null ? textColor(r) : "var(--ad-text-muted)",
                    }}
                    title={r != null ? `${(r * 100).toFixed(2)}%` : "-"}
                  >
                    {r != null ? `${(r * 100).toFixed(1)}%` : "-"}
                  </td>
                ))}
                <td
                  className="heatmap-cell heatmap-year-return"
                  style={{
                    background: heatColor(yearReturn),
                    color: textColor(yearReturn),
                    fontWeight: 600,
                  }}
                >
                  {`${yearReturn >= 0 ? "+" : ""}${(yearReturn * 100).toFixed(1)}%`}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
