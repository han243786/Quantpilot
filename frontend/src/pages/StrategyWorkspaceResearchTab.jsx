import { Suspense, lazy } from "react";
import TopToolbar from "../components/TopToolbar";
import { backtestDetailPath, navigateTo, strategyBacktestsPath } from "../router";
import { WorkspaceSection } from "./StrategyWorkspacePageSections";
import { EventPanelFallback } from "./StrategyWorkspacePanelFallbacks";

const StrategyResearchConsole = lazy(() => import("../components/StrategyResearchConsole"));

export default function StrategyWorkspaceResearchTab({ strategyId }) {
  return (
    <div className="strategy-workspace-research" data-testid="strategy-workspace-research-tab">
      <WorkspaceSection
        title="Run and backtest controls"
        subtitle="The existing toolbar remains here, but it is scoped to research mode instead of the top of the page."
        testId="workspace-run-backtest-controls-section"
        actions={
          <button
            className="ghost-btn compact-btn"
            onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
          >
            Open strategy backtests
          </button>
        }
      >
        <div className="workspace-toolbar-shell">
          <TopToolbar variant="workspace" />
        </div>
      </WorkspaceSection>

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
