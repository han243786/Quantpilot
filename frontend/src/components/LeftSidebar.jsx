import { navigateTo } from "../router";
import { useI18n } from "../i18n";
import { IconChart, IconCode, IconCheck, IconAlert, IconCamera, IconBook, IconFlask, IconSettings } from "./Icons";

const NAV_ITEMS = [
  [
    { path: "/strategies", labelKey: "策略", Icon: IconChart },
    { path: "/quantscript", labelKey: "QuantScript", Icon: IconCode },
  ],
  [
    { path: "/approvals", labelKey: "审批", Icon: IconCheck },
    { path: "/alerts", labelKey: "告警", Icon: IconAlert },
    { path: "/snapshots", labelKey: "快照", Icon: IconCamera },
    { path: "/runbook", labelKey: "故障手册", Icon: IconBook },
    { path: "/chaos", labelKey: "混沌", Icon: IconFlask },
    { path: "/settings", labelKey: "设置", Icon: IconSettings },
  ],
];

export default function LeftSidebar() {
  const { t } = useI18n();
  const current =
    typeof window !== "undefined" ? window.location.pathname : "";

  const isActive = (path) =>
    current === path ||
    current.startsWith(path + "/") ||
    current.startsWith(path + "?");

  return (
    <nav className="ad-sidebar" data-testid="app-sidebar">
      <div className="ad-sidebar-brand">
        <span className="ad-sidebar-brand__icon">QP</span>
        <span className="ad-sidebar-brand__text">QuantPilot</span>
      </div>

      {NAV_ITEMS.map((section, si) => (
        <div key={si} className="ad-sidebar-section">
          {si > 0 && <div className="ad-sidebar-divider" />}
          {section.map(({ path, labelKey, Icon }) => (
            <a
              key={path}
              href={path}
              className={`ad-sidebar-item${isActive(path) ? " ad-sidebar-item--active" : ""}`}
              onClick={(e) => { e.preventDefault(); navigateTo(path); }}
              aria-current={isActive(path) ? "page" : undefined}
              title={t(labelKey)}
            >
              <span className="ad-sidebar-item__icon" aria-hidden="true"><Icon /></span>
              <span className="ad-sidebar-item__label">{t(labelKey)}</span>
            </a>
          ))}
        </div>
      ))}
    </nav>
  );
}
