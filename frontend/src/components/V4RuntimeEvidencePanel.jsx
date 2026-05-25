import { buildV4RuntimeEvidenceProjection } from "../utils/v4RuntimeEvidence";
import ComplexityBudgetPanel from "./ComplexityBudgetPanel";

function yesNo(value) {
  return value ? "是" : "否";
}

function fmtNumber(value) {
  return Number(value || 0).toLocaleString("zh-CN", {
    maximumFractionDigits: 8
  });
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

function MachineRow({ machine }) {
  return (
    <div className="mini-item" style={{ marginLeft: `${machine.depth * 16}px` }}>
      <div className="kv-line">
        <span>{machine.machine_id}</span>
        <strong>{machine.state_id}</strong>
      </div>
      <div className="muted-line">
        {machine.template} 路 {machine.status} 路 cache {yesNo(machine.has_cache)}
      </div>
      {machine.children.map((child) => (
        <MachineRow key={child.machine_id} machine={child} />
      ))}
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
  const simulated = projection.simulated_execution;
  const lastOrder = simulated?.last_order;
  const lastFill = simulated?.last_fill;
  const boundary = projection.venue_adapter_boundary;

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
        <div className="history-meta-chip">
          <span>Orders</span>
          <strong>{simulated.order_count}</strong>
        </div>
        <div className="history-meta-chip">
          <span>Fills</span>
          <strong>{simulated.fill_count}</strong>
        </div>
      </div>

      <div className="mini-list" data-testid={`${testId}-machines`}>
        <div className="mini-list-title">状态机状态</div>
        {projection.machines.map((machine) => (
          <MachineRow key={machine.machine_id} machine={machine} />
        ))}
      </div>

      <ComplexityBudgetPanel
        metrics={projection.complexity_metrics}
        testId={`${testId}-complexity-budget`}
      />

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

      <div className="mini-list" data-testid={`${testId}-simulated-execution`}>
        <div className="mini-list-title">本地模拟执行</div>
        <div className="kv-line">
          <span>portfolio</span>
          <strong>
            {fmtNumber(simulated.portfolio_value)} {simulated.quote_asset}
          </strong>
        </div>
        <div className="kv-line">
          <span>cash / fee</span>
          <strong>
            {fmtNumber(simulated.cash_balance)} / {fmtNumber(simulated.realized_fees)}
          </strong>
        </div>
        <div className="kv-line">
          <span>open / rejected</span>
          <strong>
            {simulated.open_order_count}/{simulated.rejected_order_count}
          </strong>
        </div>
        <div className="kv-line">
          <span>asset points</span>
          <strong>{simulated.asset_curve_points}</strong>
        </div>
        {lastOrder ? (
          <div className="mini-item">
            <div className="kv-line">
              <span>{lastOrder.order_id}</span>
              <strong>{lastOrder.status}</strong>
            </div>
            <div className="muted-line">
              {lastOrder.symbol} · {lastOrder.action} · {lastOrder.order_type} ·{" "}
              {fmtNumber(lastOrder.filled_quantity)}/{fmtNumber(lastOrder.requested_quantity)}
            </div>
            {lastOrder.rejection_reason ? (
              <div className="history-note history-note-danger">
                {lastOrder.rejection_reason}
              </div>
            ) : null}
          </div>
        ) : (
          <div className="muted-line">当前没有 simulated order。</div>
        )}
        {lastFill ? (
          <div className="mini-item">
            <div className="kv-line">
              <span>{lastFill.fill_id}</span>
              <strong>{fmtNumber(lastFill.notional)}</strong>
            </div>
            <div className="muted-line">
              {lastFill.symbol} · {fmtNumber(lastFill.quantity)} @ {fmtNumber(lastFill.price)} ·
              fee {fmtNumber(lastFill.fee)} {lastFill.fee_asset}
            </div>
          </div>
        ) : null}
        {simulated.positions.length > 0 ? (
          simulated.positions.map((position) => (
            <div
              key={`${position.venue_id}-${position.symbol}`}
              className="mini-item"
              data-testid={`${testId}-position-${position.symbol}`}
            >
              <div className="kv-line">
                <span>{position.symbol}</span>
                <strong>{fmtNumber(position.net_quantity)}</strong>
              </div>
              <div className="muted-line">
                {position.venue_id} · mark {fmtNumber(position.market_price)} · value{" "}
                {fmtNumber(position.market_value)}
              </div>
            </div>
          ))
        ) : null}
      </div>

      <div className="mini-list" data-testid={`${testId}-venue-boundary`}>
        <div className="mini-list-title">VenueAdapter 边界</div>
        <div className="kv-line">
          <span>submission allowed</span>
          <strong>{yesNo(boundary.provider_order_submission_allowed)}</strong>
        </div>
        <div className="kv-line">
          <span>settlement</span>
          <strong>{boundary.settlement_authority}</strong>
        </div>
        <div className="kv-line">
          <span>pre-submit reject</span>
          <strong>{yesNo(boundary.rejection_before_provider_submit)}</strong>
        </div>
        <div className="muted-line">{boundary.reason}</div>
      </div>
    </div>
  );
}
