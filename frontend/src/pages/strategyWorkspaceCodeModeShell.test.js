import { describe, expect, it } from "vitest";

import {
  CODE_MODE_TASK_LANES_NOTE,
  buildCodeInspectorDisclosureLabel,
  buildCodeInspectorTabClassName,
  buildCodeLaneFocusMessage,
  buildCodeLaneNoticeClassName,
  isCodeInspectorExpanded,
  resolveCodeLaneStatusTone
} from "./strategyWorkspaceCodeModeShell";

describe("strategyWorkspaceCodeModeShell", () => {
  it("projects code lane status and note text", () => {
    expect(CODE_MODE_TASK_LANES_NOTE).toBe(
      "\u4e00\u6b21\u53ea\u4fdd\u6301\u4e00\u4e2a\u4e3b\u901a\u9053\u6d3b\u8dc3\uff0c\u5fc5\u8981\u65f6\u518d\u5c55\u5f00\u8f85\u52a9\u901a\u9053\u3002"
    );
    expect(resolveCodeLaneStatusTone({ mode: "manual" })).toBe("warning");
    expect(resolveCodeLaneStatusTone({ mode: "auto" })).toBe("muted");
    expect(resolveCodeLaneStatusTone()).toBe("muted");
  });

  it("builds notice classes and focus messages", () => {
    expect(buildCodeLaneNoticeClassName(null, true)).toBe("");
    expect(buildCodeLaneNoticeClassName({ tone: "warning" }, true)).toBe(
      "workspace-inspector-stack__reason workspace-inspector-stack__reason--warning"
    );
    expect(buildCodeLaneNoticeClassName({ tone: "info" }, false)).toBe(
      "workspace-inspector-stack__reason workspace-inspector-stack__reason--info workspace-inspector-stack__reason--faded"
    );
    expect(
      buildCodeLaneFocusMessage({
        focusLabel: "\u8bca\u65ad",
        focusChanged: true
      })
    ).toBe("\u753b\u5e03\u7126\u70b9\u5df2\u5207\u6362\u5230 \u8bca\u65ad\u3002");
    expect(
      buildCodeLaneFocusMessage({
        focusLabel: "\u9009\u4e2d\u9879",
        focusChanged: false
      })
    ).toBe("\u753b\u5e03\u7126\u70b9\u4fdd\u6301\u5728 \u9009\u4e2d\u9879\u3002");
    expect(buildCodeLaneFocusMessage({})).toBeNull();
  });

  it("projects inspector tab and disclosure state", () => {
    expect(buildCodeInspectorTabClassName("code", "code")).toBe(
      "workspace-inspector-nav__tab workspace-inspector-nav__tab--active"
    );
    expect(buildCodeInspectorTabClassName("params", "code")).toBe(
      "workspace-inspector-nav__tab"
    );
    expect(isCodeInspectorExpanded(["params", "code"], "code")).toBe(true);
    expect(isCodeInspectorExpanded(["params"], "code")).toBe(false);
    expect(buildCodeInspectorDisclosureLabel(true, "\u6e90\u7801")).toBe(
      "\u9690\u85cf \u6e90\u7801\u901a\u9053"
    );
    expect(buildCodeInspectorDisclosureLabel(false, "\u914d\u7f6e")).toBe(
      "\u663e\u793a \u914d\u7f6e\u901a\u9053"
    );
  });
});
