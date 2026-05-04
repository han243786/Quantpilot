import { Suspense, lazy, useEffect, useState } from "react";
import { useGraphStore } from "./store/graphStore";
import { parseRoute, strategiesPath } from "./router";
import GlobalNav from "./components/GlobalNav";

const StrategyHubPage = lazy(() => import("./pages/StrategyHubPage"));
const StrategyWorkspacePage = lazy(() => import("./pages/StrategyWorkspacePage"));
const StrategyBacktestsPage = lazy(() => import("./pages/StrategyBacktestsPage"));
const BacktestDetailPage = lazy(() => import("./pages/BacktestDetailPage"));
const BacktestComparePage = lazy(() => import("./pages/BacktestComparePage"));
const ApprovalPage = lazy(() => import("./components/ApprovalPanel"));
const AlertsPage = lazy(() => import("./pages/AlertsPage"));
const SnapshotsPage = lazy(() => import("./pages/SnapshotsPage"));
const RunbookPage = lazy(() => import("./pages/RunbookPage"));
const ChaosPage = lazy(() => import("./pages/ChaosPage"));

function AppShellFallback() {
  return (
    <div className="app-loading-shell" role="status" aria-live="polite">
      <div className="app-loading-shell__title">正在加载界面</div>
      <div className="app-loading-shell__detail">
        正在准备编辑器或分析页面资源。
      </div>
    </div>
  );
}

export default function App() {
  const initialize = useGraphStore((state) => state.initialize);
  const [isInitialized, setIsInitialized] = useState(false);
  const [route, setRoute] = useState(() =>
    parseRoute(
      typeof window === "undefined" ? "/" : window.location.pathname,
      typeof window === "undefined" ? "" : window.location.search
    )
  );

  useEffect(() => {
    let disposed = false;
    void initialize().finally(() => {
      if (!disposed) {
        setIsInitialized(true);
      }
    });
    return () => {
      disposed = true;
    };
  }, [initialize]);

  useEffect(() => {
    const handlePopState = () => {
      setRoute(parseRoute(window.location.pathname, window.location.search));
    };

    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    if (window.location.pathname !== "/" || route.name !== "strategies") return;
    window.history.replaceState({}, "", strategiesPath());
    setRoute(parseRoute(window.location.pathname, window.location.search));
  }, [route.name]);

  if (!isInitialized) {
    return <AppShellFallback />;
  }

  let content = <StrategyHubPage />;
  if (route.name === "approvals") {
    content = <ApprovalPage />;
  } else if (route.name === "alerts") {
    content = <AlertsPage />;
  } else if (route.name === "snapshots") {
    content = <SnapshotsPage />;
  } else if (route.name === "runbook") {
    content = <RunbookPage />;
  } else if (route.name === "chaos") {
    content = <ChaosPage />;
  } else if (route.name === "strategy-workspace") {
    content = <StrategyWorkspacePage strategyId={route.strategyId} />;
  } else if (route.name === "strategy-backtests") {
    content = <StrategyBacktestsPage strategyId={route.strategyId} />;
  } else if (route.name === "backtest-detail") {
    content = <BacktestDetailPage backtestId={route.backtestId} strategyId={route.strategyId} />;
  } else if (route.name === "backtest-compare") {
    content = (
      <BacktestComparePage
        backtestIds={route.backtestIds}
        strategyId={route.strategyId}
      />
    );
  }

  return (
    <>
      <GlobalNav />
      <Suspense fallback={<AppShellFallback />}>{content}</Suspense>
    </>
  );
}
