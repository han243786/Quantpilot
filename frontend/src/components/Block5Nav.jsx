import React from "react";
import { navigateTo } from "../router";

const LINKS = [
  { path: "/approvals", label: "审批队列" },
  { path: "/alerts", label: "告警面板" },
  { path: "/snapshots", label: "签名快照" },
  { path: "/runbook", label: "故障手册" },
  { path: "/chaos", label: "混沌实验" },
];

export default function Block5Nav() {
  const current =
    typeof window !== "undefined" ? window.location.pathname : "";

  return (
    <nav className="qp-subnav">
      {LINKS.map((link) => (
        <button
          key={link.path}
          onClick={() => navigateTo(link.path)}
          className={current === link.path ? "active" : ""}
        >
          {link.label}
        </button>
      ))}
    </nav>
  );
}
