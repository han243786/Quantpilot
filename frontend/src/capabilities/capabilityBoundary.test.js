import { describe, expect, it } from "vitest";
import {
  buildCapabilityContext,
  getCapabilityBoundaryIssues
} from "./capabilityBoundary";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

describe("capabilityBoundary", () => {
  it("builds runtime capability context only for trusted capability snapshots", () => {
    expect(buildCapabilityContext(backendCapabilitiesFixture)).toEqual({
      schema_hash: backendCapabilitiesFixture.schema_hash,
      permission_boundary: backendCapabilitiesFixture.permission_boundary,
    });

    expect(
      getCapabilityBoundaryIssues({
        ...backendCapabilitiesFixture,
        schema_hash: "unsafe",
      })
    ).toContain("能力 hash 缺失或格式非法。");
  });
});
