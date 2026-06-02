# v4.16.0 backend.ops_governance.hotswap equivalence baseline and extraction plan

> Batch: BE-001LM-01
> Node: `backend.ops_governance.hotswap`
> Parent: `backend.ops_governance`
> Stage: `baseline_plan`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.hotswap` is frozen as a route facade plus root handler boundary.

The current child facade lives at `src/backend/ops_governance/hotswap.rs` and registers three routes:

- `POST /api/hotswap`
- `GET /api/hotswap/list`
- `GET /api/hotswap/:hotswap_id`

The current handler owner remains `src/hotswap_api.rs`.

BE-001LM-02 may perform the minimum physical extraction needed to make the hotswap child own its handlers, but it must preserve route behavior and leave sibling ops domains untouched.

## Route Chain

| Layer | File | Boundary |
| --- | --- | --- |
| app router | `src/app_router.rs` | Calls `interface_boundary::register_hotswap_routes(router)`. |
| interface boundary | `src/backend/interface_boundary.rs` | Bridges hotswap routes into ops governance. |
| ops governance bridge | `src/backend/interface_boundary/ops_governance_bridge.rs` | Calls `crate::backend::ops_governance::register_hotswap_routes(router)`. |
| ops governance parent | `src/backend/ops_governance.rs` | Calls `hotswap::register_routes(router)`. |
| hotswap child facade | `src/backend/ops_governance/hotswap.rs` | Registers three `/api/hotswap*` routes. |
| root handler owner | `src/hotswap_api.rs` | Implements submit/status/list behavior. |

## Handler Baseline

| Handler | Input | Output | State and behavior |
| --- | --- | --- | --- |
| `submit_hotswap` | `auth::UserId`, `State<AppState>`, `Json<SubmitHotSwapRequest>` | `200 OK` with `HotSwapResponse` or `400 BAD_REQUEST` problem JSON | Creates `hotswap-{now_ms}`, stores a `HotSwapRecord` with status `proposed` and step `idle`, rejects empty `module_targets`, rejects empty `module_key`, writes by `auth::scoped_key`. |
| `get_hotswap_status` | `auth::UserId`, `State<AppState>`, `Path<String>` | `200 OK` with `HotSwapStatusResponse` or `404 NOT_FOUND` problem JSON | Reads `state.hotswap_records`, scopes by user and hotswap id, returns status/step/events. |
| `list_hotswaps` | `auth::UserId`, `State<AppState>` | `200 OK` with `{ "hotswaps": [...] }` | Reads all scoped records for the user and returns id/status/step/started/success projection. |

## Data Baseline

`src/frontend_api_types.rs` owns the hotswap DTOs:

- `HotSwapModuleTargetDto`
- `SubmitHotSwapRequest`
- `HotSwapResponse`
- `HotSwapStatusResponse`
- `HotSwapRecord`

`SubmitHotSwapRequest` keeps the default window fields:

- `safe_window_timeout_ms = 30_000`
- `observation_window_ms = 60_000`
- `shadow_replay_window_ms = 120_000`

`src/lib.rs` owns `AppState.hotswap_records: Arc<RwLock<BTreeMap<String, HotSwapRecord>>>`.

## Known Proof Gap

No dedicated hotswap test file or assertion was found in the current search result. BE-001LM-02 must therefore keep the movement mechanically small and prove equivalence with compile and governance gates. Any behavior-changing hotswap validation, response schema, auth scope, AppState owner, or persistence change requires a new proposal.

## Allowed BE-001LM-02 Movement

BE-001LM-02 may:

- move hotswap handler implementation under `backend.ops_governance.hotswap`;
- update the hotswap route facade to call the moved handlers through the child module;
- keep or remove the root `hotswap_api` wrapper only if compile and route behavior stay equivalent.

BE-001LM-02 must not:

- move sandbox, alerts, snapshots, runbook, chaos, runtime, storage security, capability, app state wiring, or test support internals;
- change route paths, HTTP methods, response status codes, problem JSON fields, scoped key behavior, DTO schema, AppState owner, or lock order;
- introduce sibling shortcuts across ops child modules;
- propose release transition.

## Next Step

BE-001LM-02 backend.ops_governance.hotswap extract_closeout

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
