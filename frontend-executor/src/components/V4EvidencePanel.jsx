import { useEffect, useMemo, useState } from "react";

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function formatNumber(value, digits = 2) {
  const number = Number(value);
  return Number.isFinite(number) ? number.toFixed(digits) : "-";
}

function latestMachine(snapshot) {
  return asArray(snapshot?.machines).find((machine) => machine.status === "active")
    || asArray(snapshot?.machines)[0]
    || null;
}

function latestExecutionEvent(events) {
  return asArray(events).findLast?.((event) => String(event?.event_type || "").startsWith("execution_"))
    || asArray(events).slice().reverse().find((event) => String(event?.event_type || "").startsWith("execution_"))
    || null;
}

export default function V4EvidencePanel({ strategyId, runtimeKind }) {
  const [snapshot, setSnapshot] = useState(null);
  const [runtimeEvents, setRuntimeEvents] = useState([]);
  const [streamStatus, setStreamStatus] = useState("idle");

  useEffect(() => {
    if (!strategyId || runtimeKind !== "v4") {
      setSnapshot(null);
      setRuntimeEvents([]);
      setStreamStatus("idle");
      return undefined;
    }

    setStreamStatus("connecting");
    const source = new EventSource(`/api/executor/strategies/${strategyId}/events`);
    source.onopen = () => setStreamStatus("connected");
    source.onerror = () => setStreamStatus("error");
    source.onmessage = (event) => {
      try {
        const payload = JSON.parse(event.data);
        if (payload.type === "v4RuntimeMemorySnapshot" && payload.memory_snapshot) {
          setSnapshot(payload.memory_snapshot);
          setRuntimeEvents(asArray(payload.runtime_events));
        }
      } catch (error) {
        console.warn("executor: v4 evidence parse", error);
      }
    };

    return () => source.close();
  }, [strategyId, runtimeKind]);

  const projection = useMemo(() => {
    const machine = latestMachine(snapshot);
    const risk = snapshot?.risk_plane || {};
    const execution = snapshot?.execution || {};
    const simulated = snapshot?.simulated_execution || {};
    const boundary = snapshot?.venue_adapter_boundary || {};
    return {
      machine,
      risk,
      execution,
      simulated,
      boundary,
      lastOrder: simulated?.last_order || null,
      latestExecutionEvent: latestExecutionEvent(runtimeEvents),
      runtimeEventCount: runtimeEvents.length,
      machineCount: asArray(snapshot?.machines).length
    };
  }, [runtimeEvents, snapshot]);

  if (runtimeKind !== "v4") {
    return null;
  }

  return (
    <section className="exec-sidebar-section v4-evidence-panel">
      <div className="params-header">
        <h3>v4 状态机证据</h3>
        <span className={`params-status params-status--${streamStatus === "connected" ? "saved" : streamStatus === "error" ? "error" : "idle"}`}>
          {streamStatus}
        </span>
      </div>

      {!snapshot ? (
        <div className="params-panel-empty">等待 v4 runtime memory_snapshot</div>
      ) : (
        <div className="v4-evidence-grid">
          <div className="v4-evidence-row">
            <span>Slice</span>
            <strong>实时模拟盘</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Settlement</span>
            <strong>本地撮合</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Provider</span>
            <strong>{projection.boundary.provider_order_submission_attached ? "provider 下单" : "无 provider 下单"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Machine</span>
            <strong>{projection.machine?.machine_id || "-"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>State</span>
            <strong>{projection.machine?.state_id || "-"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Active</span>
            <strong>{projection.machineCount}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Risk</span>
            <strong>{projection.risk.last_decision?.approved ? "approved" : projection.risk.rejected_event_count > 0 ? "rejected" : "pending"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Execution</span>
            <strong>{projection.execution.last_decision?.accepted ? "accepted" : projection.execution.rejected_count > 0 ? "rejected" : "pending"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Orders</span>
            <strong>{projection.simulated.order_count || 0} / {projection.simulated.fill_count || 0}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Last order</span>
            <strong>{projection.lastOrder?.provider_order_id || projection.lastOrder?.order_id || "-"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Portfolio</span>
            <strong>{formatNumber(projection.simulated.portfolio_value)}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Runtime events</span>
            <strong>{projection.runtimeEventCount}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Last execution</span>
            <strong>{projection.latestExecutionEvent?.event_type || "-"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Source</span>
            <strong>{projection.boundary.provider_order_submission_allowed ? "provider_allowed" : snapshot.provider_order_submission_attached ? "provider" : "runtime_simulated"}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Real path</span>
            <strong>{projection.risk.real_order_path_unlocked ? "unlocked" : "locked"}</strong>
          </div>
        </div>
      )}
    </section>
  );
}
