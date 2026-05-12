export function strategiesPath() {
  return "/strategies";
}

export function strategyWorkspacePath(strategyId) {
  return `/strategies/${encodeURIComponent(strategyId)}`;
}

export function strategyBacktestsPath(strategyId) {
  return `/strategies/${encodeURIComponent(strategyId)}/backtests`;
}

function appendStrategyContext(pathname, strategyId) {
  if (!strategyId) return pathname;
  const query = new URLSearchParams({ strategy: strategyId });
  return `${pathname}?${query.toString()}`;
}

export function backtestDetailPath(backtestId, strategyId = "") {
  return appendStrategyContext(`/backtests/${encodeURIComponent(backtestId)}`, strategyId);
}

export function approvalsPath() {
  return "/approvals";
}

export function alertsPath() {
  return "/alerts";
}

export function snapshotsPath() {
  return "/snapshots";
}

export function runbookPath() {
  return "/runbook";
}

export function chaosPath() {
  return "/chaos";
}

export function quantscriptPath() {
  return "/quantscript";
}

export function backtestComparePath(backtestIds, strategyId = "") {
  const ids = [...new Set((backtestIds || []).filter(Boolean))];
  const query = new URLSearchParams({ ids: ids.join(",") });
  if (strategyId) {
    query.set("strategy", strategyId);
  }
  return `/backtests/compare?${query.toString()}`;
}

export function parseRoute(pathname, search = "") {
  if (pathname === "/" || pathname === "/strategies") {
    return { name: "strategies" };
  }

  if (pathname === "/approvals") {
    return { name: "approvals" };
  }

  if (pathname === "/alerts") {
    return { name: "alerts" };
  }

  if (pathname === "/snapshots") {
    return { name: "snapshots" };
  }

  if (pathname === "/runbook") {
    return { name: "runbook" };
  }

  if (pathname === "/chaos") {
    return { name: "chaos" };
  }

  if (pathname === "/quantscript") {
    return { name: "quantscript" };
  }

  const strategyBacktestsMatch = pathname.match(/^\/strategies\/([^/]+)\/backtests$/);
  if (strategyBacktestsMatch) {
    return {
      name: "strategy-backtests",
      strategyId: decodeURIComponent(strategyBacktestsMatch[1])
    };
  }

  const strategyMatch = pathname.match(/^\/strategies\/([^/]+)$/);
  if (strategyMatch) {
    const decoded = decodeURIComponent(strategyMatch[1]);
    if (decoded.includes("\x00") || decoded.length > 128) {
      return { name: "strategies" };
    }
    return {
      name: "strategy-workspace",
      strategyId: decoded
    };
  }

  if (pathname === "/backtests/compare") {
    const params = new URLSearchParams(search);
    const backtestIds = (params.get("ids") || "")
      .split(",")
      .map((item) => decodeURIComponent(item.trim()))
      .filter(Boolean);
    const strategyId = params.get("strategy")
      ? decodeURIComponent(params.get("strategy"))
      : "";
    return {
      name: "backtest-compare",
      backtestIds,
      strategyId
    };
  }

  const match = pathname.match(/^\/backtests\/([^/]+)$/);
  if (match) {
    const params = new URLSearchParams(search);
    return {
      name: "backtest-detail",
      backtestId: decodeURIComponent(match[1]),
      strategyId: params.get("strategy")
        ? decodeURIComponent(params.get("strategy"))
        : ""
    };
  }

  return { name: "strategies" };
}

let _lastNavPath = "";
let _lastNavTime = 0;

export function navigateTo(pathname) {
  if (typeof window === "undefined") return;
  if (window.location.pathname === pathname) return;
  // 防 100ms 内重复导航 (快速双击/多链接连击→幽灵历史条目)
  const now = Date.now();
  if (pathname === _lastNavPath && now - _lastNavTime < 100) return;
  _lastNavPath = pathname;
  _lastNavTime = now;
  window.history.pushState({}, "", pathname + (window.location.hash || ""));
  window.dispatchEvent(new PopStateEvent("popstate"));
}
