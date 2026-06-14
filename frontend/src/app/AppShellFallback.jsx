import { useEffect, useState } from "react";
import { useGraphStore } from "../store/graphStore";
import { useI18n } from "../i18n";

export default function AppShellFallback({ onSkip }) {
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
        <button className="ad-btn ad-btn--ghost" onClick={onSkip} style={{ marginTop: 16 }}>
          {t("跳过等待，使用本地缓存")}
        </button>
      )}
    </div>
  );
}
