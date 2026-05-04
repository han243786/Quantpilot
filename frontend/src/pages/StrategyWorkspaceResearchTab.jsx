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
        title="模拟与回测控制"
        subtitle="现有工具栏保留在这里，但范围收敛到研究模式，不再占据页面顶部。"
        testId="workspace-run-backtest-controls-section"
        actions={
          <button
            className="ghost-btn compact-btn"
            onClick={() => navigateTo(strategyBacktestsPath(strategyId))}
          >
            打开策略回测
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
