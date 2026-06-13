# Retired Testing Artifacts

> Scope: ignored local screenshots and JSON run reports from the retired testing artifact surface.
> Batch: document deadwood cleanup, 2026-06-13.

This directory records the policy for retired generated testing artifacts. It is not an active evidence surface.

The local artifact subdirectories are intentionally ignored by Git and may be absent after a cleanup sweep:

| Path | Content |
|------|---------|
| `screenshots/` | Old UI screenshot captures from retired manual and scripted testing flows. |
| `test-reports/` | Old JSON scenario reports from retired local testing flows. |

Cleanup rule:

- Historical audit value must live in Git-tracked Markdown reports under `markdown/09-archive/testing-retired/`.
- Ignored screenshot and JSON artifact directories are local cache material, not long-term project documentation.
- A cleanup may delete those ignored directories when they are not part of the current testing surface.

Active testing documentation lives in `markdown/05-testing/`. Retired testing and audit documents live in `markdown/09-archive/testing-retired/`.
