import { createContext, createElement, useContext, useEffect, useMemo, useState } from "react";
import enUS from "./locales/en-US";

const LOCALE_STORAGE_KEY = "quantpilot.locale";
export const DEFAULT_LOCALE = "zh-CN";

const localeRegistry = new Map([
  [DEFAULT_LOCALE, {}],
  ["en-US", enUS]
]);

let globalLocale = DEFAULT_LOCALE;

function normalizeLocale(locale) {
  if (!locale) return DEFAULT_LOCALE;
  if (localeRegistry.has(locale)) return locale;

  const normalized = [...localeRegistry.keys()].find(
    (supportedLocale) =>
      locale.toLowerCase() === supportedLocale.toLowerCase() ||
      locale.toLowerCase().startsWith(`${supportedLocale.toLowerCase().split("-")[0]}-`)
  );

  return normalized || DEFAULT_LOCALE;
}

function interpolate(message, variables = {}) {
  return String(message).replace(/\{(\w+)\}/g, (_, key) =>
    variables[key] === undefined || variables[key] === null ? "" : String(variables[key])
  );
}

function resolveMessages(locale) {
  return localeRegistry.get(normalizeLocale(locale)) || {};
}

export function defineLocale(messages = {}) {
  return messages;
}

export function registerLocale(locale, messages) {
  localeRegistry.set(locale, messages || {});
}

export function setGlobalLocale(locale) {
  globalLocale = normalizeLocale(locale);
}

export function getGlobalLocale() {
  return globalLocale;
}

export function translateText(baseText, variables = {}, locale = globalLocale) {
  const messages = resolveMessages(locale);
  const translated = messages[baseText] || baseText;
  return interpolate(translated, variables);
}

function resolveInitialLocale(defaultLocale) {
  if (typeof window === "undefined") return normalizeLocale(defaultLocale);

  const searchLocale = new URLSearchParams(window.location.search).get("lang");
  if (searchLocale) return normalizeLocale(searchLocale);

  const injectedLocale = window.__QUANTPILOT_LOCALE__;
  if (typeof injectedLocale === "string" && injectedLocale.trim()) {
    return normalizeLocale(injectedLocale.trim());
  }

  return normalizeLocale(defaultLocale);
}

const I18nContext = createContext({
  locale: DEFAULT_LOCALE,
  setLocale: () => {},
  t: (baseText, variables) => translateText(baseText, variables, DEFAULT_LOCALE)
});

export function I18nProvider({ children, defaultLocale = DEFAULT_LOCALE }) {
  const [locale, setLocaleState] = useState(() => resolveInitialLocale(defaultLocale));

  useEffect(() => {
    setGlobalLocale(locale);
    if (typeof document !== "undefined") {
      document.documentElement.lang = locale;
    }
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.setItem(LOCALE_STORAGE_KEY, locale);
    }
  }, [locale]);

  const value = useMemo(
    () => ({
      locale,
      setLocale(nextLocale) {
        setLocaleState(normalizeLocale(nextLocale));
      },
      t(baseText, variables) {
        return translateText(baseText, variables, locale);
      }
    }),
    [locale]
  );

  return createElement(I18nContext.Provider, { value }, children);
}

export function useI18n() {
  return useContext(I18nContext);
}

export function listRegisteredLocales() {
  return [...localeRegistry.keys()];
}
