import { useState, useCallback, useEffect } from "react";

let listeners = [];

export function triggerTutorial() {
  listeners.forEach((fn) => fn(true));
}

export function closeTutorial() {
  listeners.forEach((fn) => fn(false));
}

export function useTutorial() {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    listeners.push(setOpen);
    return () => { listeners = listeners.filter((fn) => fn !== setOpen); };
  }, []);
  const start = useCallback(() => setOpen(true), []);
  const close = useCallback(() => setOpen(false), []);
  return { tutorialOpen: open, startTutorial: start, closeTutorial: close };
}
