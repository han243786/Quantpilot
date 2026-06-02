import { afterEach, describe, expect, it, vi } from "vitest";
import {
  focusCanvasTargets,
  resolveNodeCardMode,
  scheduleAfterFirstPaint
} from "./strategyCanvasInteractionShell";

describe("strategyCanvasInteractionShell", () => {
  const originalLocation = window.location.href;
  const originalRequestAnimationFrame = window.requestAnimationFrame;
  const originalCancelAnimationFrame = window.cancelAnimationFrame;
  const originalRequestIdleCallback = window.requestIdleCallback;
  const originalCancelIdleCallback = window.cancelIdleCallback;

  afterEach(() => {
    window.history.pushState({}, "", originalLocation);
    window.requestAnimationFrame = originalRequestAnimationFrame;
    window.cancelAnimationFrame = originalCancelAnimationFrame;
    window.requestIdleCallback = originalRequestIdleCallback;
    window.cancelIdleCallback = originalCancelIdleCallback;
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("resolves staged node cards by default and full cards from query state", () => {
    window.history.pushState({}, "", "/workspace/alpha");
    expect(resolveNodeCardMode()).toBe("staged");

    window.history.pushState({}, "", "/workspace/alpha?node_card_mode=full");
    expect(resolveNodeCardMode()).toBe("full");
  });

  it("focuses the anchor node before falling back to bounds", () => {
    const reactFlow = {
      fitBounds: vi.fn(),
      setCenter: vi.fn()
    };
    const nodes = [
      { id: "node_a", position: { x: 100, y: 200 } },
      { id: "node_b", position: { x: 600, y: 400 } }
    ];

    focusCanvasTargets(reactFlow, nodes, ["node_a", "node_b"], "node_b");

    expect(reactFlow.setCenter).toHaveBeenCalledWith(720, 460, {
      zoom: 0.92,
      duration: 260
    });
    expect(reactFlow.fitBounds).not.toHaveBeenCalled();
  });

  it("fits multi-node targets when there is no anchor", () => {
    const reactFlow = {
      fitBounds: vi.fn(),
      setCenter: vi.fn()
    };
    const nodes = [
      { id: "node_a", position: { x: 100, y: 200 } },
      { id: "node_b", position: { x: 600, y: 400 } }
    ];

    focusCanvasTargets(reactFlow, nodes, ["node_a", "node_b"]);

    expect(reactFlow.setCenter).not.toHaveBeenCalled();
    expect(reactFlow.fitBounds).toHaveBeenCalledWith(
      {
        x: -20,
        y: 116,
        width: 990,
        height: 508
      },
      {
        duration: 280,
        padding: 0.18
      }
    );
  });

  it("schedules callbacks after animation frame and timeout fallback", () => {
    vi.useFakeTimers();
    const callback = vi.fn();
    let frameCallback = null;
    window.requestAnimationFrame = vi.fn((next) => {
      frameCallback = next;
      return 7;
    });
    window.cancelAnimationFrame = vi.fn();
    window.requestIdleCallback = undefined;
    window.cancelIdleCallback = undefined;

    const dispose = scheduleAfterFirstPaint(callback);
    expect(callback).not.toHaveBeenCalled();

    frameCallback();
    vi.runOnlyPendingTimers();

    expect(callback).toHaveBeenCalledTimes(1);

    dispose();
    expect(window.cancelAnimationFrame).toHaveBeenCalledWith(7);
  });
});
