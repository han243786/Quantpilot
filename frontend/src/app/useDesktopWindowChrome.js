import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function resolveTauriWindow() {
  const isTauriRuntime =
    typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
  if (!isTauriRuntime) return null;
  try {
    return getCurrentWindow();
  } catch (error) {
    if (import.meta.env.DEV) {
      console.warn("[App] Tauri 窗口 API 初始化失败:", error.message);
    }
    return null;
  }
}

export function useDesktopWindowChrome() {
  const appWindow = useMemo(() => resolveTauriWindow(), []);
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    if (!appWindow) return undefined;
    let disposed = false;
    appWindow.isMaximized().then((value) => {
      if (!disposed) setIsMaximized(value);
    });
    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then((value) => {
        if (!disposed) setIsMaximized(value);
      });
    });
    return () => {
      disposed = true;
      unlisten.then((fn) => fn());
    };
  }, [appWindow]);

  return { appWindow, isMaximized };
}
