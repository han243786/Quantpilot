export const CODE_MODE_TASK_LANES_NOTE =
  "\u4e00\u6b21\u53ea\u4fdd\u6301\u4e00\u4e2a\u4e3b\u901a\u9053\u6d3b\u8dc3\uff0c\u5fc5\u8981\u65f6\u518d\u5c55\u5f00\u8f85\u52a9\u901a\u9053\u3002";

export function resolveCodeLaneStatusTone(codeLaneState = {}) {
  return codeLaneState.mode === "manual" ? "warning" : "muted";
}

export function buildCodeLaneNoticeClassName(notice, isVisible) {
  if (!notice) return "";
  return `workspace-inspector-stack__reason workspace-inspector-stack__reason--${notice.tone}${
    isVisible ? "" : " workspace-inspector-stack__reason--faded"
  }`;
}

export function buildCodeLaneFocusMessage(notice) {
  if (!notice?.focusLabel) return null;
  return notice.focusChanged
    ? `\u753b\u5e03\u7126\u70b9\u5df2\u5207\u6362\u5230 ${notice.focusLabel}\u3002`
    : `\u753b\u5e03\u7126\u70b9\u4fdd\u6301\u5728 ${notice.focusLabel}\u3002`;
}

export function buildCodeInspectorTabClassName(activeInspectorId, panelId) {
  return `workspace-inspector-nav__tab${
    activeInspectorId === panelId ? " workspace-inspector-nav__tab--active" : ""
  }`;
}

export function isCodeInspectorExpanded(expandedCodeInspectors = [], panelId) {
  return expandedCodeInspectors.includes(panelId);
}

export function buildCodeInspectorDisclosureLabel(isExpanded, panelLabel) {
  return `${isExpanded ? "\u9690\u85cf" : "\u663e\u793a"} ${panelLabel}\u901a\u9053`;
}
