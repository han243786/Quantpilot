import { useMemo } from "react";
import { useI18n } from "../i18n";
import { useGraphStore } from "../store/graphStore";
import StrategyConfigCockpit from "./StrategyConfigCockpit";

function DashboardCard({ title, children, testId }) {
  return (
    <div className="dashboard-card" data-testid={testId}>
      <div className="dashboard-card-header">{title}</div>
      <div className="dashboard-card-body">{children}</div>
    </div>
  );
}

export default function StrategyWorkspaceDashboard({
  graph,
  runtime,
  compileSummary,
  readiness,
  onNavigate,
  workspaceSurfaces = {}
}) {
  const { t } = useI18n();

  // 直接从 store 订阅最新运行时数据，确保回测后仪表盘自动更新
  const storeRuntime = useGraphStore((s) => s.runtime);
  const latestRuntime = storeRuntime ?? runtime;

  const backtestCount = useMemo(
    () => latestRuntime?.backtestHistory?.length || 0,
    [latestRuntime?.backtestHistory?.length]
  );
  const canNavigateTo = (surfaceKey) => workspaceSurfaces[surfaceKey]?.enabled !== false;
  const navigationTitle = (surfaceKey) =>
    workspaceSurfaces[surfaceKey]?.blockReason ||
    workspaceSurfaces[surfaceKey]?.reason ||
    undefined;

  return (
    <div className="strategy-workspace-dashboard" data-testid="strategy-workspace-dashboard">
      <div className="dashboard-grid">
        <StrategyConfigCockpit
          graph={graph}
          runtime={latestRuntime}
          compileSummary={compileSummary}
        />

        <DashboardCard title={t("编译状态")} testId="dashboard-compile-status">
          <div className="dashboard-metric">
            <span className="dashboard-metric-label">{t("协议")}</span>
            <span className="dashboard-metric-value">
              {compileSummary.protocol_name || t("未编译")}
            </span>
          </div>
          <div className="dashboard-metric">
            <span className="dashboard-metric-label">{t("可编译")}</span>
            <span className={`dashboard-metric-value ${readiness.tone}`}>
              {readiness.label}
            </span>
          </div>
        </DashboardCard>

        <DashboardCard title={t("运行状态")} testId="dashboard-runtime-status">
          <div className="dashboard-metric">
            <span className="dashboard-metric-label">{t("状态")}</span>
            <span className="dashboard-metric-value">{latestRuntime.status || t("空闲")}</span>
          </div>
          {latestRuntime.last_run_equity != null && (
            <div className="dashboard-metric">
              <span className="dashboard-metric-label">{t("最近权益")}</span>
              <span className="dashboard-metric-value">
                {Number(latestRuntime.last_run_equity).toFixed(2)}
              </span>
            </div>
          )}
        </DashboardCard>

        <DashboardCard title={t("最近回测")} testId="dashboard-recent-backtest">
          {backtestCount > 0 ? (
            <div className="dashboard-metric">
              <span className="dashboard-metric-label">{t("回测数")}</span>
              <span className="dashboard-metric-value">{backtestCount}</span>
            </div>
          ) : (
            <div className="dashboard-metric">
              <span className="dashboard-metric-value muted">{t("暂无回测")}</span>
            </div>
          )}
        </DashboardCard>

        <DashboardCard title={t("快速操作")} testId="dashboard-quick-actions">
          <div className="dashboard-actions">
            <button
              className="ad-btn ad-btn--primary"
              onClick={() => onNavigate?.("code")}
              disabled={!canNavigateTo("code")}
              title={navigationTitle("code")}
              data-testid="dashboard-goto-build"
            >
              {t("进入构建")}
            </button>
            <button
              className="ad-btn ad-btn--ghost"
              onClick={() => onNavigate?.("research")}
              disabled={!canNavigateTo("research")}
              title={navigationTitle("research")}
              data-testid="dashboard-goto-research"
            >
              {t("研究回测")}
            </button>
            <button
              className="ad-btn ad-btn--ghost"
              onClick={() => onNavigate?.("monitor")}
              disabled={!canNavigateTo("monitor")}
              title={navigationTitle("monitor")}
              data-testid="dashboard-goto-monitor"
            >
              {t("运行监控")}
            </button>
            <button
              className="ad-btn ad-btn--ghost"
              onClick={() => onNavigate?.("source")}
              disabled={!canNavigateTo("source")}
              title={navigationTitle("source")}
              data-testid="dashboard-goto-source"
            >
              {t("查看源码")}
            </button>
          </div>
        </DashboardCard>
      </div>
    </div>
  );
}
