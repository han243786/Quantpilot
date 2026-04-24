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
      <Suspense fallback={<StrategyHubTemplateLibraryFallback />}>
        <StrategyHubTemplateLibrarySection model={model} />
      </Suspense>

      <div className="strategy-hub-grid">
        <Suspense fallback={<StrategyHubRosterFallback />}>
          <StrategyHubRosterSection model={model} />
        </Suspense>

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
