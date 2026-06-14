import { startTransition } from "react";
import { buildCanvasFocusBounds } from "./strategyCanvasFocus";

export function resolveNodeCardMode() {
  if (typeof window === "undefined") return "staged";
  const params = new URLSearchParams(window.location.search);
  return params.get("node_card_mode") === "full" ? "full" : "staged";
}

export function scheduleAfterFirstPaint(callback) {
  if (typeof window === "undefined") {
    callback();
    return () => {};
  }

  let frameId = null;
  let idleId = null;
  let timeoutId = null;
  let disposed = false;

  const run = () => {
    if (disposed) return;
    startTransition(() => {
      callback();
    });
  };

  const queueIdle = () => {
    if (typeof window.requestIdleCallback === "function") {
      idleId = window.requestIdleCallback(run, { timeout: 600 });
      return;
    }
    timeoutId = window.setTimeout(run, 0);
  };

  frameId = window.requestAnimationFrame(queueIdle);

  return () => {
    disposed = true;
    if (frameId !== null) window.cancelAnimationFrame(frameId);
    if (idleId !== null && typeof window.cancelIdleCallback === "function") {
      window.cancelIdleCallback(idleId);
    }
    if (timeoutId !== null) window.clearTimeout(timeoutId);
  };
}

export function focusCanvasTargets(reactFlow, nodes, targetIds, anchorId = null) {
  if (!Array.isArray(targetIds) || targetIds.length === 0) return;

  if (anchorId) {
    const node = nodes.find((item) => item.id === anchorId);
    if (!node) return;

    reactFlow.setCenter(node.position.x + 120, node.position.y + 60, {
      zoom: 0.92,
      duration: 260
    });
    return;
  }

  const bounds = buildCanvasFocusBounds(nodes, targetIds);
  if (!bounds) return;

  if (targetIds.length === 1) {
    const node = nodes.find((item) => item.id === targetIds[0]);
    if (!node) return;

    reactFlow.setCenter(node.position.x + 120, node.position.y + 60, {
      zoom: 0.92,
      duration: 260
    });
    return;
  }

  reactFlow.fitBounds(bounds, {
    duration: 280,
    padding: 0.18
  });
}
