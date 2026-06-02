import { describe, expect, it } from "vitest";
import { lineRangeToSelection, sectionsToSelection } from "./propertyPanelAuthoringCards";

describe("propertyPanelAuthoringCards selection helpers", () => {
  it("maps one-based source line ranges into character selection ranges", () => {
    const source = ["first line", "second", "third"].join("\n");

    expect(lineRangeToSelection(source, 2, 3)).toEqual([11, source.length]);
    expect(lineRangeToSelection(source, 4, 4)).toBeNull();
  });

  it("combines authoring sections into a single source selection", () => {
    const source = ["risk block", "intent block", "execution block", "done"].join("\n");
    const sections = [
      { start_line: 3, end_line: 3 },
      { start_line: 1, end_line: 1 }
    ];

    expect(sectionsToSelection(source, sections)).toEqual([0, 40]);
    expect(sectionsToSelection(source, [])).toBeNull();
  });
});
