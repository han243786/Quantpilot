import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./i18n";
import { installTestBridge } from "./test/testBridge";
import "./design-system.css";
import "./styles.css";
import "./styles-responsive-panels.css";
import "./shared.css";
import "@xyflow/react/dist/style.css";

installTestBridge();

// v2.5.0: 全局 unhandledrejection 处理, 防止静默 Promise 失败
window.addEventListener("unhandledrejection", (event) => {
  console.error("[UnhandledRejection]", event.reason);
});

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <I18nProvider>
      <App />
    </I18nProvider>
  </React.StrictMode>
);
