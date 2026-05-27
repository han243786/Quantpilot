/// v3.4.0: 实时 K 线图表 — lightweight-charts 渲染
/// 专业级 K 线 + 一字线 + 成交量 + 资产曲线

import { memo, useEffect, useRef, useState } from "react";
import { useI18n } from "../i18n";

const KlineChart = memo(function KlineChart({ strategyId }) {
  const { t } = useI18n();
  const containerRef = useRef(null);
  const chartRef = useRef(null);
  const [chartReady, setChartReady] = useState(false);
  const [barCount, setBarCount] = useState(200);
  const [timeframe, setTimeframe] = useState("1m");

  useEffect(() => {
    if (!containerRef.current) return;

    const loadLightweightCharts = async () => {
      const { createChart, ColorType } = await import("lightweight-charts");

      const chart = createChart(containerRef.current, {
        layout: {
          background: { type: ColorType.Solid, color: "#1a1a24" },
          textColor: "#909090",
        },
        grid: {
          vertLines: { color: "#2a2a3a" },
          horzLines: { color: "#2a2a3a" },
        },
        crosshair: { mode: 0 },
        rightPriceScale: { borderColor: "#2a2a3a" },
        timeScale: { borderColor: "#2a2a3a", timeVisible: true },
      });

      // 主窗格: K 线
      const candleSeries = chart.addCandlestickSeries({
        upColor: "#26a69a",
        downColor: "#ef5350",
        borderUpColor: "#26a69a",
        borderDownColor: "#ef5350",
        wickUpColor: "#26a69a",
        wickDownColor: "#ef5350",
      });

      // 成交量窗格
      const volumeSeries = chart.addHistogramSeries({
        priceFormat: { type: "volume" },
        priceScaleId: "volume",
      });
      chart.priceScale("volume").applyOptions({
        scaleMargins: { top: 0.85, bottom: 0 },
      });

      chartRef.current = { chart, candleSeries, volumeSeries };
      setChartReady(true);
    };

    loadLightweightCharts();

    return () => {
      chartRef.current?.chart?.remove();
      chartRef.current = null;
      setChartReady(false);
    };
  }, []);

  // 轮询 K 线数据
  useEffect(() => {
    if (!strategyId || !chartRef.current) return;
    const interval = setInterval(async () => {
      try {
        const params = new URLSearchParams({ count: String(barCount), interval: timeframe });
        const res = await fetch(`/api/executor/strategies/${strategyId}/klines?${params.toString()}`);
        if (!res.ok) return;
        const data = await res.json();
        const { candleSeries, volumeSeries } = chartRef.current;
        if (data.bars?.length > 0) {
          candleSeries.setData(data.bars.map(b => ({
            time: b.open_time_ms / 1000,
            open: b.open, high: b.high, low: b.low, close: b.close,
          })));
          volumeSeries.setData(data.bars.map(b => ({
            time: b.open_time_ms / 1000,
            value: b.volume,
            color: b.close >= b.open ? "rgba(38,166,154,0.3)" : "rgba(239,83,80,0.3)",
          })));
        }
      } catch (e) { console.warn("[KlineChart] fetch error:", e.message); }
    }, 2000);
    return () => clearInterval(interval);
  }, [barCount, chartReady, strategyId, timeframe]);

  if (!strategyId) {
    return <div className="exec-empty"><div className="exec-empty-text">{t("等待策略加载...")}</div></div>;
  }
  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      {!chartReady && (
        <div className="exec-empty">
          <div className="exec-empty-text">{t("图表加载中...")}</div>
        </div>
      )}
      <div className="exec-kline-toolbar" role="group" aria-label={t("图表周期与范围")}>
        {["1m", "5m", "15m"].map((item) => (
          <button
            key={item}
            type="button"
            className={timeframe === item ? "active" : ""}
            onClick={() => setTimeframe(item)}
          >
            {item}
          </button>
        ))}
        <span className="exec-kline-toolbar__divider" />
        {[100, 200, 500].map((count) => (
          <button
            key={count}
            type="button"
            className={barCount === count ? "active" : ""}
            onClick={() => setBarCount(count)}
          >
            {count}
          </button>
        ))}
        <span className="exec-kline-toolbar__source">{t("数据源: public market-data / demo provider")}</span>
      </div>
      <div ref={containerRef} style={{ width: "100%", height: "100%" }} />
    </div>
  );
});
export default KlineChart;
