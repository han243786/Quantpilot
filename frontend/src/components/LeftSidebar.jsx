import { navigateTo } from "../router";

const NAV_ITEMS = [
  [
    { path: "/strategies", label: "策略", icon: "\u{1F4CA}" },
    { path: "/quantscript", label: "QS", icon: "\u{1F4DD}" },
  ],
  [
    { path: "/approvals", label: "审批", icon: "\u{2705}" },
    { path: "/alerts", label: "告警", icon: "\u{26A0}" },
    { path: "/snapshots", label: "快照", icon: "\u{1F4F7}" },
    { path: "/runbook", label: "故障手册", icon: "\u{1F4D6}" },
    { path: "/chaos", label: "混沌", icon: "\u{1F9F0}" },
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
          {section.map((item) => (
            <button
              key={item.path}
              className={`ad-sidebar-item${isActive(item.path) ? " ad-sidebar-item--active" : ""}`}
              onClick={() => navigateTo(item.path)}
              title={item.label}
            >
              <span className="ad-sidebar-item__icon">{item.icon}</span>
              <span className="ad-sidebar-item__label">{item.label}</span>
            </button>
          ))}
        </div>
      ))}
    </nav>
  );
}
