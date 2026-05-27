import React from "react";
import ReactDOM from "react-dom/client";
import ExecutorApp from "./ExecutorApp";
import { I18nProvider } from "./i18n";
import "./design-system.css";

const THEME_STORAGE_KEY = "quantpilot.theme";

function applyStoredTheme() {
  try {
    const theme = window.localStorage?.getItem(THEME_STORAGE_KEY) || "auto";
    if (theme === "light" || theme === "dark") {
      document.documentElement.dataset.theme = theme;
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
  } catch (_) {
    document.documentElement.removeAttribute("data-theme");
  }
}

applyStoredTheme();

window.addEventListener("storage", (event) => {
  if (event.key === THEME_STORAGE_KEY) {
    applyStoredTheme();
  }
});

window.addEventListener("unhandledrejection", (e) => {
  console.error("[Executor] 未捕获的 Promise 错误:", e.reason);
});

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <I18nProvider>
      <ExecutorApp />
    </I18nProvider>
  </React.StrictMode>
);
