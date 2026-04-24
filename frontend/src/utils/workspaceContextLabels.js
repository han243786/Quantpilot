export function canvasFocusStatusLabel(focusMode) {
  if (focusMode === "issues") return "问题聚焦";
  if (focusMode === "recent") return "最近编辑聚焦";
  return "当前选中聚焦";
}

export function taskLaneStatusLabel(codeLaneState, pinnedInspectorDefinition) {
  if (codeLaneState?.mode === "manual" && pinnedInspectorDefinition?.label) {
    return `固定到${pinnedInspectorDefinition.label}`;
  }
  return "自动跟随";
}
