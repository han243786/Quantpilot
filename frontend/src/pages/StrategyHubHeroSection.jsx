import StrategyHubInlineNote from "./StrategyHubInlineNote";
import { StrategyMetricCard, StrategyOpsCard, StrategyTaskGroup } from "./StrategyHubSharedComponents";
import { formatCount, formatTime } from "../utils/strategyHubFormatters";
import { navigateTo, strategyWorkspacePath } from "../router";

export default function StrategyHubHeroSection({ model }) {
  const headerCards = [
    {
      label: "策略文件",
      value: formatCount(model.hubSummary.trackedCount),
      note: "当前由后端真实追踪、可在策略中心加载的策略文件数量。"
    },
    {
      label: "可运行策略",
      value: formatCount(model.hubSummary.runnableCount),
      note: "当前没有明显阻塞，可以直接编译或启动模拟的策略数量。"
    },
    {
      label: "可研究策略",
      value: formatCount(model.hubSummary.researchReadyCount),
      note: "至少已经有一条持久化回测，可继续查看或对比的策略数量。"
    },
    {
      label: "最近活动",
      value: model.hubSummary.latestActivityAt
        ? formatTime(model.hubSummary.latestActivityAt)
        : "暂无活动",
      note: `对比队列：已选 ${formatCount(model.hubSummary.compareCount)} 项`
    }
  ];

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
        </div>

        <div className="strategy-hub-hero__actions">
          <StrategyTaskGroup label="管理" tone="muted" showLabel={false}>
            <button
              className="ghost-btn"
              onClick={() => void Promise.all([model.refreshRunHistory(), model.refreshBacktestHistory()])}
            >
              刷新活动
            </button>
            <button className="ghost-btn" onClick={() => void model.loadLatestGraph()}>
              同步最新策略图
            </button>
          </StrategyTaskGroup>
          <StrategyTaskGroup label="构建" tone="info" showLabel={false}>
            <button
              className="primary-btn"
              data-testid="strategy-hub-open-current-workspace"
              onClick={() =>
                navigateTo(strategyWorkspacePath(model.graph.metadata?.graph_id || "draft_graph"))
              }
            >
              打开当前工作区
            </button>
            <button
              className="ghost-btn"
              data-testid="strategy-hub-open-blank-workspace"
              onClick={() => void model.openBlankWorkspace()}
            >
              打开空白工作区
            </button>
          </StrategyTaskGroup>
        </div>
      </header>

      <section className="strategy-hub-status-strip" aria-label="策略中心状态总览">
        {headerCards.map((item) => (
          <StrategyMetricCard key={item.label} label={item.label} value={item.value} note={item.note} />
        ))}
        <StrategyOpsCard
          title="待修复"
          value={formatCount(model.hubSummary.issueCount)}
          note="当前仍被编译或校验问题阻塞的策略数量。"
          tone="danger"
        />
        <StrategyOpsCard
          title="运行就绪"
          value={formatCount(model.hubSummary.runnableCount)}
          note="可直接进入模拟或回测复盘的策略数量。"
          tone="success"
        />
        <StrategyOpsCard
          title="对比队列"
          value={formatCount(model.compareSelection.length)}
          note="进入对比页前，这里应刚好保留两条回测。"
          tone="info"
        />
        <StrategyOpsCard
          title="已选策略"
          value={formatCount(model.selectedStrategyCount)}
          note="进入工作区前，可先用勾选把管理范围收敛到一小组策略。"
          tone="muted"
        />
      </section>

      <section className="strategy-hub-toolbar">
        <label className="strategy-hub-search">
          <span>搜索</span>
          <input
            value={model.query}
            onChange={(event) => model.setQuery(event.target.value)}
            placeholder="按策略 ID、名称、编译 ID 或数据集搜索"
          />
        </label>

        <label className="strategy-hub-filter">
          <span>范围</span>
          <select value={model.scopeFilter} onChange={(event) => model.setScopeFilter(event.target.value)}>
            <option value="all">全部策略</option>
            <option value="current">当前策略</option>
            <option value="active">有模拟记录</option>
            <option value="backtested">有回测记录</option>
          </select>
        </label>

        <label className="strategy-hub-filter">
          <span>状态</span>
          <select value={model.healthFilter} onChange={(event) => model.setHealthFilter(event.target.value)}>
            <option value="all">全部状态</option>
            <option value="runnable">可运行</option>
            <option value="issues">待修复</option>
            <option value="tracked">仅历史记录</option>
          </select>
        </label>

        <label className="strategy-hub-filter">
          <span>排序</span>
          <select value={model.sortMode} onChange={(event) => model.setSortMode(event.target.value)}>
            <option value="activity">最近活动</option>
            <option value="health">状态优先</option>
            <option value="research">研究深度</option>
            <option value="return">最近收益</option>
          </select>
        </label>
      </section>
    </>
  );
}
