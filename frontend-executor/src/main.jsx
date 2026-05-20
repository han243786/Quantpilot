import React from "react";
import ReactDOM from "react-dom/client";
import ExecutorApp from "./ExecutorApp";
import "./design-system.css";

window.addEventListener("unhandledrejection", (e) => {
  console.error("[Executor] 未捕获的 Promise 错误:", e.reason);
});

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <ExecutorApp />
  </React.StrictMode>
);
