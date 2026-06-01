import { navigateTo } from "../router";
import { useI18n } from "../i18n";
import {
  SHELL_NAV_SECTIONS,
  isShellNavPathActive,
} from "../routing/shellNavigation";
import { IconChart, IconCode, IconCheck, IconAlert, IconCamera, IconBook, IconFlask, IconSettings } from "./Icons";

const ICONS_BY_KEY = {
  alert: IconAlert,
  book: IconBook,
  camera: IconCamera,
  chart: IconChart,
  check: IconCheck,
  code: IconCode,
  flask: IconFlask,
  settings: IconSettings,
};

export default function LeftSidebar() {
  const { t } = useI18n();
  const current =
    typeof window !== "undefined" ? window.location.pathname : "";

  return (
    <nav className="ad-sidebar" data-testid="app-sidebar">
      <div className="ad-sidebar-brand">
        <span className="ad-sidebar-brand__icon">QP</span>
        <span className="ad-sidebar-brand__text">QuantPilot</span>
      </div>

      {SHELL_NAV_SECTIONS.map((section, si) => (
        <div key={si} className="ad-sidebar-section">
          {si > 0 && <div className="ad-sidebar-divider" />}
          {section.map(({ id, path, labelKey, iconKey }) => {
            const Icon = ICONS_BY_KEY[iconKey];
            const isActive = isShellNavPathActive(current, path);
            return (
              <a
                key={id}
                href={path}
                className={`ad-sidebar-item${isActive ? " ad-sidebar-item--active" : ""}`}
                onClick={(e) => { e.preventDefault(); navigateTo(path); }}
                aria-current={isActive ? "page" : undefined}
                title={t(labelKey)}
              >
                <span className="ad-sidebar-item__icon" aria-hidden="true"><Icon /></span>
                <span className="ad-sidebar-item__label">{t(labelKey)}</span>
              </a>
            );
          })}
        </div>
      ))}
    </nav>
  );
}
