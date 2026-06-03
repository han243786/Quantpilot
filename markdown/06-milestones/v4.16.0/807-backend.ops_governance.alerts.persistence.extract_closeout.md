# v4.16.0 backend.ops_governance.alerts.persistence extraction closeout

> Batch: BE-001NA-03
> Node: `backend.ops_governance.alerts.persistence`
> Parent: `backend.ops_governance.alerts`
> Stage: `extract_closeout`
> Movement: Alert firing persistence implementation moved into a private child module.

---

## Summary

`backend.ops_governance.alerts.persistence` is extracted into `src/backend/ops_governance/alerts/handlers/persistence.rs`.

The alerts handler owner keeps the parent bridge:

- `persist_alert_firing`

The child owns:

- storage quota enforcement for alert firings;
- alert store directory creation;
- alert firing file path construction;
- atomic JSON write call.

## Boundary Result

| Surface | Result |
| --- | --- |
| Parent write-flow mediation | `acknowledge_flow` and `trigger_engine` still call parent-owned `persist_alert_firing`. |
| Persistence child | The child owns only the storage quota, directory creation, path construction, and atomic write implementation. |
| Storage lifecycle internals | Not moved. |
| Runtime persistence internals | Not moved. |
| Routes and schema | Not moved. |
| Release transition | No release-transition shortcut or sibling direct connection was introduced. |

## Equivalence Proof

The extraction is mechanical:

- return type remains `std::io::Result<()>`;
- storage root remains `std::path::Path::new("storage")`;
- storage namespace remains `"alerts"`;
- lifecycle class remains `StorageLifecycle::Transient`;
- quota check still happens before directory creation;
- `fs::create_dir_all(store_dir).await?` is unchanged;
- file path still uses `format!("{}.json", firing.firing_id)`;
- final write still calls `runtime_persistence::atomic_write_json(&file_path, firing).await`.

## Next Step

BE-001NA-04 backend.ops_governance.alerts.persistence single_leaf_closeout

## Gates

- `cargo fmt`
- `cargo check -p quantpilot`
- `cargo test -p quantpilot backend::ops_governance::alerts::handlers`
- `git diff --check`
- `cargo fmt --check`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
