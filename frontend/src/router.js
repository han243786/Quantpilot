export {
  alertsPath,
  approvalsPath,
  backtestComparePath,
  backtestDetailPath,
  chaosPath,
  parseRoute,
  quantscriptPath,
  runbookPath,
  settingsPath,
  snapshotsPath,
  strategiesPath,
  strategyBacktestsPath,
  strategyWorkspacePath,
} from "./routing/routeContract";

let _lastNavPath = "";
let _lastNavTime = 0;

export function navigateTo(pathname) {
  if (typeof window === "undefined") return;
  if (window.location.pathname === pathname) return;
  const now = Date.now();
  if (pathname === _lastNavPath && now - _lastNavTime < 100) {
    if (import.meta.env.DEV) {
      console.debug("[router] 100ms 内重复导航已忽略:", pathname);
    }
    return;
  }
  _lastNavPath = pathname;
  _lastNavTime = now;
  window.history.pushState({}, "", pathname + (window.location.hash || ""));
  window.dispatchEvent(new PopStateEvent("popstate"));
}
