import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import AppRoot from "./AppRoot";

vi.mock("../App", () => ({
  default: () => <div data-testid="app-root-child" />
}));

describe("AppRoot", () => {
  it("renders the application child tree", () => {
    render(<AppRoot />);

    expect(screen.getByTestId("app-root-child")).toBeInTheDocument();
  });
});
