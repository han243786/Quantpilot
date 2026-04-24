import { Suspense, lazy } from "react";
import { useStrategyHubRosterData } from "../hooks/useStrategyHubRosterData";
import { StrategyHubInspectorSectionFallback } from "./StrategyHubSectionFallbacks";

const StrategyHubRosterDirectorySection = lazy(() => import("./StrategyHubRosterDirectorySection"));
const StrategyHubActivityPanelsSection = lazy(() => import("./StrategyHubActivityPanelsSection"));

export default function StrategyHubRosterSection({ model }) {
  const { backtestItems, runItems, toolbar, rosterRows } = useStrategyHubRosterData(model);

  return (
    <div className="strategy-hub-main">
      <Suspense fallback={<StrategyHubInspectorSectionFallback title="策略清单" />}>
        <StrategyHubRosterDirectorySection
          model={model}
          toolbar={toolbar}
          rosterRows={rosterRows}
        />
      </Suspense>

      <Suspense fallback={<StrategyHubInspectorSectionFallback title="近期活动" />}>
        <StrategyHubActivityPanelsSection
          model={model}
          backtestItems={backtestItems}
          runItems={runItems}
        />
      </Suspense>
    </div>
  );
}
