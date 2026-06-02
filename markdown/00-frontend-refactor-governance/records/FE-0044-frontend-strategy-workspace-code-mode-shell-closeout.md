# FE-0044 Frontend Strategy Workspace Code Mode Shell Closeout

Status: closed.

## Leaf Node

`frontend.strategy_workspace.code_mode_shell`

## Code Changes

- Added `frontend/src/pages/strategyWorkspaceCodeModeShell.js`.
- Added `frontend/src/pages/strategyWorkspaceCodeModeShell.test.js`.
- Updated `frontend/src/pages/StrategyWorkspaceCodeTab.jsx` to delegate task-lane note text, lane status tone, lane notice class names, focus-change text, inspector tab class names, disclosure expanded checks, and disclosure labels to the extracted shell module.

## Preserved Behavior

- Code mode still renders module sidebar, strategy canvas, and inspector rail in the same split layout.
- Manual code-lane mode still shows warning tone; automatic mode still shows muted tone.
- Code-lane notice visibility and faded class behavior remain equivalent.
- Focus notice text still distinguishes between changed focus and retained focus.
- Active inspector tabs and secondary disclosure labels still render the same active/expanded states.

## Public Inputs

- Code lane state.
- Code lane notice and visibility state.
- Active inspector id.
- Inspector panel ids and labels.
- Expanded inspector id list.

## Public Outputs

- `CODE_MODE_TASK_LANES_NOTE`.
- `resolveCodeLaneStatusTone(codeLaneState)`.
- `buildCodeLaneNoticeClassName(notice, isVisible)`.
- `buildCodeLaneFocusMessage(notice)`.
- `buildCodeInspectorTabClassName(activeInspectorId, panelId)`.
- `isCodeInspectorExpanded(expandedCodeInspectors, panelId)`.
- `buildCodeInspectorDisclosureLabel(isExpanded, panelLabel)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/pages/strategyWorkspaceCodeModeShell.test.js src/pages/StrategyWorkspacePage.codeMode.test.jsx`: passed, 2 test files and 11 tests.

## Further-Split Decision

`frontend.strategy_workspace.code_mode_shell` does not need a deeper split yet. The shell projection is now separated from the rendering component while lazy-loaded inspector components remain mounted from the code tab.

## Residuals

- Continue with `frontend.strategy_workspace.dashboard_overview`.
