import { Suspense, lazy } from "react";
import { useStrategyHubBodyData } from "../hooks/useStrategyHubBodyData";
import {
  StrategyHubTemplateLibraryFallback,
  StrategyHubInspectorFallback,
  StrategyHubRosterFallback
} from "./StrategyHubSectionFallbacks";

const StrategyHubTemplateLibrarySection = lazy(() => import("./StrategyHubTemplateLibrarySection"));
const StrategyHubRosterSection = lazy(() => import("./StrategyHubRosterSection"));
const StrategyHubInspectorSection = lazy(() => import("./StrategyHubInspectorSection"));

export default function StrategyHubBodySection({ model }) {
  const { selectedStrategy, compareSelection } = useStrategyHubBodyData(model);

  return (
    <div className="strategy-hub-body">
      <div className="strategy-hub-grid">
        <div className="strategy-hub-main">
          <Suspense fallback={<StrategyHubTemplateLibraryFallback />}>
            <StrategyHubTemplateLibrarySection model={model} />
          </Suspense>

          <Suspense fallback={<StrategyHubRosterFallback />}>
            <StrategyHubRosterSection model={model} />
          </Suspense>
        </div>

        <Suspense fallback={<StrategyHubInspectorFallback />}>
          <StrategyHubInspectorSection
            model={model}
            selectedStrategy={selectedStrategy}
            compareSelection={compareSelection}
          />
        </Suspense>
      </div>
    </div>
  );
}
