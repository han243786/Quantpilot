import { useI18n } from "../i18n";

export default function DesktopTitleBar({ appWindow, isMaximized }) {
  const { t } = useI18n();
  if (!appWindow) return null;

  return (
    <div className="ad-titlebar" data-tauri-drag-region>
      <span className="ad-titlebar-title">QuantPilot</span>
      <div className="ad-titlebar-controls">
        <button
          className="ad-titlebar-btn"
          onClick={() => appWindow.minimize()}
          aria-label={t("最小化")}
        >
          —
        </button>
        <button
          className="ad-titlebar-btn"
          onClick={() => appWindow.toggleMaximize()}
          aria-label={t("最大化")}
        >
          {isMaximized ? "□" : "❐"}
        </button>
        <button
          className="ad-titlebar-btn ad-titlebar-btn--close"
          onClick={() => appWindow.close()}
          aria-label={t("关闭")}
        >
          ✕
        </button>
      </div>
    </div>
  );
}
