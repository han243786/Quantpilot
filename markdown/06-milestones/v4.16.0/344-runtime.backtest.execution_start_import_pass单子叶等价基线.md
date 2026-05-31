# v4.16.0 runtime.backtest.execution_start_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DK-01
> 基准: `343-runtime.backtest_import_pass第三轮父叶残余判断.md`
> 目标子叶: `runtime.backtest.execution_start_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DK-02 `runtime.backtest.execution_start_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DK-01 `runtime.backtest.execution_start_import_pass` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | explicit import pass、parent surface、execution_start pocket、release transition guard | 基线冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass` | execution_start import 白箱 |
| 模块树 | `runtime.backtest.execution_start_import_pass` | 新增基线 |

---

## 基线范围

本基线冻结以下五文件 import pocket:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

这些文件仍存在 `use super::*` 或 `super::`，是 `runtime.backtest` 下最后一组 parent bridge 依赖。BE-001DK-01 只建立等价基线，不改 Rust。

```text
remaining_parent_import_bridge_28
```

---

## 白箱输入面

### `execution_start.rs`

父级入口:

```text
start_backtest_run
execute_backtest_request
execute_v4_backtest_request
```

当前父级输入依赖包括:

```text
auth::UserId
State<AppState>
Json<FrontendRunRequest>
BacktestRunResponse
BacktestRecord
FrontendRunRequest
Value
StatusCode
json_bad_request
json_bad_request_with_details
internal_error
io_error
current_time_ms
normalize_actor_identity
collaboration_with_run_actor
validate_runtime_capability_guard
validate_runtime_config_capabilities
validate_backtest_execution_assumption_overrides
build_compile_runtime_targets_from_graph
merge_runtime_targets
runtime_governance_snapshot
collect_frontend_events_for_backtest
prepend_capability_snapshot_event
attach_runtime_event_envelopes
validate_runtime_event_envelopes
account_summary_from_portfolio
build_backtest_spec
build_backtest_artifact_views
maybe_spill_transient_backtest_record
backtest_run_response
safe_eprintln
auth::scoped_key
```

内部子模块输入:

```text
prepare_legacy_backtest_dispatch
run_legacy_backtest_dispatch
LegacyBacktestDispatchOutput
build_v4_backtest_output
frontend_events_from_v4_backtest_artifact
v4_equity_curve_from_artifact
is_v4_backtest_request
resolve_v4_backtest_graph
resolve_v4_backtest_market_event_type
resolve_v4_backtest_symbols
run_v4_backtest_runtime_execution
```

### `legacy_dispatch.rs`

白箱方法与结构:

```text
LegacyBacktestDispatchOutput
LegacyBacktestDispatchPlan
prepare_legacy_backtest_dispatch
run_legacy_backtest_dispatch
```

当前依赖面包括 QS compile、execution assumption override、CompileArtifactBundle、FrontendBacktestReplaySource、FastBacktestSandbox、DeterministicTestMode、BACKTEST_DETERMINISTIC_SEED、BTreeMap、tokio::task::spawn_blocking、internal_error。

### `v4_projection.rs`

白箱方法:

```text
build_v4_backtest_output
v4_equity_curve_from_artifact
frontend_events_from_v4_backtest_artifact
```

内部私有方法:

```text
v4_win_rate_from_equity_curve
v4_portfolio_from_artifact
v4_frontend_event
```

本文件还有 test-scope `use super::*`，BE-001DK-02 必须判断是否在同一 import pass 中用显式测试输入替换；不得把测试通配误判为业务 sibling link。

### `v4_request_resolution.rs`

白箱方法:

```text
is_v4_backtest_request
resolve_v4_backtest_graph
resolve_v4_backtest_symbols
resolve_v4_backtest_market_event_type
```

当前依赖面包括 FrontendRunRequest、Value、StatusCode、json_bad_request、json_bad_request_with_code、compile_runtime_protocol_via_qs、compile_runtime_protocol_config、runtime_v4_static_bundle、quantscript v4 audit / handoff、qrpc_core_ir v4 bridge。

### `v4_runtime_execution.rs`

白箱方法:

```text
run_v4_backtest_runtime_execution
```

当前依赖面包括 qrpc_core_ir::v4::V4MachineGraphContract、qrpc_runtime deterministic replay bar/tick builders、runtime_simulated_v4_matrix、V4PaperSimulatedRuntime、spawn_blocking、internal_error。

---

## 等价基线

BE-001DK-02 / BE-001DK-03 不得改变以下行为:

1. `start_backtest_run` 的 route handler 输入、输出、错误结构和 `BacktestRunResponse` 映射。
2. legacy path 的 QS compile、execution assumption override、deterministic mock / historical replay fallback、compile artifact bundle 和 sandbox replay。
3. v4 path 的 graph resolution、symbol resolution、event type selection、runtime execution、projection event ordering、equity curve extraction 和 no-execution-data guard。
4. backtest id 生成、governance snapshot、capability snapshot event、runtime event envelope、record assembly、artifact views、transient spill、state write、audit log。
5. 父子通信规则: 子模块只允许经 `execution_start.rs` 父级白箱暴露给 `runtime.backtest`，不得新增 sibling horizontal link。
6. release transition 未启动，不能提出性能旁路。

---

## 方案阶段必须判断

BE-001DK-02 必须先判断:

1. 五文件是否可作为一个 import rewrite pocket 一次收敛。
2. 若五文件输入面过宽，是否先拆为 `execution_start.v4_request_resolution_import_pass`、`execution_start.v4_projection_import_pass`、`execution_start.v4_runtime_execution_import_pass`、`execution_start.legacy_dispatch_import_pass`。
3. `v4_projection.rs` 的 test-scope `use super::*` 是否与顶层 parent wildcard 同批清理。
4. 是否需要 visibility 调整；若需要，只能在五文件 pocket 内调整，不能扩大到 root bridge、mutation 或 state/persistence owner。

---

## 排除项

- 不修改 Rust 代码。
- 不直接改写五文件 import。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 不新增 sibling horizontal link。
- 不启动 release transition。
- 不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001DK-03 若进入实际 import rewrite，至少额外执行:

```powershell
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001DK-01 完成时，必须说明:

1. 本批是 `no code movement` 单子叶等价基线。
2. 冻结五文件 execution_start import pocket。
3. 下一步只能进入 BE-001DK-02 抽离方案，不能直接改 Rust。
4. `runtime.backtest_import_pass` 尚未完成，parent import bridge 剩余仍为 28。
5. root bridge、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 execution_start import 已经收敛、`runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `344-runtime.backtest.execution_start_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 五文件 pocket、白箱输入面、等价基线和方案阶段判断点均已冻结。
3. 下一步固定为 BE-001DK-02 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
