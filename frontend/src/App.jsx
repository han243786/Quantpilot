import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { navigateTo, parseRoute, strategiesPath } from "./router";
import DesktopTitleBar from "./app/DesktopTitleBar";
import AppShellFallback from "./app/AppShellFallback";
import { useAppEnvironmentEvents } from "./app/useAppEnvironmentEvents";
import { useAppInitialization } from "./app/useAppInitialization";
import { useDesktopWindowChrome } from "./app/useDesktopWindowChrome";
import LeftSidebar from "./components/LeftSidebar";
import CommandPalette from "./components/CommandPalette";
import { useI18n } from "./i18n";
import ErrorBoundary from "./components/ErrorBoundary";

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
const QuantScriptEditor = lazy(() => import("./pages/QuantScriptEditor"));
const NotFoundPage = lazy(() => import("./pages/NotFoundPage"));
const SettingsPage = lazy(() => import("./pages/SettingsPage"));
import TutorialOverlay from "./components/TutorialOverlay";
import ToastContainer from "./components/ToastContainer";
import { createTutorialSteps } from "./data/tutorialSteps";
import { useTutorial } from "./hooks/useTutorial";

export default function App() {
  const isInitialized = useAppInitialization();
  const { tutorialOpen, closeTutorial } = useTutorial();
  const { t } = useI18n();
  const tutorialSteps = createTutorialSteps(t);
  const [forceReady, setForceReady] = useState(false);
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false);
  const mainRef = useRef(null);
  const { appWindow, isMaximized } = useDesktopWindowChrome();
  const [route, setRoute] = useState(() =>
    parseRoute(
      typeof window === "undefined" ? "/" : window.location.pathname,
      typeof window === "undefined" ? "" : window.location.search
    )
  );
  const toggleCommandPalette = useCallback(() => {
    setCmdPaletteOpen((value) => !value);
  }, []);
  const {
    isOffline,
    storageQuotaExceeded,
    setStorageQuotaExceeded,
  } = useAppEnvironmentEvents({
    route,
    onToggleCommandPalette: toggleCommandPalette,
  });

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

  if (!isInitialized && !forceReady) {
    return <AppShellFallback onSkip={() => setForceReady(true)} />;
  }

  // v1.1.7: 每个路由独立 ErrorBoundary，单页崩溃不影响全局
  const wrapRoute = (el) => <ErrorBoundary key={route.name}>{el}</ErrorBoundary>;
  let content = wrapRoute(<StrategyHubPage />);
  if (route.name === "approvals") {
    content = wrapRoute(<ApprovalPage />);
  } else if (route.name === "alerts") {
    content = wrapRoute(<AlertsPage />);
  } else if (route.name === "snapshots") {
    content = wrapRoute(<SnapshotsPage />);
  } else if (route.name === "runbook") {
    content = wrapRoute(<RunbookPage />);
  } else if (route.name === "chaos") {
    content = wrapRoute(<ChaosPage />);
  } else if (route.name === "quantscript") {
    content = wrapRoute(<QuantScriptEditor />);
  } else if (route.name === "settings") {
    content = wrapRoute(<SettingsPage />);
  } else if (route.name === "strategy-workspace") {
    content = wrapRoute(<StrategyWorkspacePage strategyId={route.strategyId} />);
  } else if (route.name === "strategy-backtests") {
    content = wrapRoute(<StrategyBacktestsPage strategyId={route.strategyId} />);
  } else if (route.name === "backtest-detail") {
    content = wrapRoute(<BacktestDetailPage backtestId={route.backtestId} strategyId={route.strategyId} />);
  } else if (route.name === "backtest-compare") {
    content = wrapRoute(
      <BacktestComparePage
        backtestIds={route.backtestIds}
        strategyId={route.strategyId}
      />
    );
  } else if (route.name === "not-found") {
    content = wrapRoute(<NotFoundPage pathname={route.pathname} />);
  }

  return (
    <>
      <DesktopTitleBar appWindow={appWindow} isMaximized={isMaximized} />
      <LeftSidebar />
      {!appWindow && isOffline ? (
        <div className="ad-offline-banner" role="alert">
          {t("网络连接已断开，部分功能不可用。")}
        </div>
      ) : null}
      {storageQuotaExceeded ? (
        <div className="ad-offline-banner" role="alert" style={{background:"var(--ad-warning-soft)",color:"var(--ad-warning)"}}>
          {t("本地存储空间不足，策略图未保存。请前往策略中心，清理不需要的策略图旧版本以释放空间。")}
          <button className="ad-btn ad-btn--ghost" style={{marginLeft:12,textDecoration:"underline"}} onClick={() => { setStorageQuotaExceeded(false); navigateTo(strategiesPath()); }}>
            {t("前往策略中心")}
          </button>
        </div>
      ) : null}
      <a href="#main-content" className="ad-skip-link">{t("跳转到内容")}</a>
      <main id="main-content" className="ad-main-content" ref={mainRef} tabIndex={-1} style={appWindow ? { marginTop: 32, height: "calc(100% - 32px)" } : {}}>
        <Suspense fallback={<AppShellFallback />}>{content}</Suspense>
      </main>
      {tutorialOpen && (
        <TutorialOverlay steps={tutorialSteps} onClose={closeTutorial} />
      )}
      <CommandPalette open={cmdPaletteOpen} onClose={() => setCmdPaletteOpen(false)} />
      <ToastContainer />
    </>
  );
}
