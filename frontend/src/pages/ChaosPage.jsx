import { useState, useEffect, useCallback } from "react";
import { useI18n } from "../i18n";
import { API_BASE } from "../utils/api";

const CHAOS_TYPE_KEYS = {
  DataLatencyInjection: "data_latency_injection",
  EventLossInjection: "event_loss_injection",
  DiskPressureInjection: "disk_pressure_injection",
  ClockSkewInjection: "clock_skew_injection",
};

export default function ChaosPage() {
  const { t } = useI18n();

  const TYPE_LABELS = {
    DataLatencyInjection: t("数据延迟注入"),
    EventLossInjection: t("事件丢失注入"),
    DiskPressureInjection: t("磁盘压力注入"),
    ClockSkewInjection: t("时钟偏移注入"),
  };

  const [experiments, setExperiments] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [creating, setCreating] = useState(false);

  const fetchData = async (signal) => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/v1/chaos/experiments`, { signal });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setExperiments(Array.isArray(data) ? data : []);
    } catch (e) {
      if (!signal?.aborted) setError(e.message);
    } finally {
      if (!signal?.aborted) setLoading(false);
    }
  };

  useEffect(() => {
    const controller = new AbortController();
    fetchData(controller.signal);
    return () => controller.abort();
  }, []);

  const handleCreate = useCallback(async (experimentType) => {
    if (!window.confirm("确认启动混沌实验？这可能会影响系统性能。")) return;
    setCreating(experimentType);
    try {
      const INJECTION_SPECS = {
        DataLatencyInjection: { target: "data_module", parameter: "latency_ms", value: 500, duration_ms: 30000 },
        EventLossInjection: { target: "event_stream", parameter: "loss_rate", value: 0.3, duration_ms: 30000 },
        DiskPressureInjection: { target: "storage", parameter: "fill_mb", value: 400, duration_ms: 30000 },
        ClockSkewInjection: { target: "system_clock", parameter: "skew_ms", value: 5000, duration_ms: 30000 },
      };
      await fetch(`${API_BASE}/v1/chaos/experiments`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          experiment_type: CHAOS_TYPE_KEYS[experimentType],
          injection: INJECTION_SPECS[experimentType]
        }),
      });
      fetchData();
    } catch (_) {}
    setCreating(false);
  }, []);

  const fmtTime = (ts) =>
    ts ? new Date(ts).toLocaleString() : "-";

  return (
    <main className="qp-page">

      <div className="chaos-header">
        <h1>{t("混沌实验")}</h1>
        <div className="chaos-actions">
          {Object.keys(CHAOS_TYPE_KEYS).map((type) => (
            <button
              key={type}
              className="qp-btn qp-btn--primary qp-btn--sm"
              onClick={() => handleCreate(type)}
              disabled={creating === type}
            >
              {creating === type ? t("创建中...") : TYPE_LABELS[type]}
            </button>
          ))}
        </div>
      </div>

      {error && (
        <div className="qp-error" role="alert">
          <span>{t("加载失败")}: {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={fetchData}>{t("重试")}</button>
        </div>
      )}

      {loading && <div className="qp-loading">{t("加载实验数据...")}</div>}

      {!loading && !error && (
        <>
          <button className="qp-btn qp-btn--ghost qp-btn--sm chaos-refresh" onClick={fetchData}>
            {t("刷新")}
          </button>

          {experiments.length === 0 && (
            <div className="qp-empty">{t("暂无混沌实验记录")}</div>
          )}

          {experiments.map((e) => (
            <div className="qp-card qp-fade-in" key={e.experiment_id} role="listitem">
              <div className="qp-card__header">
                <span className="qp-card__title qp-metric chaos-experiment-id">
                  {e.experiment_id}
                </span>
                <span className={e.passed ? "qp-badge qp-badge--ok" : "qp-badge qp-badge--err"}>
                  {e.passed ? t("通过") : t("失败")}
                </span>
              </div>
              <div className="qp-card__meta">
                <span>{TYPE_LABELS[e.experiment_type] || e.experiment_type}</span>
                <span>{fmtTime(e.executed_at)}</span>
                <span className="qp-metric">{t("恢复")} {e.recovery_duration_ms}ms</span>
              </div>
              {e.alerts_triggered?.length > 0 && (
                <div className="qp-card__body">
                  <span className="chaos-alert-trigger">
                    {t("触发告警")}: {e.alerts_triggered.join(" / ")}
                  </span>
                  {e.degradation_summary && (
                    <span className="chaos-degradation">
                      {t("性能下降")}: {e.degradation_summary}
                    </span>
                  )}
                </div>
              )}
            </div>
          ))}
        </>
      )}
    </main>
  );
}
