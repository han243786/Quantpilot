import { lazy, Suspense } from "react";
import AppShellFallback from "./AppShellFallback";
import ErrorBoundary from "../components/ErrorBoundary";

const StrategyHubPage = lazy(() => import("../pages/StrategyHubPage"));
const StrategyWorkspacePage = lazy(() => import("../pages/StrategyWorkspacePage"));
const StrategyBacktestsPage = lazy(() => import("../pages/StrategyBacktestsPage"));
const BacktestDetailPage = lazy(() => import("../pages/BacktestDetailPage"));
const BacktestComparePage = lazy(() => import("../pages/BacktestComparePage"));
const ApprovalPage = lazy(() => import("../components/ApprovalPanel"));
const AlertsPage = lazy(() => import("../pages/AlertsPage"));
const SnapshotsPage = lazy(() => import("../pages/SnapshotsPage"));
const RunbookPage = lazy(() => import("../pages/RunbookPage"));
const ChaosPage = lazy(() => import("../pages/ChaosPage"));
const QuantScriptEditor = lazy(() => import("../pages/QuantScriptEditor"));
const NotFoundPage = lazy(() => import("../pages/NotFoundPage"));
const SettingsPage = lazy(() => import("../pages/SettingsPage"));

function wrapRoute(route, element) {
  return <ErrorBoundary key={route.name}>{element}</ErrorBoundary>;
}

export default function AppRouteHost({ route }) {
  let content = wrapRoute(route, <StrategyHubPage />);
  if (route.name === "approvals") {
    content = wrapRoute(route, <ApprovalPage />);
  } else if (route.name === "alerts") {
    content = wrapRoute(route, <AlertsPage />);
  } else if (route.name === "snapshots") {
    content = wrapRoute(route, <SnapshotsPage />);
  } else if (route.name === "runbook") {
    content = wrapRoute(route, <RunbookPage />);
  } else if (route.name === "chaos") {
    content = wrapRoute(route, <ChaosPage />);
  } else if (route.name === "quantscript") {
    content = wrapRoute(route, <QuantScriptEditor />);
  } else if (route.name === "settings") {
    content = wrapRoute(route, <SettingsPage />);
  } else if (route.name === "strategy-workspace") {
    content = wrapRoute(route, <StrategyWorkspacePage strategyId={route.strategyId} />);
  } else if (route.name === "strategy-backtests") {
    content = wrapRoute(route, <StrategyBacktestsPage strategyId={route.strategyId} />);
  } else if (route.name === "backtest-detail") {
    content = wrapRoute(
      route,
      <BacktestDetailPage backtestId={route.backtestId} strategyId={route.strategyId} />
    );
  } else if (route.name === "backtest-compare") {
    content = wrapRoute(
      route,
      <BacktestComparePage
        backtestIds={route.backtestIds}
        strategyId={route.strategyId}
      />
    );
  } else if (route.name === "not-found") {
    content = wrapRoute(route, <NotFoundPage pathname={route.pathname} />);
  }

  return <Suspense fallback={<AppShellFallback />}>{content}</Suspense>;
}
