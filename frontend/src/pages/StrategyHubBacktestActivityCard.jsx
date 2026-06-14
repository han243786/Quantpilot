import { backtestDetailPath, navigateTo } from "../router";
import { ActivityListCard, StrategyTaskGroup } from "../components/strategySharedComponents";

export default function StrategyHubBacktestActivityCard({ model, items }) {
  return (
    <ActivityListCard
      title="近期研究活动"
      subtitle="让回测直接留在首页可见，避免研究线索被埋进单个策略页面。"
      items={items}
      emptyText="暂无近期回测。"
      testId="strategy-hub-activity-card-backtest"
      renderMeta={(item) => {
        const checked = model.compareSelection.includes(item.id);
        return (
          <StrategyTaskGroup label="研究" tone="info" className="strategy-task-group--inline">
            <button
              className="ad-btn ad-btn--ghost compact-btn"
              onClick={() => navigateTo(backtestDetailPath(item.id, item.graphId))}
            >
              详情
            </button>
            <button
              className={`ad-btn ad-btn--ghost compact-btn${checked ? " compact-btn--selected" : ""}`}
              onClick={() => model.toggleBacktestCompareSelection(item.id)}
            >
              {checked ? "已选择" : "加入对比"}
            </button>
          </StrategyTaskGroup>
        );
      }}
    />
  );
}
