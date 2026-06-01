import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { resolveTauriWindow, useDesktopWindowChrome } from "./useDesktopWindowChrome";

const getCurrentWindowMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: getCurrentWindowMock
}));

function DesktopChromeProbe() {
  const { appWindow, isMaximized } = useDesktopWindowChrome();
  return (
    <div>
      <span data-testid="window-state">{appWindow ? "desktop" : "browser"}</span>
      <span data-testid="maximized-state">{isMaximized ? "maximized" : "normal"}</span>
    </div>
  );
}

describe("useDesktopWindowChrome", () => {
  afterEach(() => {
    delete window.__TAURI_INTERNALS__;
    getCurrentWindowMock.mockReset();
    vi.restoreAllMocks();
  });

  it("returns no window outside Tauri runtime", () => {
    expect(resolveTauriWindow()).toBeNull();
  });

  it("tracks desktop maximized state", async () => {
    window.__TAURI_INTERNALS__ = {};
    const unlisten = vi.fn();
    const appWindow = {
      isMaximized: vi.fn().mockResolvedValue(true),
      onResized: vi.fn().mockResolvedValue(unlisten),
    };
    getCurrentWindowMock.mockReturnValue(appWindow);

    render(<DesktopChromeProbe />);

    expect(screen.getByTestId("window-state")).toHaveTextContent("desktop");
    await waitFor(() => {
      expect(screen.getByTestId("maximized-state")).toHaveTextContent("maximized");
    });
    expect(appWindow.onResized).toHaveBeenCalledTimes(1);
  });
});
