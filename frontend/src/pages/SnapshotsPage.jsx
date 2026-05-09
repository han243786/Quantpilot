import { useState, useEffect, useCallback } from "react";

import { API_BASE } from "../utils/api";

const ACTOR_ID = "local_operator";

export default function SnapshotsPage() {
  const [snapshots, setSnapshots] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/api/v1/snapshots`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      setSnapshots(Array.isArray(data) ? data : []);
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
      await fetch(`${API_BASE}/api/v1/snapshots/create`, { method: "POST" });
      fetchData();
    } catch (_) {}
    setCreating(false);
  }, []);

  const handleRestore = useCallback(async (snapshotId) => {
    setRestoring(snapshotId);
    try {
      await fetch(`${API_BASE}/api/v1/snapshots/${snapshotId}/restore`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ actor_id: ACTOR_ID }),
      });
      fetchData();
    } catch (_) {}
    setRestoring(null);
  }, []);

  return (
    <div className="qp-page">

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2>签名快照</h2>
        <button
          className="qp-btn qp-btn--primary qp-btn--sm"
          onClick={handleCreate}
          disabled={creating}
        >
          {creating ? "创建中..." : "创建快照"}
        </button>
      </div>

      {error && (
        <div className="qp-error" role="alert">
          <span>加载失败: {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={fetchData}>重试</button>
        </div>
      )}

      {loading && <div className="qp-loading">加载快照数据...</div>}

      {!loading && !error && (
        <>
          <button className="qp-btn qp-btn--ghost qp-btn--sm" onClick={fetchData} style={{ marginBottom: 16 }}>
            刷新
          </button>

          {snapshots.length === 0 && (
            <div className="qp-empty">暂无签名快照, 点击「创建快照」生成第一个部署状态签名。</div>
          )}

          {snapshots.map((s) => (
            <div className="qp-card qp-fade-in" key={s.snapshot_id} role="listitem">
              <div className="qp-card__header">
                <span className="qp-card__title qp-metric">{s.snapshot_id}</span>
                <span className="qp-badge qp-badge--info">快照</span>
              </div>
              <div className="qp-card__meta">
                <span>部署版本: <span className="qp-metric">{s.deployment_revision}</span></span>
                <span>策略: <span className="qp-metric">{s.strategy_version}</span></span>
                <span>参数: <span className="qp-metric">{s.parameter_version}</span></span>
              </div>
              <div className="qp-card__body" style={{ fontSize: 11, wordBreak: "break-all", fontFamily: "var(--tv-mono)" }}>
                {s.signature?.substring(0, 64)}
              </div>
              <div className="qp-card__body" style={{ marginTop: 8 }}>
                <button
                  className="qp-btn qp-btn--ghost qp-btn--sm"
                  onClick={() => handleRestore(s.snapshot_id)}
                  disabled={restoring === s.snapshot_id}
                >
                  {restoring === s.snapshot_id ? "恢复中..." : "恢复到此快照"}
                </button>
              </div>
            </div>
          ))}
        </>
      )}
    </div>
  );
}
