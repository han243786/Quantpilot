import React from "react";
import { navigateTo } from "../router";

const SECTIONS = [
  { path: "/strategies", label: "策略" },
];

const BLOCK5 = [
  { path: "/approvals", label: "审批" },
  { path: "/alerts", label: "告警" },
  { path: "/snapshots", label: "快照" },
  { path: "/runbook", label: "故障手册" },
  { path: "/chaos", label: "混沌" },
];

export default function GlobalNav() {
  const current =
    typeof window !== "undefined" ? window.location.pathname : "";

  const isActive = (path) =>
    current === path || current.startsWith(path + "/") || current.startsWith(path + "?");

  return (
    <nav className="qp-global-nav">
      <span className="qp-global-nav__brand">
        Quant<span>Pilot</span>
      </span>

      {SECTIONS.map((s) => (
        <button
          key={s.path}
          onClick={() => navigateTo(s.path)}
          className={isActive(s.path) ? "active" : ""}
        >
          {s.label}
        </button>
      ))}

      <span className="qp-global-nav__sep" />

      {BLOCK5.map((s) => (
        <button
          key={s.path}
          onClick={() => navigateTo(s.path)}
          className={isActive(s.path) ? "active" : ""}
        >
          {s.label}
        </button>
      ))}
    </nav>
  );
}
