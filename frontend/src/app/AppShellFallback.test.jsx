import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import AppShellFallback from "./AppShellFallback";

const storeState = vi.hoisted(() => ({ capabilityStatus: "degraded" }));

vi.mock("../i18n", () => ({
  useI18n: () => ({ t: (text) => text })
}));

vi.mock("../store/graphStore", () => ({
  useGraphStore: (selector) => selector(storeState)
}));

describe("AppShellFallback", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    storeState.capabilityStatus = "degraded";
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows the capability stage and delayed skip action", () => {
    const onSkip = vi.fn();

    render(<AppShellFallback onSkip={onSkip} />);

    expect(screen.getByRole("status")).toHaveTextContent("已加载本地缓存");
    expect(screen.queryByRole("button")).not.toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(5000);
    });

    screen.getByRole("button", { name: "跳过等待，使用本地缓存" }).click();

    expect(onSkip).toHaveBeenCalledTimes(1);
  });
});
