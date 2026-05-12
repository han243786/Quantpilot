import { useState, useEffect, useCallback } from "react";
import { useI18n } from "../i18n";
import { API_BASE } from "../utils/api";

const ACTOR_ID = "local_operator";

export default function SnapshotsPage() {
  const { t } = useI18n();
  const [snapshots, setSnapshots] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/v1/snapshots`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      // v1.0.5: 自动解包分页响应 {data, total, limit, offset}
      setSnapshots(Array.isArray(data) ? data : (data?.data || []));
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  const handleCreate = useCallback(async () => {
    setCreating(true);
    try {
      await fetch(`${API_BASE}/v1/snapshots/create`, { method: "POST" });
      fetchData();
    } catch (_) {}
    setCreating(false);
  }, []);

  const handleRestore = useCallback(async (snapshotId) => {
    setRestoring(snapshotId);
    try {
      await fetch(`${API_BASE}/v1/snapshots/${snapshotId}/restore`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ actor_id: ACTOR_ID }),
      });
      fetchData();
    } catch (_) {}
    setRestoring(null);
  }, []);

  return (
    <main className="qp-page">

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1>{t("签名快照")}</h1>
        <button
          className="qp-btn qp-btn--primary qp-btn--sm"
          onClick={handleCreate}
          disabled={creating}
        >
          {creating ? t("创建中...") : t("创建快照")}
        </button>
      </div>

      {error && (
        <div className="qp-error" role="alert">
          <span>{t("加载失败")}: {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={fetchData}>{t("重试")}</button>
        </div>
      )}

      {loading && <div className="qp-loading">{t("加载快照数据...")}</div>}

      {!loading && !error && (
        <>
          <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={fetchData} style={{ marginBottom: 16 }}>
            {t("刷新")}
          </button>

          {snapshots.length === 0 && (
            <div className="qp-empty">{t("暂无签名快照")}</div>
          )}

          {snapshots.map((s) => (
            <div className="qp-card qp-fade-in" key={s.snapshot_id} role="listitem">
              <div className="qp-card__header">
                <span className="qp-card__title qp-metric">{s.snapshot_id}</span>
                <span className="qp-badge qp-badge--info">{t("快照")}</span>
              </div>
              <div className="qp-card__meta">
                <span>{t("部署版本")}: <span className="qp-metric">{s.deployment_revision}</span></span>
                <span>{t("策略")}: <span className="qp-metric">{s.strategy_version}</span></span>
                <span>{t("参数")}: <span className="qp-metric">{s.parameter_version}</span></span>
              </div>
              <div className="qp-card__body" style={{ fontSize: 11, wordBreak: "break-all", fontFamily: "var(--ad-font-mono)" }}>
                {s.signature?.substring(0, 64)}
              </div>
              <div className="qp-card__body" style={{ marginTop: 8 }}>
                <button
                  className="qp-btn qp-btn--ghost qp-btn--sm"
                  onClick={() => handleRestore(s.snapshot_id)}
                  disabled={restoring === s.snapshot_id}
                >
                  {restoring === s.snapshot_id ? t("恢复中...") : t("恢复到此快照")}
                </button>
              </div>
            </div>
          ))}
        </>
      )}
    </main>
  );
}
