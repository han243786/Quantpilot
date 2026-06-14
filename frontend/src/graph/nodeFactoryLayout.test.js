import { describe, expect, it } from "vitest";
import {
  createNodePositionAllocator,
  initialNodeLaneY,
  nodeLaneX
} from "./nodeFactoryLayout";

describe("nodeFactoryLayout", () => {
  it("allocates deterministic positions per graph node lane", () => {
    const nextPosition = createNodePositionAllocator();

    expect(nextPosition("runtime")).toEqual({ x: nodeLaneX.runtime, y: initialNodeLaneY.runtime });
    expect(nextPosition("data")).toEqual({ x: nodeLaneX.data, y: initialNodeLaneY.data });
    expect(nextPosition("data")).toEqual({ x: nodeLaneX.data, y: initialNodeLaneY.data + 180 });
  });

  it("falls back to the data lane when category is unknown", () => {
    const nextPosition = createNodePositionAllocator();

    expect(nextPosition("custom")).toEqual({ x: 120, y: 120 });
  });
});
