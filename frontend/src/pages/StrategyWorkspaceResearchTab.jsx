import { Suspense, lazy } from "react";
import { backtestDetailPath, navigateTo } from "../router";
import { EventPanelFallback } from "./StrategyWorkspacePanelFallbacks";
import { useI18n } from "../i18n";
import { buildWorkspaceResearchStripModel } from "./strategyWorkspaceAuxiliaryTabsShell";

const StrategyResearchConsole = lazy(() => import("../components/StrategyResearchConsole"));

export default function StrategyWorkspaceResearchTab({ strategyId }) {
  const { t } = useI18n();
  const stripModel = buildWorkspaceResearchStripModel(t);

  return (
    <div className="strategy-workspace-research" data-testid="strategy-workspace-research-tab">
      <div className="workspace-mode-strip">
        <strong>{stripModel.title}</strong>
        <div className="workspace-mode-strip__pills">
          {stripModel.pills.map((pill) => (
            <span key={`${pill.tone}-${pill.label}`} className={`status-pill ${pill.tone}`}>
              {pill.label}
            </span>
          ))}
        </div>
      </div>
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
