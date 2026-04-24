export default function StrategyHubRecentRunItem({ item }) {
  return (
    <div className="strategy-inspector-item">
      <div>
        <strong>{item.runId}</strong>
        <div className="muted-line">{item.createdAtLabel}</div>
      </div>
      <div className={`status-pill ${item.statusTone}`}>{item.compileIdLabel}</div>
    </div>
  );
}
