import { ActivityListCard } from "../components/strategySharedComponents";

export default function StrategyHubRunActivityCard({ items }) {
  return (
    <ActivityListCard
      title="近期运行活动"
      subtitle="不离开策略中心即可跟踪模拟记录与编译 ID。"
      items={items}
      emptyText="暂无近期模拟。"
      testId="strategy-hub-activity-card-run"
    />
  );
}
