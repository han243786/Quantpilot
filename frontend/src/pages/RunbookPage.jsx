import React, { useState, useEffect } from "react";
import { useI18n } from "../i18n";
import { API_BASE } from "../utils/api";

export default function RunbookPage() {
  const { t } = useI18n();
  const [scenarios, setScenarios] = useState([]);
  const [expanded, setExpanded] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_BASE}/v1/runbook`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      setScenarios(await res.json());
    } catch (e) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  const badge = (s) => {
    const map = { P1: "err", P2: "warn", P3: "info" };
    return `qp-badge qp-badge--${map[s] || "muted"}`;
  };

  return (
    <main className="qp-page">

      <h1>{t("故障场景手册")}</h1>

      {error && (
        <div className="qp-error" role="alert">
          <span>{t("加载失败")}: {error}</span>
          <button className="qp-btn qp-btn--sm" onClick={fetchData}>{t("重试")}</button>
        </div>
      )}

      {loading && <div className="qp-loading">{t("加载场景数据...")}</div>}

      {!loading && !error && scenarios.length === 0 && (
        <div className="qp-empty">{t("暂无故障场景数据")}</div>
      )}

      {!loading &&
        !error &&
        scenarios.map((s) => {
          const isOpen = expanded === s.scenario_id;
          return (
            <div className="qp-card qp-fade-in" key={s.scenario_id} role="listitem">
              <div
                className="qp-card__header"
                style={{ cursor: "pointer" }}
                onClick={() => setExpanded(isOpen ? null : s.scenario_id)}
                onKeyDown={(e) =>
                  e.key === "Enter" && setExpanded(isOpen ? null : s.scenario_id)
                }
                tabIndex={0}
                role="button"
                aria-expanded={isOpen}
              >
                <span className="qp-card__title">
                  {isOpen ? "▾" : "▸"} {s.name}
                </span>
                <span className={badge(s.severity)}>{s.severity}</span>
              </div>
              <div className="qp-card__meta">
                <span>{t("症状")}: {s.symptoms?.join(" / ")}</span>
              </div>

              {isOpen && (
                <div className="qp-card__body">
                  <h3>{t("诊断步骤")}</h3>
                  {s.diagnostic_steps?.map((d, i) => (
                    <div key={i} style={{ padding: "2px 0" }}>
                      <span style={{ color: "var(--ad-accent)", fontWeight: 500 }}>
                        {d.step_number}.
                      </span>{" "}
                      {d.description}
                      {d.api_call && (
                        <code
                          style={{
                            marginLeft: 8,
                            fontSize: 11,
                            color: "var(--ad-accent)",
                            background: "rgba(41, 98, 255, 0.08)",
                            padding: "1px 6px",
                            borderRadius: 3,
                          }}
                        >
                          {d.api_call}
                        </code>
                      )}
                    </div>
                  ))}

                  <h3>{t("恢复步骤")}</h3>
                  {s.recovery_steps?.map((r, i) => (
                    <div key={i} style={{ padding: "2px 0" }}>
                      <span style={{ color: "var(--ad-warning)", fontWeight: 500 }}>
                        {r.step_number}.
                      </span>{" "}
                      <span style={{ color: "var(--ad-text-secondary)" }}>
                        [{r.condition}]
                      </span>{" "}
                      → {r.action}
                    </div>
                  ))}

                  <hr className="qp-divider" />

                  <h3>{t("验证")}</h3>
                  <p style={{ color: "var(--ad-success)", margin: 0 }}>{s.verification}</p>
                </div>
              )}
            </div>
          );
        })}
    </main>
  );
}
