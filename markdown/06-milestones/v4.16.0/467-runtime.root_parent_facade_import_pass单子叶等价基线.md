# v4.16.0 runtime.root_parent_facade_import_pass 单子叶等价基线
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FL-01
> 基线: `466-runtime.parent_import_bridge第四轮父叶残余判断.md`
> 目标子叶: `runtime.root_parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FL-02 `runtime.root_parent_facade_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FL-01 `runtime.root_parent_facade_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge / explicit import pass / root facade boundary / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass` | root facade 白箱输入面 |
| 模块树 | `runtime.root_parent_facade_import_pass` | `root_parent_facade_import_pass baseline_frozen` |

---

## 当前真实边界

本批只冻结 `src/runtime/mod.rs` 的 root parent facade 输入面。当前生产级 runtime parent bridge residual 只剩:

```text
src/runtime/mod.rs
remaining_runtime_parent_import_bridge_1
remaining_root_parent_import_bridge_1
```

当前根 residual:

```rust
use super::*;
use axum::extract::Query;
```

其中 `cargo check -p quantpilot` 已把两者都报告为 warning，说明下一步应在 root facade 范围内建立显式化方案，但本批不直接改 Rust。

test-local residual 不纳入本批生产级收口:

```text
src/runtime/run_guard.rs
src/runtime/mutation/ai_proposal/static_check.rs
```

---

## 模块声明面冻结

`module declaration surface frozen`:

```text
backtest_execution_start
backtest_experiment_sweep
backtest_record_store
backtest_replay
event_stream
evidence_health
experiment_limit
mutation_ai_proposal
mutation_parameter_mutation
mutation_shared_governance
query_support
report_ops
response_support
run_guard
run_record_store
run_replay_status
run_session_start
run_v4_handoff
```

BE-001FL-02/03 不得改动 child module declaration、`#[path = "..."]` 映射或文件归属。

---

## public re-export 面冻结

`public re-export surface frozen`:

```text
start_backtest_run
discard_backtest_record
get_backtest_detail
list_backtests
save_backtest_record
get_backtest_replay
stream_run_events
cleanup_runtime_evidence
get_runtime_evidence_health
approve_ai_proposal
claim_ai_proposal_review
create_runtime_ai_proposal
get_runtime_ai_proposal_detail
get_runtime_approval_detail
list_runtime_ai_proposals
list_runtime_approvals
reject_ai_proposal
activate_runtime_parameter_mutation
create_runtime_parameter_mutation
get_runtime_parameter_mutation_detail
list_runtime_parameter_mutations
rollback_runtime_parameter_mutation
create_runtime_report
export_runtime_report_artifact
get_audit_weekly_report
get_ops_daily_report
get_research_monthly_report
get_runtime_report_detail
get_storage_health
list_config_generations
list_merge_records
list_runtime_reports
discard_run_record
get_run_detail
list_runs
save_run_record
get_run_replay
get_run_status
start_test_run
start_v4_runtime_run
discard_experiment_record
get_experiment_detail
list_experiments
save_experiment_record
start_backtest_experiment
```

这些 public / parent-visible 方法仍由 `src/runtime/mod.rs` 作为父级 facade 对外暴露。root import rewrite 不得迁移 handler owner，不得改变 route facade 调用点，不得改变返回类型、状态码、事件顺序或持久化顺序。

---

## private helper bridge 面冻结

`private helper bridge surface frozen`:

```text
execute_backtest_request
MAX_EXPERIMENT_VARIANTS
append_parameter_mutation_events_to_run
build_runtime_parameter_mutation_event
canonical_runtime_parameter_version
governance_with_parameter_version
mutation_event_contract
runtime_parameter_mutation_governance
validate_runtime_parameter_mutation_target
RunInProgressGuard
runtime_simulated_v4_matrix
runtime_v4_static_bundle
```

这些 helper 是子模块通过父级 facade 进行受控沟通的白箱节点。BE-001FL-03 只允许在 `src/runtime/mod.rs` 顶部 import 面做最小显式化，不得新增 sibling horizontal link。

---

## query / response parent surface 冻结

`query_support parent surface frozen`:

```text
clean_optional_filter
normalized_replay_options
AuditWeeklyQuery
OpsDailyQuery
ResearchMonthlyQuery
RuntimeAiProposalListQuery
RuntimeApprovalListQuery
RuntimeParameterMutationListQuery
RuntimeReplayQuery
```

`response_support parent surface frozen`:

```text
DiscardRuntimeArtifactResponse
MergeRecordEntry
MergeRecordsResponse
```

`RuntimeApprovalListQuery` 是此前 parent facade import pass 暴露出的 hidden parent input，必须保持显式可见。

---

## 不进入范围

本批不处理:

1. 不修改 Rust 代码。
2. 不删除 `use super::*`。
3. 不删除 `use axum::extract::Query`。
4. 不处理 test-local residual。
5. 不改变任何 child module、handler、schema、route facade、state owner 或 frontend caller。
6. 不启动 release transition，不新增 sibling horizontal link。
7. 不宣称 `runtime.parent_import_bridge stop_split: true`。
8. 不宣称 backend.runtime 或 Rust 重构完成。

---

## 下一步边界

下一步只能进入:

```text
BE-001FL-02
runtime.root_parent_facade_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass
```

BE-001FL-02 只负责提出 root facade import rewrite 方案；实际删除 `use super::*` 和 `use axum::extract::Query` 只能在 BE-001FL-03 发生。

---

## 验证要求

本批是 `no code movement` 基线，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

后续实际抽离批次至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001FL-01 完成时，必须说明:

1. 本批是 `no code movement` 单子叶等价基线。
2. `src/runtime/mod.rs` 尚未改写。
3. `root_parent_facade_import_pass baseline_frozen` 已记录 module declaration、public re-export、private helper bridge、query_support 与 response_support parent surface。
4. 当前生产级 residual 仍为 `remaining_runtime_parent_import_bridge_1` / `remaining_root_parent_import_bridge_1`。
5. 下一步只能进入 BE-001FL-02 抽离方案。
6. `old_three_leaf_pause_target_cancelled` 保持取消状态。
7. `progress_report_instruction_discarded` 保持丢弃状态。

不得宣称 runtime parent bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `467-runtime.root_parent_facade_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.root_parent_facade_import_pass` 基线冻结完成。
3. 下一步固定为 BE-001FL-02 抽离方案。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
