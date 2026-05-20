/// v3.4.0: 挂单面板 — 实时订单状态 + 成交记录

import { memo, useEffect, useState } from "react";

const API = "/api/executor";

const OrderPanel = memo(function OrderPanel({ strategyId }) {
  const [orders, setOrders] = useState([]);

  useEffect(() => {
    if (!strategyId) return;
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`${API}/strategies/${strategyId}`);
        if (res.ok) {
          const data = await res.json();
          setOrders(data.open_orders || []);
        }
      } catch (e) { console.warn("[OrderPanel] fetch error:", e.message); }
    }, 1500);
    return () => clearInterval(interval);
  }, [strategyId]);

  const pendingOrders = orders.filter(o => o.status !== "Filled" && o.status !== "Cancelled");
  const filledOrders = orders.filter(o => o.status === "Filled");

  return (
    <>
      <div className="exec-sidebar-section">
        <div className="exec-sidebar-title">挂单 ({pendingOrders.length})</div>
        {pendingOrders.length === 0 ? (
          <div style={{ color: "var(--exec-text-secondary)", fontSize: 12, padding: "8px 0" }}>
            <div>暂无挂单</div>
            <div style={{ marginTop: 4, opacity: 0.7 }}>策略部署到执行器后将在此显示订单状态。</div>
          </div>
        ) : (
          pendingOrders.map(o => (
            <div key={o.order_id} className={`exec-order-row ${o.side === "Buy" ? "buy" : "sell"}`}>
              <span>{o.symbol || "?"}</span>
              <span>{o.side === "Buy" ? "买入" : "卖出"}</span>
              <span>{o.quantity || 0}</span>
              <span className="exec-order-status">{orderStatusText(o.status)}</span>
            </div>
          ))
        )}
      </div>
      <div className="exec-sidebar-section">
        <div className="exec-sidebar-title">成交记录 ({filledOrders.length})</div>
        {filledOrders.slice(-10).reverse().map(o => (
          <div key={o.order_id} className={`exec-order-row ${o.side === "Buy" ? "buy" : "sell"}`}>
            <span>{o.symbol || "?"}</span>
            <span>{o.side === "Buy" ? "买入" : "卖出"}</span>
            <span>{(o.executed_qty || 0).toFixed(4)}</span>
            <span className="exec-order-status">{o.fill_price ? `$${o.fill_price}` : ""}</span>
          </div>
        ))}
      </div>
    </>
  );
});
export default OrderPanel;

function orderStatusText(status) {
  switch (status) {
    case "Created": return "已创建";
    case "Submitted": return "已提交";
    case "Accepted": return "已接受";
    case "PartiallyFilled": return "部分成交";
    case "Filled": return "已成交";
    case "Cancelled": return "已取消";
    case "Rejected": return "已拒绝";
    default: return status || "?";
  }
}
