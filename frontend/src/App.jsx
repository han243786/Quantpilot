import { useCallback, useRef, useState } from "react";
import { navigateTo, strategiesPath } from "./router";
import AppGlobalOverlays from "./app/AppGlobalOverlays";
import AppRouteHost from "./app/AppRouteHost";
import DesktopTitleBar from "./app/DesktopTitleBar";
import AppShellFallback from "./app/AppShellFallback";
import { useAppEnvironmentEvents } from "./app/useAppEnvironmentEvents";
import { useAppInitialization } from "./app/useAppInitialization";
import { useAppRoute } from "./app/useAppRoute";
import { useDesktopWindowChrome } from "./app/useDesktopWindowChrome";
import LeftSidebar from "./components/LeftSidebar";
import { useI18n } from "./i18n";

export default function App() {
  const isInitialized = useAppInitialization();
  const { t } = useI18n();
  const [forceReady, setForceReady] = useState(false);
  const [cmdPaletteOpen, setCmdPaletteOpen] = useState(false);
  const mainRef = useRef(null);
  const { appWindow, isMaximized } = useDesktopWindowChrome();
  const route = useAppRoute();
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

  if (!isInitialized && !forceReady) {
    return <AppShellFallback onSkip={() => setForceReady(true)} />;
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
        <AppRouteHost route={route} />
      </main>
      <AppGlobalOverlays
        commandPaletteOpen={cmdPaletteOpen}
        onCloseCommandPalette={() => setCmdPaletteOpen(false)}
      />
    </>
  );
}
