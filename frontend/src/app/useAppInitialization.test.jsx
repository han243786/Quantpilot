import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAppInitialization } from "./useAppInitialization";

const initializeMock = vi.hoisted(() => vi.fn());

vi.mock("../store/graphStore", () => ({
  useGraphStore: (selector) => selector({ initialize: initializeMock })
}));

function InitializationProbe() {
  const isInitialized = useAppInitialization();
  return <div>{isInitialized ? "ready" : "loading"}</div>;
}

describe("useAppInitialization", () => {
  it("marks the app ready after initialize settles", async () => {
    initializeMock.mockResolvedValueOnce(undefined);

    render(<InitializationProbe />);

    expect(screen.getByText("loading")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("ready")).toBeInTheDocument());
    expect(initializeMock).toHaveBeenCalledTimes(1);
  });
});
