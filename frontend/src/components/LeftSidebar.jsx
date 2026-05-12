import { navigateTo } from "../router";
import { IconChart, IconCode, IconCheck, IconAlert, IconCamera, IconBook, IconFlask } from "./Icons";

const NAV_ITEMS = [
  [
    { path: "/strategies", label: "策略", Icon: IconChart },
    { path: "/quantscript", label: "QS", Icon: IconCode },
  ],
  [
    { path: "/approvals", label: "审批", Icon: IconCheck },
    { path: "/alerts", label: "告警", Icon: IconAlert },
    { path: "/snapshots", label: "快照", Icon: IconCamera },
    { path: "/runbook", label: "故障手册", Icon: IconBook },
    { path: "/chaos", label: "混沌", Icon: IconFlask },
  ],
];

export default function LeftSidebar() {
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
          {section.map(({ path, label, Icon }) => (
            <a
              key={path}
              href={path}
              className={`ad-sidebar-item${isActive(path) ? " ad-sidebar-item--active" : ""}`}
              onClick={(e) => { e.preventDefault(); navigateTo(path); }}
              aria-current={isActive(path) ? "page" : undefined}
              title={label}
            >
              <span className="ad-sidebar-item__icon" aria-hidden="true"><Icon /></span>
              <span className="ad-sidebar-item__label">{label}</span>
            </a>
          ))}
        </div>
      ))}
    </nav>
  );
}
