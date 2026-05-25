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

export default function V4EvidencePanel({ strategyId, runtimeKind }) {
  const [snapshot, setSnapshot] = useState(null);
  const [streamStatus, setStreamStatus] = useState("idle");

  useEffect(() => {
    if (!strategyId || runtimeKind !== "v4") {
      setSnapshot(null);
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
    return {
      machine,
      risk,
      execution,
      simulated,
      machineCount: asArray(snapshot?.machines).length
    };
  }, [snapshot]);

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
            <span>Portfolio</span>
            <strong>{formatNumber(projection.simulated.portfolio_value)}</strong>
          </div>
          <div className="v4-evidence-row">
            <span>Source</span>
            <strong>{snapshot.provider_order_submission_attached ? "provider" : "runtime_simulated"}</strong>
          </div>
        </div>
      )}
    </section>
  );
}
