import { useEffect, useState } from "react";
import { listRegisteredLocales, useI18n } from "../i18n";

const THEME_KEY = "quantpilot.theme";
const QS_AUTOSAVE_KEY = "quantpilot.quantscript.autosave";

function readSetting(key, fallback) {
  try {
    return window.localStorage?.getItem(key) || fallback;
  } catch (_) {
    return fallback;
  }
}

function writeSetting(key, value, errorMessage) {
  try {
    window.localStorage?.setItem(key, value);
  } catch (_) {
    window.dispatchEvent(
      new CustomEvent("qp-toast", {
        detail: { type: "error", message: errorMessage },
      }),
    );
  }
}

function applyTheme(theme, errorMessage) {
  if (typeof document === "undefined") return;
  if (theme === "light" || theme === "dark") {
    document.documentElement.dataset.theme = theme;
  } else {
    document.documentElement.removeAttribute("data-theme");
  }
  writeSetting(THEME_KEY, theme, errorMessage);
  window.dispatchEvent(new CustomEvent("qp-theme-change", { detail: { theme } }));
}

export default function SettingsPage() {
  const { t, locale, setLocale } = useI18n();
  const [theme, setTheme] = useState(() => readSetting(THEME_KEY, "auto"));
  const [autoSave, setAutoSave] = useState(() => readSetting(QS_AUTOSAVE_KEY, "1") !== "0");
  const storageErrorMessage = t("设置保存失败，本地存储不可用。");

  useEffect(() => {
    applyTheme(theme, storageErrorMessage);
  }, [storageErrorMessage, theme]);

  useEffect(() => {
    writeSetting(QS_AUTOSAVE_KEY, autoSave ? "1" : "0", storageErrorMessage);
  }, [autoSave, storageErrorMessage]);

  return (
    <section className="settings-page" aria-labelledby="settings-title">
      <header className="settings-page__header">
        <h1 id="settings-title">{t("设置")}</h1>
        <p>{t("管理默认语言、主题偏好和编辑器行为。")}</p>
      </header>

      <div className="settings-page__grid">
        <section className="settings-panel" aria-labelledby="settings-language">
          <h2 id="settings-language">{t("语言")}</h2>
          <select value={locale} onChange={(e) => setLocale(e.target.value)}>
            {listRegisteredLocales().map((item) => (
              <option key={item} value={item}>
                {item === "zh-CN" ? "中文" : "English"}
              </option>
            ))}
          </select>
        </section>

        <section className="settings-panel" aria-labelledby="settings-theme">
          <h2 id="settings-theme">{t("主题")}</h2>
          <div className="segmented-control" role="group" aria-label={t("主题")}>
            {[
              ["auto", t("跟随系统")],
              ["dark", t("暗色")],
              ["light", t("亮色")],
            ].map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={theme === value ? "segmented-control__item segmented-control__item--active" : "segmented-control__item"}
                onClick={() => setTheme(value)}
              >
                {label}
              </button>
            ))}
          </div>
        </section>

        <section className="settings-panel" aria-labelledby="settings-editor">
          <h2 id="settings-editor">{t("编辑器")}</h2>
          <label className="settings-toggle">
            <input
              type="checkbox"
              checked={autoSave}
              onChange={(e) => setAutoSave(e.target.checked)}
            />
            <span>{t("自动保存 QuantScript 草稿")}</span>
          </label>
        </section>

      </div>
    </section>
  );
}
