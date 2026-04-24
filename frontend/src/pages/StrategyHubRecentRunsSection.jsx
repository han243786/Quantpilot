import StrategyHubRecentRunItem from "./StrategyHubRecentRunItem";
import { projectStrategyHubRecentRunsView } from "../utils/strategyHubRecentRunsView";

export default function StrategyHubRecentRunsSection({ items }) {
  const view = projectStrategyHubRecentRunsView(items);

  return (
    <section className="strategy-inspector-section">
      <div className="mini-list-title">{view.title}</div>
      <div className="strategy-inspector-list">
        {view.items.map((item) => (
          <StrategyHubRecentRunItem key={item.runId} item={item} />
        ))}
        {view.items.length === 0 ? <div className="muted-line">{view.emptyText}</div> : null}
      </div>
    </section>
  );
}
