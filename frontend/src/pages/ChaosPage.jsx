import { useState, useEffect, useCallback } from "react";

import { API_BASE } from "../utils/api";

const TYPE_LABELS = {
  DataLatencyInjection: "数据延迟注入",
  EventLossInjection: "事件丢失注入",
  DiskPressureInjection: "磁盘压力注入",
  ClockSkewInjection: "时钟偏移注入",
};

const CHAOS_TYPES = Object.keys(TYPE_LABELS);

export default function ChaosPage() {
  const [experiments, setExperiments] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [creating, setCreating] = useState(false);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/api/v1/chaos/experiments`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setExperiments(Array.isArray(data) ? data : []);
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  const handleCreate = useCallback(async (type) => {
    setCreating(type);
    try {
      await fetch(`${API_BASE}/api/v1/chaos/experiments`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ experiment_type: type }),
      });
      fetchData();
    } catch (_) {}
    setCreating(false);
  }, []);

  const fmtTime = (ts) => {
    if (!ts) return "-";
    const d = new Date(ts);
    return isNaN(d.getTime()) ? ts : d.toLocaleString();
  };

  return (
    <div className="qp-page">

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", flexWrap: "wrap", gap: 8 }}>
        <h2>混沌实验</h2>
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          {CHAOS_TYPES.map((type) => (
            <button
              key={type}
              className="qp-btn qp-btn--primary qp-btn--sm"
              onClick={() => handleCreate(type)}
              disabled={creating === type}
            >
              {creating === type ? "创建中..." : TYPE_LABELS[type]}
            </button>
          ))}
        </div>
      </div>

      {error && (
        <div className="qp-error" role="alert">
          <span>加载失败: {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={fetchData}>重试</button>
        </div>
      )}

      {loading && <div className="qp-loading">加载实验数据...</div>}

      {!loading && !error && (
        <>
          <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={fetchData} style={{ marginBottom: 16 }}>
            刷新
          </button>

          {experiments.length === 0 && (
            <div className="qp-empty">暂无混沌实验记录，点击上方按钮创建实验。</div>
          )}

          {experiments.map((e) => (
            <div className="qp-card qp-fade-in" key={e.experiment_id} role="listitem">
              <div className="qp-card__header">
                <span className="qp-card__title qp-metric" style={{ fontSize: 12 }}>
                  {e.experiment_id}
                </span>
                <span className={e.passed ? "qp-badge qp-badge--ok" : "qp-badge qp-badge--err"}>
                  {e.passed ? "通过" : "失败"}
                </span>
              </div>
              <div className="qp-card__meta">
                <span>{TYPE_LABELS[e.experiment_type] || e.experiment_type}</span>
                <span>{fmtTime(e.executed_at)}</span>
                <span className="qp-metric">恢复 {e.recovery_duration_ms}ms</span>
              </div>
              {e.alerts_triggered?.length > 0 && (
                <div className="qp-card__body">
                  <span style={{ color: "var(--tv-orange)" }}>
                    触发告警: {e.alerts_triggered.join(" / ")}
                  </span>
                  {e.degradation_actions?.length > 0 && (
                    <span style={{ marginLeft: 16, color: "var(--tv-text-secondary)" }}>
                      降级动作: {e.degradation_actions.join(" / ")}
                    </span>
                  )}
                </div>
              )}
            </div>
          ))}
        </>
      )}
    </div>
  );
}
