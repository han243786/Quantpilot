# FE-0005 Worktree Pre-Commit Hook Gate

Status: closed.

## Problem

The tooling pre-commit step attempted to read `.git/hooks/pre-commit` directly. In this derived worktree, `.git` is a worktree pointer instead of the common repository git directory, so the check reported a missing hook path even though commits still ran the configured hook.

## Decision

Resolve the hook path with `git rev-parse --git-path hooks/pre-commit`.

This keeps the check valid for both normal checkouts and derived worktrees.

## Changed Files

- `tools/check-pre-commit-hook.ps1`
- `markdown/00-frontend-refactor-governance/frontend-recursive-state.json`
- `markdown/00-frontend-refactor-governance/records/FE-0005-worktree-pre-commit-hook-gate.md`

## Verification

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-pre-commit-hook.ps1`: passed.
- `git diff --check`: passed.
- `frontend-recursive-state.json` JSON parse: passed.

## Further-Split Decision

No further split is useful. This is a small tooling compatibility fix required for reliable per-step commits in the frontend recursion worktree.
