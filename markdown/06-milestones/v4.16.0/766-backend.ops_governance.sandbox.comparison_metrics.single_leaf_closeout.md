# v4.16.0 backend.ops_governance.sandbox.comparison_metrics single leaf closeout continues split

> Batch: BE-001MF-03
> Node: `backend.ops_governance.sandbox.comparison_metrics`
> Parent: `backend.ops_governance.sandbox`
> Stage: `single_leaf_closeout`
> Movement: No code movement.

---

## Summary

`backend.ops_governance.sandbox.comparison_metrics` remains open for further recursive splitting.

Decision:

`stop_split: false`

The extracted parent currently contains two concrete owners:

- `v4_replay_shape`: pure v4 artifact replay-shape comparison and risk rejection counting with a direct test.
- `backtest_projection`: AppState backtest selection plus `BacktestRecord` to `SandboxMetrics` projection and fidelity fallback.

## Split Gate Result

Further splitting is required by the recursive split rules:

| Rule | Result |
| --- | --- |
| Concrete owner exists? | Yes. v4 replay-shape helper and backtest projection are separate owners. |
| Independent IO or state failure mode? | Yes. backtest projection reads AppState, while v4 replay-shape is pure artifact comparison. |
| Parent-child communication improves? | Yes. Each child can return a narrow result instead of sharing mixed helper ownership. |
| Local proof improves? | Yes. The existing replay-shape test can live with its pure child. |
| Line count only? | No. Split is behavior and ownership driven. |

## Parent Residual Queue

| Child | Current boundary | Decision |
| --- | --- | --- |
| `backend.ops_governance.sandbox.comparison_metrics.v4_replay_shape` | v4 artifact replay-shape helper, risk rejection counter, and direct test. | Candidate for next parent residual selection. |
| `backend.ops_governance.sandbox.comparison_metrics.backtest_projection` | AppState backtest selection, metrics projection, and fidelity fallback. | Candidate for later selection. |

## Hard Boundaries

The next parent residual pass must not move:

- metrics_evaluation closed leaf internals;
- proposal loader;
- disk loader;
- report_api closed leaf internals;
- verification_run closed parent internals;
- root compatibility bridge exports;
- runtime mutation internals;
- release transition policy.

## Next Step

BE-001MG-01 backend.ops_governance.sandbox.comparison_metrics parent_residual_judgment

## Gates

- `git diff --check`
- `cargo fmt --check`
- `cargo check -p quantpilot`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
