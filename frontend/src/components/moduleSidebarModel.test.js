import { describe, expect, it } from "vitest";
import {
  buildCategoryLabels,
  buildPrioritizedCategories,
  laneRecommendation,
  moduleAvailabilityLabel,
  moduleAvailabilityTone
} from "./moduleSidebarModel";

describe("moduleSidebarModel", () => {
  it("prioritizes categories by workspace lane and selected node type", () => {
    expect(buildPrioritizedCategories("diagnostics", "custom")).toEqual([
      "execution",
      "custom",
      "risk",
      "runtime",
      "agent",
      "data",
      "intent"
    ]);

    expect(buildPrioritizedCategories(null, "risk")).toEqual([
      "risk",
      "data",
      "intent",
      "agent",
      "execution",
      "runtime"
    ]);
  });

  it("keeps lane recommendations contextual to the current selection", () => {
    expect(laneRecommendation("code", "Code", "runtime")).toContain("runtime");
    expect(laneRecommendation(null, "Canvas", "agent")).toContain("agent");
  });

  it("maps module availability and category labels through the caller translator", () => {
    const t = (text) => `translated:${text}`;

    expect(moduleAvailabilityTone("unsupported")).toBe("warning");
    expect(moduleAvailabilityTone("supported")).toBe("success");
    expect(moduleAvailabilityLabel("unsupported", t)).toContain("translated:");
    expect(moduleAvailabilityLabel("supported", t)).toContain("translated:");
    expect(buildCategoryLabels(t).data).toContain("translated:");
  });
});
