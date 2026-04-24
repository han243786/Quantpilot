import { describe, expect, it } from "vitest";
import {
  collectVisibleNodeIds,
  isNodeVisibleInViewport
} from "./strategyCanvasViewport";

describe("strategyCanvasViewport", () => {
  const viewport = { x: 0, y: 0, zoom: 1 };
  const viewportSize = { width: 1280, height: 900 };

  it("keeps nodes visible when they intersect the viewport", () => {
    expect(
      isNodeVisibleInViewport(
        { position: { x: 100, y: 120 } },
        viewport,
        viewportSize
      )
    ).toBe(true);
  });

  it("filters nodes well outside the viewport", () => {
    const visible = collectVisibleNodeIds(
      [
        { id: "visible", position: { x: 120, y: 120 } },
        { id: "far-right", position: { x: 2200, y: 120 } },
        { id: "far-top", position: { x: 120, y: -600 } }
      ],
      viewport,
      viewportSize
    );

    expect(Array.from(visible)).toEqual(["visible"]);
  });
});
