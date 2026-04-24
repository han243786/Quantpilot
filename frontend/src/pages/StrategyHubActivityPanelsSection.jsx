import { Suspense, lazy } from "react";
import { StrategyHubInspectorSectionFallback } from "./StrategyHubSectionFallbacks";

const StrategyHubBacktestActivityCard = lazy(() => import("./StrategyHubBacktestActivityCard"));
const StrategyHubRunActivityCard = lazy(() => import("./StrategyHubRunActivityCard"));

export default function StrategyHubActivityPanelsSection({ model, backtestItems, runItems }) {
  return (
    <section className="strategy-activity-grid">
      <Suspense fallback={<StrategyHubInspectorSectionFallback title="近期研究活动" />}>
        <StrategyHubBacktestActivityCard model={model} items={backtestItems} />
      </Suspense>

      <Suspense fallback={<StrategyHubInspectorSectionFallback title="近期运行活动" />}>
        <StrategyHubRunActivityCard items={runItems} />
      </Suspense>
    </section>
  );
}
