import { describe, expect, it } from "vitest";
import {
  COMMAND_NAVIGATION_DEFS,
  SHELL_NAV_SECTIONS,
  isShellNavPathActive,
} from "./shellNavigation";

describe("shellNavigation", () => {
  it("defines sidebar sections with stable route paths", () => {
    const paths = SHELL_NAV_SECTIONS.flat().map((item) => item.path);

    expect(paths).toEqual([
      "/strategies",
      "/quantscript",
      "/approvals",
      "/alerts",
      "/snapshots",
      "/runbook",
      "/chaos",
      "/settings",
    ]);
  });

  it("keeps command palette navigation entries route-backed", () => {
    expect(COMMAND_NAVIGATION_DEFS.map((item) => [item.id, item.keys[0]])).toEqual([
      ["strategies", "/strategies"],
      ["quantscript", "/quantscript"],
      ["approvals", "/approvals"],
      ["alerts", "/alerts"],
      ["snapshots", "/snapshots"],
      ["runbook", "/runbook"],
      ["chaos", "/chaos"],
    ]);
  });

  it("matches exact, nested, and query active paths", () => {
    expect(isShellNavPathActive("/strategies", "/strategies")).toBe(true);
    expect(isShellNavPathActive("/strategies/alpha", "/strategies")).toBe(true);
    expect(isShellNavPathActive("/strategies?view=list", "/strategies")).toBe(true);
    expect(isShellNavPathActive("/strategy", "/strategies")).toBe(false);
  });
});
