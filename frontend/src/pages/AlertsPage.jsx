import { useState, useEffect, useCallback } from "react";
import Block5Nav from "../components/Block5Nav";
import { API_BASE } from "../utils/api";
import { useI18n } from "../i18n";

const ACTOR_ID = "local_operator";

export default function AlertsPage() {
  const { t } = useI18n();
  const [data, setData] = useState({ firings: [], rules: [] });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [acking, setAcking] = useState(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/api/v1/alerts`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      setData(await res.json());
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  const handleAcknowledge = useCallback(async (firingId) => {
    setAcking(firingId);
    try {
      await fetch(`${API_BASE}/api/v1/alerts/${firingId}/acknowledge`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ actor_id: ACTOR_ID }),
      });
      fetchData();
    } catch (_) {}
    setAcking(null);
  }, []);

  const badge = (s) => {
    const map = { P1: "err", P2: "warn", P3: "info" };
    return `qp-badge qp-badge--${map[s] || "muted"}`;
  };

  const fmtTime = (ms) => {
    if (!ms) return "-";
    return new Date(ms).toLocaleString();
  };

  return (
    <div className="qp-page">
      <Block5Nav />

      <h2>{t("告警面板")}</h2>

      {error && (
        <div className="qp-error" role="alert">
          <span>{t("加载失败:")} {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={fetchData}>{t("重试")}</button>
        </div>
      )}

      {loading && <div className="qp-loading">{t("加载告警数据...")}</div>}

      {!loading && !error && (
        <>
          <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={fetchData} style={{ marginBottom: 16 }}>
            {t("刷新")}
          </button>

          <h3>{t("活跃告警")} ({data.firings.length})</h3>
          {data.firings.length === 0 && (
            <div className="qp-empty">{t("暂无活跃告警，系统运行正常")}</div>
          )}
          {data.firings.map((f) => (
            <div className="qp-card qp-fade-in" key={f.firing_id} role="listitem">
              <div className="qp-card__header">
                <span className="qp-card__title">{f.rule_name}</span>
                <span className={badge(f.severity)}>{f.severity}</span>
              </div>
              <div className="qp-card__meta">
                <span>{f.state}</span>
                <span>{fmtTime(f.fired_at_ms)}</span>
                {f.acknowledged_by && <span>{t("确认人:")} {f.acknowledged_by}</span>}
              </div>
              <div className="qp-card__body">{f.detail}</div>
              {f.state !== "Acknowledged" && f.state !== "Resolved" && (
                <div className="qp-card__body" style={{ marginTop: 8 }}>
                  <button
                    className="qp-btn qp-btn--primary qp-btn--sm"
                    onClick={() => handleAcknowledge(f.firing_id)}
                    disabled={acking === f.firing_id}
                  >
                    {acking === f.firing_id ? t("确认中...") : t("确认告警")}
                  </button>
                </div>
              )}
            </div>
          ))}

          <h3>{t("告警规则")} ({data.rules.length})</h3>
          {data.rules
            .filter((r) => r.enabled)
            .map((r) => (
              <div className="qp-card qp-fade-in" key={r.rule_name}>
                <div className="qp-card__header">
                  <span className="qp-card__title">{r.rule_name}</span>
                  <span className={badge(r.severity)}>{r.severity}</span>
                </div>
                <div className="qp-card__meta">
                  <span>{r.description}</span>
                </div>
                <div className="qp-card__body" style={{ color: "var(--tv-text-secondary)", fontSize: 12 }}>
                  {r.action}
                </div>
              </div>
            ))}
        </>
      )}
    </div>
  );
}
