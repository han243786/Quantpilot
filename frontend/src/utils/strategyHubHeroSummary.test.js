import { describe, expect, it } from "vitest";

import {
  buildStrategyHubMetricCards,
  buildStrategyHubOpsCards
} from "./strategyHubHeroSummary";

describe("strategyHubHeroSummary", () => {
  it("builds hero metric cards from hub summary values", () => {
    const cards = buildStrategyHubMetricCards({
      trackedCount: 12,
      runnableCount: 5,
      researchReadyCount: 4,
      compareCount: 2,
      latestActivityAt: 1710000000000
    });

    expect(cards).toHaveLength(4);
    expect(cards.map((card) => card.label)).toEqual([
      "策略文件",
      "可运行策略",
      "可研究策略",
      "最近活动"
    ]);
    expect(cards[0]).toMatchObject({
      value: "12",
      note: "当前由后端真实追踪、可在策略中心加载的策略文件数量。"
    });
    expect(cards[3].value).toContain("2024");
    expect(cards[3].note).toBe("对比队列：已选 2 项");
  });

  it("builds explicit ops cards and preserves empty activity fallback", () => {
    expect(
      buildStrategyHubMetricCards({
        trackedCount: 0,
        runnableCount: 0,
        researchReadyCount: 0,
        compareCount: 0,
        latestActivityAt: 0
      })[3]
    ).toMatchObject({
      label: "最近活动",
      value: "暂无活动",
      note: "对比队列：已选 0 项"
    });

    const opsCards = buildStrategyHubOpsCards({
      hubSummary: { issueCount: 1, runnableCount: 3 },
      compareSelection: ["bt_a", "bt_b"],
      selectedStrategyCount: 2
    });

    expect(opsCards).toEqual([
      {
        title: "待修复",
        value: "1",
        note: "当前仍被编译或校验问题阻塞的策略数量。",
        tone: "danger"
      },
      {
        title: "运行就绪",
        value: "3",
        note: "可直接进入模拟或回测复盘的策略数量。",
        tone: "success"
      },
      {
        title: "对比队列",
        value: "2",
        note: "进入对比页前，这里应刚好保留两条回测。",
        tone: "info"
      },
      {
        title: "已选策略",
        value: "2",
        note: "进入工作区前，可先用勾选把管理范围收敛到一小组策略。",
        tone: "muted"
      }
    ]);
  });
});
