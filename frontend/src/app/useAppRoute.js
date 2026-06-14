import { useEffect, useState } from "react";
import { parseRoute, strategiesPath } from "../router";

export function getInitialAppRoute() {
  return parseRoute(
    typeof window === "undefined" ? "/" : window.location.pathname,
    typeof window === "undefined" ? "" : window.location.search
  );
}

export function useAppRoute() {
  const [route, setRoute] = useState(getInitialAppRoute);

  useEffect(() => {
    const handlePopState = () => {
      setRoute(parseRoute(window.location.pathname, window.location.search));
    };

    window.addEventListener("popstate", handlePopState);
    return () => window.removeEventListener("popstate", handlePopState);
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    if (window.location.pathname !== "/" || route.name !== "strategies") return;
    window.history.replaceState({}, "", strategiesPath());
    setRoute(parseRoute(window.location.pathname, window.location.search));
  }, [route.name]);

  return route;
}
