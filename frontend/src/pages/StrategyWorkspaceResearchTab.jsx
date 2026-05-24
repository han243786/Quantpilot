import { Suspense, lazy } from "react";
import { backtestDetailPath, navigateTo } from "../router";
import { EventPanelFallback } from "./StrategyWorkspacePanelFallbacks";

const StrategyResearchConsole = lazy(() => import("../components/StrategyResearchConsole"));

export default function StrategyWorkspaceResearchTab({ strategyId }) {
  return (
    <div className="strategy-workspace-research" data-testid="strategy-workspace-research-tab">
      <Suspense fallback={<EventPanelFallback />}>
        <div className="strategy-workspace-research__panel">
          <StrategyResearchConsole
            onOpenBacktestDetail={(backtestId) =>
              navigateTo(backtestDetailPath(backtestId, strategyId))
            }
          />
        </div>
      </Suspense>
    </div>
  );
}
