import { describe, expect, it } from "vitest";
import {
  CAPABILITY_ACTION_MAP,
  SUPPORT_MATRIX,
  WORKSPACE_SURFACE_MAP
} from "./capabilityCatalog";

describe("capabilityCatalog", () => {
  it("keeps workspace surfaces and UI actions exposed through the support matrix", () => {
    expect(Object.keys(SUPPORT_MATRIX.workspace.surfaces)).toEqual(
      Object.keys(WORKSPACE_SURFACE_MAP)
    );
    expect(Object.keys(SUPPORT_MATRIX.uiActionMap)).toEqual(
      Object.keys(CAPABILITY_ACTION_MAP)
    );
  });
});
