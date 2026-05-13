import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { useGraphStore } from "./store/graphStore";
import { navigateTo, parseRoute, strategiesPath } from "./router";
import LeftSidebar from "./components/LeftSidebar";
import CommandPalette from "./components/CommandPalette";
import ErrorBoundary from "./components/ErrorBoundary";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

function AppShellFallback({ onSkip }) {
  const [waited, setWaited] = useState(false);
  const capabilityStatus = useGraphStore((s) => s.capabilityStatus);
  useEffect(() => {
    const t = setTimeout(() => setWaited(true), 5000);
    return () => clearTimeout(t);
  }, []);

  const stageText =
    capabilityStatus === "loading" ? "正在连接后端..." :
    capabilityStatus === "degraded" ? "已加载本地缓存" :
    capabilityStatus === "error" ? "后端连接失败，已进入离线模式" :
    "正在准备编辑器...";

  return (
    <div className="app-loading-shell" role="status" aria-live="polite">
      <div className="app-loading-shell__skeleton">
        <div className="skeleton-block skeleton-block--wide" />
        <div className="skeleton-block skeleton-block--medium" />
        <div className="skeleton-block skeleton-block--short" />
      </div>
      <div className="app-loading-shell__title">{stageText}</div>
      {waited && onSkip && (
        <button className="ghost-btn" onClick={onSkip} style={{marginTop:16}}>
          跳过等待，使用本地缓存
        </button>
      )}
    </div>
  );
}

export default function App() {
  const initialize = useGraphStore((state) => state.initialize);
  const { tutorialOpen, closeTutorial } = useTutorial();
  const { t } = useI18n();
  const tutorialSteps = createTutorialSteps(t);
  const [isInitialized, setIsInitialized] = useState(false);
  const [forceReady, setForceReady] = useState(false);
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false);
  const [isOffline, setIsOffline] = useState(
    typeof navigator !== "undefined" ? !navigator.onLine : false
  );
  const [storageQuotaExceeded, setStorageQuotaExceeded] = useState(false);
  const mainRef = useRef(null);
  const [isMaximized, setIsMaximized] = useState(false);
  let appWindow = null;
  try { appWindow = getCurrentWindow(); } catch (e) { if (import.meta.env.DEV) console.warn("[App] Tauri API 不可用，使用浏览器模式:", e.message); }

  // 监听窗口最大化状态
  useEffect(() => {
    if (!appWindow) return;
    let disposed = false;
    appWindow.isMaximized().then((v) => { if (!disposed) setIsMaximized(v); });
    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then((v) => { if (!disposed) setIsMaximized(v); });
    });
    return () => { disposed = true; unlisten.then((fn) => fn()); };
  }, []);

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

  // v1.0.5: localStorage 配额超限通知
  useEffect(() => {
    const handler = () => setStorageQuotaExceeded(true);
    window.addEventListener("qp-storage-quota-exceeded", handler);
    return () => window.removeEventListener("qp-storage-quota-exceeded", handler);
  }, []);

  // v1.0.5: 标签页可见性变化 — 后台时标记, 前台时同步刷新
  useEffect(() => {
    const handle = () => {
      if (!document.hidden) {
        useGraphStore.getState().refreshGraphIndex?.();
      }
    };
    document.addEventListener("visibilitychange", handle);
    return () => document.removeEventListener("visibilitychange", handle);
  }, []);

  const [route, setRoute] = useState(() =>
    parseRoute(
      typeof window === "undefined" ? "/" : window.location.pathname,
      typeof window === "undefined" ? "" : window.location.search
    )
  );

  // 未保存更改时关闭/刷新提醒
  const routeRef = useRef(route);
  routeRef.current = route;
  useEffect(() => {
    const handler = (e) => {
      const name = routeRef.current.name;
      if (name === "strategy-workspace" || name === "quantscript") {
        e.preventDefault();
        e.returnValue = "当前有未保存的策略图更改，离开此页面将丢失更改。";
      }
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
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

  if (!isInitialized && !forceReady) {
    return <AppShellFallback onSkip={() => setForceReady(true)} />;
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
      {appWindow ? (
        <div className="ad-titlebar" data-tauri-drag-region>
          <span className="ad-titlebar-title">QuantPilot</span>
          <div className="ad-titlebar-controls">
            <button className="ad-titlebar-btn" onClick={() => appWindow.minimize()} aria-label="最小化">—</button>
            <button className="ad-titlebar-btn" onClick={() => appWindow.toggleMaximize()} aria-label="最大化">{isMaximized ? "□" : "❐"}</button>
            <button className="ad-titlebar-btn ad-titlebar-btn--close" onClick={() => appWindow.close()} aria-label="关闭">✕</button>
          </div>
        </div>
      ) : null}
      <LeftSidebar />
      {!appWindow && isOffline ? (
        <div className="ad-offline-banner" role="alert">
          {t("网络连接已断开，部分功能不可用。")}
        </div>
      ) : null}
      {storageQuotaExceeded ? (
        <div className="ad-offline-banner" role="alert" style={{background:"var(--ad-warning-soft)",color:"var(--ad-warning)"}}>
          本地存储空间不足，策略图未保存。请前往策略中心，清理不需要的策略图旧版本以释放空间。
          <button className="ghost-btn" style={{marginLeft:12,textDecoration:"underline"}} onClick={() => { setStorageQuotaExceeded(false); navigateTo(strategiesPath()); }}>
            前往策略中心
          </button>
        </div>
      ) : null}
      <a href="#main-content" className="ad-skip-link">跳转到内容</a>
      <main id="main-content" className="ad-main-content" ref={mainRef} tabIndex={-1} style={appWindow ? { marginTop: 32, height: "calc(100% - 32px)" } : {}}>
        <ErrorBoundary>
          <Suspense fallback={<AppShellFallback />}>{content}</Suspense>
        </ErrorBoundary>
      </main>
      {tutorialOpen && (
        <TutorialOverlay steps={tutorialSteps} onClose={closeTutorial} />
      )}
      <CommandPalette open={cmdPaletteOpen} onClose={() => setCmdPaletteOpen(false)} />
    </>
  );
}
