import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  __resetNavigationDispatchForTest,
  navigateTo,
} from "./navigationDispatch";

describe("navigationDispatch", () => {
  beforeEach(() => {
    __resetNavigationDispatchForTest();
    window.history.replaceState({}, "", "/initial");
  });

  afterEach(() => {
    vi.restoreAllMocks();
    __resetNavigationDispatchForTest();
    window.history.replaceState({}, "", "/");
  });

  it("pushes a new history entry and emits popstate", () => {
    const onPopState = vi.fn();
    window.addEventListener("popstate", onPopState);
    window.history.replaceState({}, "", "/initial#focus");

    navigateTo("/strategies");

    expect(window.location.pathname).toBe("/strategies");
    expect(window.location.hash).toBe("#focus");
    expect(onPopState).toHaveBeenCalledTimes(1);

    window.removeEventListener("popstate", onPopState);
  });

  it("ignores navigation to the current pathname", () => {
    const onPopState = vi.fn();
    window.addEventListener("popstate", onPopState);
    window.history.replaceState({}, "", "/strategies");

    navigateTo("/strategies");

    expect(onPopState).not.toHaveBeenCalled();
    expect(window.location.pathname).toBe("/strategies");

    window.removeEventListener("popstate", onPopState);
  });

  it("deduplicates repeated target paths within 100ms", () => {
    const onPopState = vi.fn();
    const pushState = vi.spyOn(window.history, "pushState");
    const debug = vi.spyOn(console, "debug").mockImplementation(() => {});
    window.addEventListener("popstate", onPopState);
    const now = vi.spyOn(Date, "now").mockReturnValue(1_000);

    navigateTo("/strategies?view=one");
    now.mockReturnValue(1_050);
    navigateTo("/strategies?view=one");

    expect(pushState).toHaveBeenCalledTimes(1);
    expect(onPopState).toHaveBeenCalledTimes(1);
    expect(debug).toHaveBeenCalledTimes(1);

    window.removeEventListener("popstate", onPopState);
  });
});
