import StrategyHubInlineNote from "./StrategyHubInlineNote";
import { StrategyMetricCard, StrategyOpsCard } from "./StrategyHubSharedComponents";
import { buildStrategyHubMetricCards, buildStrategyHubOpsCards } from "../utils/strategyHubHeroSummary";
import { navigateTo, strategyWorkspacePath } from "../router";

export default function StrategyHubHeroSection({ model }) {
  const openTutorial = () => {
    window.dispatchEvent(new CustomEvent("qp-open-tutorial"));
  };
  const metricCards = buildStrategyHubMetricCards(model.hubSummary);
  const opsCards = buildStrategyHubOpsCards({
    hubSummary: model.hubSummary,
    compareSelection: model.compareSelection,
    selectedStrategyCount: model.selectedStrategyCount
  });

  return (
    <>
      <header className="strategy-hub-hero" data-testid="strategy-hub-hero">
        <div className="strategy-hub-hero__copy">
          <StrategyHubInlineNote
            title="策略中心说明"
            triggerLabel="查看策略中心说明"
            triggerText="策略中心"
            content="这里是策略管理的总入口。先在此查看健康状态、近期研究与运行压力，再只在需要深入编辑或诊断单个策略时进入对应工作区。"
          />
          <button
            className="ad-btn ad-btn--primary strategy-hub-hero__cta"
            onClick={() => void model.openBlankWorkspace()}
            data-testid="strategy-hub-hero-cta"
          >
            开始使用 — 创建第一个策略
          </button>
        </div>

        <div className="strategy-hub-hero__actions">
          <button
            className="ad-btn ad-btn--primary"
            data-testid="strategy-hub-open-current-workspace"
            onClick={() =>
              navigateTo(strategyWorkspacePath(model.graph.metadata?.graph_id || "draft_graph"))
            }
          >
            打开当前工作区
          </button>
          <button
            className="ad-btn ad-btn--ghost"
            data-testid="strategy-hub-open-blank-workspace"
            onClick={() => void model.openBlankWorkspace()}
          >
            打开空白工作区
          </button>
          <button
            className="ad-btn ad-btn--ghost"
            data-testid="strategy-hub-open-tutorial"
            onClick={openTutorial}
          >
            新手指引
          </button>
          <button
            className="ad-btn ad-btn--ghost"
            onClick={() => void Promise.all([model.refreshRunHistory(), model.refreshBacktestHistory()])}
          >
            刷新活动
          </button>
        </div>
      </header>

      <section className="strategy-hub-status-strip" aria-label="策略中心状态总览">
        {metricCards.map((item) => (
          <StrategyMetricCard key={item.label} label={item.label} value={item.value} note={item.note} />
        ))}
        {opsCards.map((item) => (
          <StrategyOpsCard
            key={item.title}
            title={item.title}
            value={item.value}
            note={item.note}
            tone={item.tone}
          />
        ))}
      </section>
    </>
  );
}
