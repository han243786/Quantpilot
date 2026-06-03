# v4.16.0 backend.app_state_wiring single leaf closeout

> Batch: BE-001OZ-01
> Node: `backend.app_state_wiring`
> Parent: `backend`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.app_state_wiring` is closed as a thin backend wiring leaf.

Decision:

`stop_split: true`

## Current White-Box Boundary

| Surface | Owner | Status |
| --- | --- | --- |
| `health` | `src/backend/app_state_wiring.rs` -> `health_route` -> `app_runtime_helpers::health` | Preserved. |
| `new_app_state` | `src/backend/app_state_wiring/state_factory.rs` re-export | Preserved. |
| `attach_state` | `src/backend/app_state_wiring.rs` | Preserved as router state attachment facade. |

## Equivalence Evidence

The current leaf matches the earlier BE-001C-08 and BE-001E-07 boundary:

- `src/backend/app_state_wiring.rs` remains only the parent facade for health, state factory, and router state attachment;
- `src/backend/app_state_wiring/health_route.rs` still delegates to the original health implementation;
- `src/backend/app_state_wiring/state_factory.rs` still re-exports the original state factory;
- no AppState field owner, lock order, storage directory owner, health response schema, or startup chain behavior moved.

## Split Decision Rules

The required leaf split rules were evaluated:

| Rule | Result |
| --- | --- |
| Public boundary | Already represented by `health`, `new_app_state`, and `attach_state`; no new public owner emerges. |
| State-machine phase | Not applicable without moving AppState ownership or lock order. |
| Strategy branch | Not applicable; this is a wiring facade, not a behavioral strategy family. |
| Independent failure mode | Health and state factory are delegated compatibility surfaces; deeper split would not isolate a new failure mode. |
| Communication cost | Further split would only add parent bridge noise around tiny facades. |

## Hard Boundaries

This closeout does not authorize:

- AppState field migration;
- lock order changes;
- health schema changes;
- storage directory ownership changes;
- frontend caller migration;
- release transition connection proposals.

## Parent Return

Return to the `backend` parent residual queue.

Remaining backend top-level residual:

- `backend.test_support`.

## Next Step

BE-001PA-01 `backend` parent_residual_judgment selects `backend.test_support`.

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
