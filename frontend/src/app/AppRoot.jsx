import React from "react";
import App from "../App";
import { I18nProvider } from "../i18n";

export default function AppRoot() {
  return (
    <React.StrictMode>
      <I18nProvider>
        <App />
      </I18nProvider>
    </React.StrictMode>
  );
}
