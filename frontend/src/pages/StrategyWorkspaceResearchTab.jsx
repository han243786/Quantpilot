import { Suspense, lazy } from "react";
import { backtestDetailPath, navigateTo } from "../router";
import { EventPanelFallback } from "./StrategyWorkspacePanelFallbacks";
import { useI18n } from "../i18n";

const StrategyResearchConsole = lazy(() => import("../components/StrategyResearchConsole"));

export default function StrategyWorkspaceResearchTab({ strategyId }) {
  const { t } = useI18n();

  return (
    <div className="strategy-workspace-research" data-testid="strategy-workspace-research-tab">
      <div className="workspace-mode-strip">
        <strong>{t("研究回测工作区")}</strong>
        <div className="workspace-mode-strip__pills">
          <span className="status-pill muted">{t("结果")}</span>
          <span className="status-pill info">{t("时间线")}</span>
          <span className="status-pill muted">{t("详情")}</span>
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
