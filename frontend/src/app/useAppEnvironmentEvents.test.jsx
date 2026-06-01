import { act, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAppEnvironmentEvents } from "./useAppEnvironmentEvents";

const refreshGraphIndex = vi.hoisted(() => vi.fn());

vi.mock("../store/graphStore", () => ({
  useGraphStore: Object.assign(() => null, {
    getState: () => ({ refreshGraphIndex })
  })
}));

function setOnline(value) {
  Object.defineProperty(navigator, "onLine", {
    configurable: true,
    value,
  });
}

function setDocumentHidden(value) {
  Object.defineProperty(document, "hidden", {
    configurable: true,
    value,
  });
}

function EnvironmentProbe({ route, onToggleCommandPalette = () => {} }) {
  const {
    isOffline,
    storageQuotaExceeded,
    setStorageQuotaExceeded,
  } = useAppEnvironmentEvents({ route, onToggleCommandPalette });

  return (
    <div>
      <div data-testid="network-state">{isOffline ? "offline" : "online"}</div>
      <div data-testid="quota-state">{storageQuotaExceeded ? "quota" : "ok"}</div>
      <button onClick={() => setStorageQuotaExceeded(false)}>clear quota</button>
    </div>
  );
}

describe("useAppEnvironmentEvents", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    refreshGraphIndex.mockReset();
    document.documentElement.removeAttribute("data-theme");
    window.localStorage.clear();
    setOnline(true);
    setDocumentHidden(false);
  });

  it("tracks browser environment events", () => {
    const onToggleCommandPalette = vi.fn();
    setOnline(true);
    setDocumentHidden(false);
    window.localStorage.setItem("quantpilot.theme", "dark");

    render(
      <EnvironmentProbe
        route={{ name: "strategies" }}
        onToggleCommandPalette={onToggleCommandPalette}
      />
    );

    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(screen.getByTestId("network-state")).toHaveTextContent("online");

    act(() => {
      window.dispatchEvent(new Event("offline"));
    });
    expect(screen.getByTestId("network-state")).toHaveTextContent("offline");

    act(() => {
      window.dispatchEvent(new Event("online"));
      window.dispatchEvent(new Event("qp-storage-quota-exceeded"));
      document.dispatchEvent(new Event("visibilitychange"));
      window.dispatchEvent(new KeyboardEvent("keydown", { ctrlKey: true, key: "k" }));
    });

    expect(screen.getByTestId("network-state")).toHaveTextContent("online");
    expect(screen.getByTestId("quota-state")).toHaveTextContent("quota");
    expect(refreshGraphIndex).toHaveBeenCalledTimes(1);
    expect(onToggleCommandPalette).toHaveBeenCalledTimes(1);

    act(() => {
      screen.getByRole("button", { name: "clear quota" }).click();
    });
    expect(screen.getByTestId("quota-state")).toHaveTextContent("ok");
  });

  it("guards unload for editing routes", () => {
    render(<EnvironmentProbe route={{ name: "strategy-workspace" }} />);

    const event = new Event("beforeunload", { cancelable: true });
    act(() => {
      window.dispatchEvent(event);
    });

    expect(event.defaultPrevented).toBe(true);
  });
});
