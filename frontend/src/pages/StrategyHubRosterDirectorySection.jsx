import { Suspense, lazy } from "react";
import { StrategyHubInspectorSectionFallback } from "./StrategyHubSectionFallbacks";

const StrategyHubRosterToolbar = lazy(() => import("./StrategyHubRosterToolbar"));
const StrategyHubRosterTableSection = lazy(() => import("./StrategyHubRosterTableSection"));

export default function StrategyHubRosterDirectorySection({ model, toolbar, rosterRows }) {
  return (
    <section className="strategy-directory-card">
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">策略清单</div>
          <div className="strategy-card-subtitle">
            让列表保持在总览管理层面：先筛查策略，再只在必要时深入到单个工作区。
          </div>
        </div>
        <div className="status-pill muted">显示 {toolbar.filteredCountLabel} 条</div>
      </div>

      <Suspense fallback={<StrategyHubInspectorSectionFallback title="批量操作" />}>
        <StrategyHubRosterToolbar model={model} toolbar={toolbar} />
      </Suspense>

      <Suspense fallback={<StrategyHubInspectorSectionFallback title="策略清单" />}>
        <StrategyHubRosterTableSection model={model} rosterRows={rosterRows} />
      </Suspense>
    </section>
  );
}
