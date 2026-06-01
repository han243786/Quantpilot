let lastNavPath = "";
let lastNavTime = 0;

export function __resetNavigationDispatchForTest() {
  lastNavPath = "";
  lastNavTime = 0;
}

export function navigateTo(pathname) {
  if (typeof window === "undefined") return;
  if (window.location.pathname === pathname) return;

  const now = Date.now();
  if (pathname === lastNavPath && now - lastNavTime < 100) {
    if (import.meta.env.DEV) {
      console.debug("[router] 100ms 内重复导航已忽略:", pathname);
    }
    return;
  }

  lastNavPath = pathname;
  lastNavTime = now;
  window.history.pushState({}, "", pathname + (window.location.hash || ""));
  window.dispatchEvent(new PopStateEvent("popstate"));
}
