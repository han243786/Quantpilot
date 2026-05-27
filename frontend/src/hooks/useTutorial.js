import { useState, useCallback, useEffect } from "react";

let listeners = [];
const FIRST_VISIT_KEY = "qp.tutorial.seen";
const LEGACY_FIRST_VISIT_KEY = "quantpilot.tutorial.seen";

function markTutorialSeen() {
  try {
    window.localStorage?.setItem(FIRST_VISIT_KEY, "1");
    window.localStorage?.setItem(LEGACY_FIRST_VISIT_KEY, "1");
  } catch (_) {
    // localStorage may be unavailable in locked-down webviews.
  }
}

function hasSeenTutorial() {
  try {
    return (
      window.localStorage?.getItem(FIRST_VISIT_KEY) === "1" ||
      window.localStorage?.getItem(LEGACY_FIRST_VISIT_KEY) === "1"
    );
  } catch (_) {
    return true;
  }
}

export function triggerTutorial() {
  listeners.forEach((fn) => fn(true));
}

export function closeTutorial() {
  markTutorialSeen();
  listeners.forEach((fn) => fn(false));
}

export function useTutorial() {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    listeners.push(setOpen);
    return () => { listeners = listeners.filter((fn) => fn !== setOpen); };
  }, []);

  useEffect(() => {
    const handleOpenTutorial = () => setOpen(true);
    window.addEventListener("qp-open-tutorial", handleOpenTutorial);
    return () => window.removeEventListener("qp-open-tutorial", handleOpenTutorial);
  }, []);

  useEffect(() => {
    if (typeof window === "undefined" || hasSeenTutorial()) return undefined;
    const timer = window.setTimeout(() => setOpen(true), 650);
    return () => window.clearTimeout(timer);
  }, []);

  const start = useCallback(() => setOpen(true), []);
  const close = useCallback(() => {
    markTutorialSeen();
    setOpen(false);
  }, []);
  return { tutorialOpen: open, startTutorial: start, closeTutorial: close };
}
