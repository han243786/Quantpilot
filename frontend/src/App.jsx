import { Suspense, lazy, useCallback, useEffect, useRef, useState } from "react";
import { useGraphStore } from "./store/graphStore";
import { navigateTo, parseRoute, strategiesPath } from "./router";
import LeftSidebar from "./components/LeftSidebar";
import CommandPalette from "./components/CommandPalette";
import { useI18n } from "./i18n";
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
const NotFoundPage = lazy(() => import("./pages/NotFoundPage"));
const SettingsPage = lazy(() => import("./pages/SettingsPage"));
import TutorialOverlay from "./components/TutorialOverlay";
import ToastContainer from "./components/ToastContainer";
import { createTutorialSteps } from "./data/tutorialSteps";
import { useTutorial } from "./hooks/useTutorial";

function resolveTauriWindow() {
  const isTauriRuntime =
    typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
  if (!isTauriRuntime) return null;
  try {
    return getCurrentWindow();
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn("[App] Tauri 窗口 API 初始化失败:", error.message);
    }
    return null;
  }
}

function AppShellFallback({ onSkip }) {
  const { t } = useI18n();
  const [waited, setWaited] = useState(false);
  const capabilityStatus = useGraphStore((s) => s.capabilityStatus);
  useEffect(() => {
    const t = setTimeout(() => setWaited(true), 5000);
    return () => clearTimeout(t);
  }, []);

  const STAGE_TEXT = {
    loading: t("正在连接后端..."),
    degraded: t("已加载本地缓存"),
    error: t("后端连接失败，已进入离线模式"),
  };
  const stageText = STAGE_TEXT[capabilityStatus] || t("正在准备编辑器...");

  return (
    <div className="app-loading-shell" role="status" aria-live="polite">
      <div className="app-loading-shell__skeleton">
        <div className="skeleton-block skeleton-block--wide" />
        <div className="skeleton-block skeleton-block--medium" />
        <div className="skeleton-block skeleton-block--short" />
      </div>
      <div className="app-loading-shell__title">{stageText}</div>
      {waited && onSkip && (
        <button className="ad-btn ad-btn--ghost" onClick={onSkip} style={{marginTop:16}}>
          {t("跳过等待，使用本地缓存")}
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
  const appWindow = resolveTauriWindow();

  useEffect(() => {
    const applyStoredTheme = () => {
      try {
        const theme = window.localStorage?.getItem("quantpilot.theme") || "auto";
        if (theme === "light" || theme === "dark") {
          document.documentElement.dataset.theme = theme;
        } else {
          document.documentElement.removeAttribute("data-theme");
        }
      } catch (_) {
        document.documentElement.removeAttribute("data-theme");
      }
    };
    applyStoredTheme();
    window.addEventListener("qp-theme-change", applyStoredTheme);
    return () => window.removeEventListener("qp-theme-change", applyStoredTheme);
  }, []);

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
        // v2.4.0 U7: 根据当前 locale 选择提示语言
        const locale = localStorage.getItem("quantpilot.locale") || "zh-CN";
        e.returnValue = locale === "en-US"
          ? "You have unsaved strategy changes. Leaving this page will discard your changes."
          : "当前有未保存的策略图更改，离开此页面将丢失更改。";
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
      {appWindow ? (
        <div className="ad-titlebar" data-tauri-drag-region>
          <span className="ad-titlebar-title">QuantPilot</span>
          <div className="ad-titlebar-controls">
            <button className="ad-titlebar-btn" onClick={() => appWindow.minimize()} aria-label={t("最小化")}>—</button>
            <button className="ad-titlebar-btn" onClick={() => appWindow.toggleMaximize()} aria-label={t("最大化")}>{isMaximized ? "□" : "❐"}</button>
            <button className="ad-titlebar-btn ad-titlebar-btn--close" onClick={() => appWindow.close()} aria-label={t("关闭")}>✕</button>
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
