/// v3.4.0: 资产面板 — 余额/持仓/权益概要

import { memo, useEffect, useState } from "react";
import { useI18n } from "../i18n";

const API = "/api/executor";

const AssetPanel = memo(function AssetPanel({ strategyId }) {
  const { t } = useI18n();
  const [assets, setAssets] = useState(null);

  useEffect(() => {
    if (!strategyId) return;
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`${API}/strategies/${strategyId}`);
        if (res.ok) {
          const data = await res.json();
          setAssets(data.portfolio || null);
        }
      } catch (e) { console.warn("[AssetPanel] fetch error:", e.message); }
    }, 2000);
    return () => clearInterval(interval);
  }, [strategyId]);

  if (!assets) {
    return (
      <div className="exec-sidebar-section">
        <div className="exec-sidebar-title">{t("资产")}</div>
        <div style={{ color: "var(--exec-text-secondary)", fontSize: 12 }}>{t("加载中...")}</div>
      </div>
    );
  }

  return (
    <div className="exec-sidebar-section">
      <div className="exec-sidebar-title">{t("资产")}</div>
      <div className="exec-asset-item">
        <span>{t("总权益")}</span>
        <span className="exec-asset-value">
          ${((assets.cash_balance || 0) + (assets.total_net_notional || 0)).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
        </span>
      </div>
      <div className="exec-asset-item">
        <span>{t("现金余额")}</span>
        <span className="exec-asset-value">${(assets.cash_balance || 0).toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
      </div>
      <div className="exec-asset-item">
        <span>{t("可用现金")}</span>
        <span>${(assets.available_cash_balance || 0).toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
      </div>
      <div className="exec-asset-item">
        <span>{t("冻结")}</span>
        <span>${(assets.frozen_cash_balance || 0).toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
      </div>
      {assets.positions && assets.positions.length > 0 && (
        <>
          <div className="exec-sidebar-title" style={{ marginTop: 8 }}>{t("持仓")}</div>
          {assets.positions.map((p, i) => (
            <div key={i} className="exec-asset-item">
              <span>{p.symbol || "?"}</span>
              <span className="exec-asset-value">{(p.net_qty || 0).toFixed(4)}</span>
            </div>
          ))}
        </>
      )}
    </div>
  );
});
export default AssetPanel;
