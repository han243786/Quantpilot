import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./i18n";
import { installTestBridge } from "./test/testBridge";
import "./styles.css";
import "./shared.css";
import "@xyflow/react/dist/style.css";

installTestBridge();

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <I18nProvider>
      <App />
    </I18nProvider>
  </React.StrictMode>
);
