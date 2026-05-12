import { useMemo } from "react";

export default function StrategyWorkspaceDebugTab({ debugBars }) {
  const bars = debugBars || [];

  const { columns, rows, chartPaths } = useMemo(() => {
    if (bars.length === 0) return { columns: [], rows: [], chartPaths: [] };

    const allKeys = new Set();
    bars.forEach((b) => Object.keys(b).forEach((k) => allKeys.add(k)));
    const columns = Array.from(allKeys).filter((k) => k !== "bar");
    const rows = bars;

    // Generate SVG polyline paths for each column
    const chartPaths = columns.map((col) => {
      const values = bars
        .map((b) => b[col])
        .filter((v) => v !== null && v !== undefined);
      if (values.length < 2) return { col, points: "" };

      const chartW = 600;
      const chartH = 100;
      const max = Math.max(...values);
      const min = Math.min(...values);
      const range = max - min || 1;
      const points = values
        .map((v, i) => {
          const x = (i / (values.length - 1)) * chartW;
          const y = chartH - ((v - min) / range) * chartH;
          return `${x.toFixed(1)},${y.toFixed(1)}`;
        })
        .join(" ");
      return { col, points, min, max, chartW, chartH };
    });

    return { columns, rows, chartPaths };
  }, [bars]);

  const activeColumns = useMemo(() => {
    if (!bars || !bars.length) return [];
    const cols = new Set();
    bars.forEach(bar => {
      Object.keys(bar).forEach(key => {
        if (key !== 'bar' && key !== 'equity' && key !== 'timestamp_ms' && bar[key] != null) {
          cols.add(key);
        }
      });
    });
    return Array.from(cols);
  }, [bars]);

  if (bars.length === 0) {
    return (
      <div className="property-card workspace-debug-empty">
        <p>暂无调试数据。</p>
        <p style={{ fontSize: "13px" }}>
          在 QS 策略中添加 <code>@debug(var1, var2)</code> 指令后运行回测即可看到 per-bar 数据。
        </p>
      </div>
    );
  }

  return (
    <div className="workspace-debug-container">
      <div className="property-card">
        <div className="property-card-heading">
          <div className="property-card-title">调试数据 — Per-Bar 值</div>
          <div className="property-card-caption">
            {bars.length} 根 bar · {activeColumns.length} 个活跃变量（共 {columns.length} 列）
          </div>
        </div>
      </div>

      {/* Chart section */}
      {chartPaths.length > 0 && (
        <div className="property-card">
          <div className="property-card-heading">
            <div className="property-card-title">趋势图</div>
          </div>
          {chartPaths.map(({ col, points, min, max, chartW, chartH }) => (
            <div key={col} className="workspace-debug-chart">
              <div className="workspace-debug-chart__label">
                {col} (min={min?.toFixed(4)}, max={max?.toFixed(4)})
              </div>
              <svg
                width={chartW}
                height={chartH}
                className="workspace-debug-chart__svg"
              >
                <polyline
                  points={points}
                  fill="none"
                  stroke="var(--ad-accent)"
                  strokeWidth="1.5"
                />
              </svg>
            </div>
          ))}
        </div>
      )}

      {/* Table section */}
      <div className="property-card" style={{ overflow: "auto", maxHeight: "500px" }}>
        <table className="workspace-debug-table">
          <thead>
            <tr className="workspace-debug-table__header">
              <th>bar</th>
              {activeColumns.map((col) => (
                <th key={col}>{col}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, i) => (
              <tr key={i}>
                <td className="bar-index">{row.bar}</td>
                {activeColumns.map((col) => {
                  const val = row[col];
                  return (
                    <td key={col}>
                      {val !== null && val !== undefined ? (
                        typeof val === "boolean" ? (
                          <span style={{ color: val ? "var(--ad-success)" : "var(--ad-error)" }}>{String(val)}</span>
                        ) : (
                          <span className="val-number">{Number(val).toFixed(4)}</span>
                        )
                      ) : (
                        <span className="val-null">—</span>
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
