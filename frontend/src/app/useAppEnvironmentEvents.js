import { useEffect, useRef, useState } from "react";
import { useGraphStore } from "../store/graphStore";

export function useAppEnvironmentEvents({ route, onToggleCommandPalette }) {
  const [isOffline, setIsOffline] = useState(
    typeof navigator !== "undefined" ? !navigator.onLine : false
  );
  const [storageQuotaExceeded, setStorageQuotaExceeded] = useState(false);

  useEffect(() => {
    const applyStoredTheme = () => {
      try {
        const theme = window.localStorage?.getItem("quantpilot.theme") || "auto";
        if (theme === "light" || theme === "dark") {
          document.documentElement.dataset.theme = theme;
        } else {
          document.documentElement.removeAttribute("data-theme");
        }
      } catch (_) {
        document.documentElement.removeAttribute("data-theme");
      }
    };
    applyStoredTheme();
    window.addEventListener("qp-theme-change", applyStoredTheme);
    return () => window.removeEventListener("qp-theme-change", applyStoredTheme);
  }, []);

  useEffect(() => {
    const goOffline = () => setIsOffline(true);
    const goOnline = () => setIsOffline(false);
    window.addEventListener("offline", goOffline);
    window.addEventListener("online", goOnline);
    return () => {
      window.removeEventListener("offline", goOffline);
      window.removeEventListener("online", goOnline);
    };
  }, []);

  useEffect(() => {
    const handler = () => setStorageQuotaExceeded(true);
    window.addEventListener("qp-storage-quota-exceeded", handler);
    return () => window.removeEventListener("qp-storage-quota-exceeded", handler);
  }, []);

  useEffect(() => {
    const handle = () => {
      if (!document.hidden) {
        useGraphStore.getState().refreshGraphIndex?.();
      }
    };
    document.addEventListener("visibilitychange", handle);
    return () => document.removeEventListener("visibilitychange", handle);
  }, []);

  const routeRef = useRef(route);
  routeRef.current = route;
  useEffect(() => {
    const handler = (event) => {
      const name = routeRef.current.name;
      if (name === "strategy-workspace" || name === "quantscript") {
        event.preventDefault();
        const locale = localStorage.getItem("quantpilot.locale") || "zh-CN";
        event.returnValue = locale === "en-US"
          ? "You have unsaved strategy changes. Leaving this page will discard your changes."
          : "当前有未保存的策略图更改，离开此页面将丢失更改。";
      }
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, []);

  useEffect(() => {
    const handleKey = (event) => {
      if ((event.metaKey || event.ctrlKey) && event.key === "k") {
        event.preventDefault();
        onToggleCommandPalette();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onToggleCommandPalette]);

  return {
    isOffline,
    storageQuotaExceeded,
    setStorageQuotaExceeded,
  };
}
