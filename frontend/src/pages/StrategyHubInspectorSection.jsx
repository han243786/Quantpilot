import { Suspense, lazy } from "react";
import { useStrategyHubInspectorData } from "../hooks/useStrategyHubInspectorData";
import { StrategyHubInspectorSectionFallback } from "./StrategyHubSectionFallbacks";
import StrategyHubInspectorOverviewSection from "./StrategyHubInspectorOverviewSection";

const StrategyHubRecentBacktestsSection = lazy(() => import("./StrategyHubRecentBacktestsSection"));
const StrategyHubRecentRunsSection = lazy(() => import("./StrategyHubRecentRunsSection"));
const StrategyHubCompareQueueSection = lazy(() => import("./StrategyHubCompareQueueSection"));

export default function StrategyHubInspectorSection({ model, selectedStrategy, compareSelection }) {
  const { overview, recentBacktests, recentRuns, compareQueue } = useStrategyHubInspectorData(
    selectedStrategy,
    compareSelection
  );

  return (
    <aside className="strategy-inspector-card">
      <StrategyHubInspectorOverviewSection
        model={model}
        selectedStrategy={selectedStrategy}
        overview={overview}
      />

      {selectedStrategy ? (
        <>
          <Suspense fallback={<StrategyHubInspectorSectionFallback title="近期回测" />}>
            <StrategyHubRecentBacktestsSection
              graphId={selectedStrategy.graphId}
              items={recentBacktests}
              onToggleCompare={(backtestId) => model.toggleBacktestCompareSelection(backtestId)}
            />
          </Suspense>

          <Suspense fallback={<StrategyHubInspectorSectionFallback title="近期模拟" />}>
            <StrategyHubRecentRunsSection items={recentRuns} />
          </Suspense>

          <Suspense fallback={<StrategyHubInspectorSectionFallback title="对比队列" />}>
            <StrategyHubCompareQueueSection
              graphId={selectedStrategy.graphId}
              compareQueue={compareQueue}
              onClearSelection={() => model.clearBacktestCompareSelection()}
            />
          </Suspense>
        </>
      ) : null}
    </aside>
  );
}
