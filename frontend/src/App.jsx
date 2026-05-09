import { Suspense, lazy, useEffect, useRef, useState } from "react";
import { useGraphStore } from "./store/graphStore";
import { parseRoute, strategiesPath } from "./router";
import LeftSidebar from "./components/LeftSidebar";
import CommandPalette from "./components/CommandPalette";

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
import TutorialOverlay from "./components/TutorialOverlay";
import { createTutorialSteps } from "./data/tutorialSteps";
import { useTutorial } from "./hooks/useTutorial";
import { useI18n } from "./i18n";

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
  const { tutorialOpen, closeTutorial } = useTutorial();
  const { t } = useI18n();
  const tutorialSteps = createTutorialSteps(t);
  const [isInitialized, setIsInitialized] = useState(false);
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false);
  const [isOffline, setIsOffline] = useState(
    typeof navigator !== "undefined" ? !navigator.onLine : false
  );
  const mainRef = useRef(null);
  const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  function tauriWindow(action) {
    if (!isTauri) return;
    window.__TAURI_INTERNALS__?.invoke?.(`plugin:window|${action}`);
  }

  // 离线/在线检测
  useEffect(() => {
    const goOffline = () => setIsOffline(true);
    const goOnline = () => setIsOffline(false);
    window.addEventListener("offline", goOffline);
    window.addEventListener("online", goOnline);
    return () => {
      window.removeEventListener("offline", goOffline);
      window.removeEventListener("online", goOnline);
    };
  }, []);

  // ⌘K / Ctrl+K 全局监听
  useEffect(() => {
    const handleKey = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setCmdPaletteOpen((v) => !v);
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);
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
  } else if (route.name === "quantscript") {
    content = <QuantScriptEditor />;
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
      {isTauri ? (
        <div className="ad-titlebar" data-tauri-drag-region>
          <span className="ad-titlebar-title">QuantPilot</span>
          <div className="ad-titlebar-controls">
            <button className="ad-titlebar-btn" onClick={() => tauriWindow("minimize")} aria-label="最小化">—</button>
            <button className="ad-titlebar-btn" onClick={() => tauriWindow("internal_toggle_maximize")} aria-label="最大化">□</button>
            <button className="ad-titlebar-btn ad-titlebar-btn--close" onClick={() => tauriWindow("close")} aria-label="关闭">✕</button>
          </div>
        </div>
      ) : null}
      <LeftSidebar />
      {isOffline ? (
        <div className="ad-offline-banner" role="alert">
          网络连接已断开，部分功能不可用。
        </div>
      ) : null}
      <a href="#main-content" className="ad-skip-link">跳转到内容</a>
      <main id="main-content" className="ad-main-content" ref={mainRef} tabIndex={-1} style={isTauri ? { marginTop: 32, height: "calc(100% - 32px)" } : {}}>
        <Suspense fallback={<AppShellFallback />}>{content}</Suspense>
      </main>
      {tutorialOpen && (
        <TutorialOverlay steps={tutorialSteps} onClose={closeTutorial} />
      )}
      <CommandPalette open={cmdPaletteOpen} onClose={() => setCmdPaletteOpen(false)} />
    </>
  );
}
