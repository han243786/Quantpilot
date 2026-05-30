# v4.16.0 runtime.parent_import_bridge 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CX-01
> 基准: `308-backend.runtime第九轮父叶残余判断.md`
> 目标子叶: `runtime.parent_import_bridge`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CX-01 `runtime.parent_import_bridge` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、super wildcard dependency、explicit import pass、release transition guard | 等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge` | 白箱依赖基线 |
| 模块树 | `runtime.parent_import_bridge` | 新残余子叶登记 |

---

## 当前事实

`src/runtime/mod.rs` 当前不再持有 handler/function/struct/enum/const 行为体，但仍作为 parent import bridge 存在:

- `src/runtime/mod.rs` 负责 runtime child declaration、受控 `pub(crate) use`、private `use` 和 `use super::*` 桥接。
- 当前扫描到 46 个 `src/runtime/**.rs` 文件存在 `use super::*` 或 `super::` 依赖。
- 子模块仍通过父级导入面获得 `RunInProgressGuard`、`AuditWeeklyQuery` / query DTO、`DiscardRuntimeArtifactResponse` / response DTO、`append_parameter_mutation_events_to_run` / shared governance helper、`execute_backtest_request`、`runtime_v4_static_bundle`、`runtime_simulated_v4_matrix`、`Query` 等符号。

---

## 依赖清单

当前依赖 parent import bridge 的文件清单:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/record_store.rs
src/runtime/backtest/replay.rs
src/runtime/backtest/start_orchestration.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
src/runtime/event_stream.rs
src/runtime/evidence_health.rs
src/runtime/mod.rs
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/record_query.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/mutation/shared_governance.rs
src/runtime/query_support.rs
src/runtime/report_ops.rs
src/runtime/report_ops/merge_generation_health.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/report_ops/v1_report_endpoints.rs
src/runtime/response_support.rs
src/runtime/run/record_store.rs
src/runtime/run/replay_status.rs
src/runtime/run/session_start.rs
src/runtime/run/v4_handoff.rs
src/runtime/run_guard.rs
```

---

## 等价边界

本基线只冻结事实，不进行代码移动。后续 explicit import pass 必须满足:

1. 每批只选择一个小目录或一个小功能簇，不能一次性改完 46 个文件。
2. 每批只把 `use super::*` 收敛为显式 import 或局部 `super::{...}`，不得改变 handler owner、visibility、route facade、schema owner 或状态 owner。
3. `pub(crate) use` 对外 surface 必须保持等价。
4. `pub(super)` helper 若需要跨 child 访问，必须先由方案说明父子通信边界，不得新增 sibling horizontal link。
5. 未经开发者明确声明发布过渡，不得提出 release transition 或性能旁路。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不批量替换 `use super::*`。
- 本批不移动 schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState、lock order 或 response schema。
- 本批不启动 release transition。

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

后续实际 explicit import pass 还必须按影响面补跑 `cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_mutation`、`cargo test -p quantpilot --test api_ai_proposal` 和相关报告/证据接口测试。

---

## 下一步

下一步只能进入:

```text
BE-001CX-02 runtime.parent_import_bridge 抽离方案
```

BE-001CX-02 只能设计 explicit import pass 的拆分顺序、最小批次、允许修改清单、回退点和验证门禁；不得直接执行 Rust import 改写。

---

## 幻觉检查点

AI 声称 BE-001CX-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 当前真实残余是 parent import bridge，不是 handler 行为体。
3. 当前扫描到 46 个 runtime 文件存在 `use super::*` 或 `super::` 依赖。
4. 下一步只能进入 BE-001CX-02 抽离方案，不能直接批量改 Rust。

不得宣称 parent import bridge 已消除、`backend.runtime` 已完成、Rust 重构已完成或 release transition 已启动。

---

## 验收标准

1. `309-runtime.parent_import_bridge单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 parent import bridge 的当前依赖面和 46 文件清单。
3. 下一步固定为 BE-001CX-02 `runtime.parent_import_bridge` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
