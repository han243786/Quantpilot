import { buildV4RuntimeEvidenceProjection } from "../utils/v4RuntimeEvidence";

function yesNo(value) {
  return value ? "是" : "否";
}

function CapabilityEntry({ entry, testId }) {
  return (
    <div className="open-order-item" data-testid={testId}>
      <div className="open-order-topline">
        <strong>{entry.capability}</strong>
        <span className={`status-pill ${entry.status_tone}`}>{entry.status}</span>
      </div>
      <div className="kv-line">
        <span>来源</span>
        <strong>
          <span className={`status-pill ${entry.source_tone}`}>{entry.source}</span>
        </strong>
      </div>
      <div className="muted-line">{entry.reason}</div>
    </div>
  );
}

export default function V4RuntimeEvidencePanel({
  source,
  testId = "v4-runtime-evidence-panel"
}) {
  const projection = buildV4RuntimeEvidenceProjection(source || {});
  if (!projection.available) return null;

  const riskDecision = projection.risk_plane?.last_decision;
  const executionDecision = projection.execution?.last_decision;
  const executionEntries = projection.execution?.entries || [];

  return (
    <div className="open-orders-card" data-testid={testId}>
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">v4 状态机证据</div>
          <div className="muted-line">
            状态、Risk Plane 与 Execution capability 只读投影，不代表真实下单接入。
          </div>
        </div>
        <strong>
          <span className="status-pill neutral">{projection.runtime_mode}</span>
        </strong>
      </div>

      <div className="history-meta-grid" data-testid={`${testId}-summary`}>
        <div className="history-meta-chip">
          <span>Machine</span>
          <strong>{projection.machine_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>Active</span>
          <strong>{projection.active_machine_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>SoftSilent</span>
          <strong>{projection.soft_silent_machine_count}</strong>
        </div>
        <div className="history-meta-chip history-meta-chip-wide">
          <span>Provider order</span>
          <strong>{projection.boundary_label}</strong>
        </div>
      </div>

      <div className="mini-list" data-testid={`${testId}-machines`}>
        <div className="mini-list-title">状态机状态</div>
        {projection.machines.map((machine) => (
          <div key={machine.machine_id} className="mini-item">
            <div className="kv-line">
              <span>{machine.machine_id}</span>
              <strong>{machine.state_id}</strong>
            </div>
            <div className="muted-line">
              {machine.template} · {machine.status} · cache {yesNo(machine.has_cache)}
            </div>
          </div>
        ))}
      </div>

      <div className="mini-list" data-testid={`${testId}-risk-plane`}>
        <div className="mini-list-title">Risk Plane</div>
        <div className="kv-line">
          <span>required</span>
          <strong>{yesNo(projection.risk_plane.required)}</strong>
        </div>
        <div className="kv-line">
          <span>approval / rejection</span>
          <strong>
            {projection.risk_plane.approved_event_count}/
            {projection.risk_plane.rejected_event_count}
          </strong>
        </div>
        <div className="kv-line">
          <span>real order path</span>
          <strong>{projection.risk_plane.real_order_path_unlocked ? "unlocked" : "locked"}</strong>
        </div>
        {riskDecision ? (
          <div className={`history-note history-note-${riskDecision.tone}`}>
            {riskDecision.source_machine_id} -&gt; {riskDecision.target_machine_id} ·{" "}
            {riskDecision.reason}
          </div>
        ) : null}
      </div>

      <div className="mini-list" data-testid={`${testId}-execution`}>
        <div className="mini-list-title">Execution capability</div>
        <div className="kv-line">
          <span>venue</span>
          <strong>{projection.execution.venue_id || "-"}</strong>
        </div>
        <div className="kv-line">
          <span>accepted / rejected</span>
          <strong>
            {projection.execution.accepted_count}/{projection.execution.rejected_count}
          </strong>
        </div>
        {executionDecision ? (
          <div className={`history-note history-note-${executionDecision.tone}`}>
            {executionDecision.runtime_mode} · {executionDecision.reason}
          </div>
        ) : null}
        {executionEntries.length > 0 ? (
          executionEntries.map((entry) => (
            <CapabilityEntry
              key={`${entry.capability}-${entry.source}-${entry.status}`}
              entry={entry}
              testId={`${testId}-capability-${entry.capability}`}
            />
          ))
        ) : (
          <div className="muted-line">当前没有 execution capability decision entry。</div>
        )}
      </div>
    </div>
  );
}
