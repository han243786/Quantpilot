import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import LeftSidebar from "./LeftSidebar";

vi.mock("../router", () => ({ navigateTo: vi.fn() }));

describe("LeftSidebar", () => {
  it("renders brand icon and text", () => {
    render(<LeftSidebar />);
    expect(screen.getByText("QP")).toBeTruthy();
    expect(screen.getByText("QuantPilot")).toBeTruthy();
  });

  it("renders all 7 navigation items", () => {
    render(<LeftSidebar />);
    const items = ["策略", "QuantScript", "审批", "告警", "快照", "故障手册", "混沌"];
    for (const label of items) {
      expect(screen.getByTitle(label)).toBeTruthy();
    }
  });

  it("renders nav items as anchor tags with href", () => {
    render(<LeftSidebar />);
    const links = screen.getAllByRole("link");
    expect(links.length).toBeGreaterThanOrEqual(7);
    expect(links[0].getAttribute("href")).toBe("/strategies");
  });

  it("has accessible navigation role", () => {
    render(<LeftSidebar />);
    expect(screen.getByRole("navigation")).toBeTruthy();
    expect(screen.getByTestId("app-sidebar").getAttribute("role")).toBeNull(); // <nav> is implicit
  });

  it("marks current route as active with aria-current", () => {
    Object.defineProperty(window, "location", {
      value: { pathname: "/strategies" },
      writable: true,
    });
    render(<LeftSidebar />);
    const strategies = screen.getByTitle("策略");
    expect(strategies.getAttribute("aria-current")).toBe("page");
  });
});
