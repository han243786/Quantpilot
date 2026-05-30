# v4.16.0 runtime.parent_include_cleanup 清理记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CV-03
> 基准: `306-runtime.parent_include_cleanup抽离方案.md`
> 目标子叶: `runtime.parent_include_cleanup`
> 模块树坐标: `root.backend.runtime.runtime.parent_include_cleanup`
> 代码动作: actual cleanup

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CV-03 `runtime.parent_include_cleanup` 实际 cleanup | 抽离执行 |
| 规范矩阵 | drained include cleanup、deleted drained file、public surface preservation、release transition guard | 执行边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_include_cleanup` | cleanup record |
| 模块树 | `runtime.parent_include_cleanup` | 实际清理登记 |

---

## 实际变更

本批次严格执行 BE-001CV-02 的 single cleanup batch:

1. 从 `src/runtime/mod.rs` 删除 `include!("backtest.rs")`。
2. 从 `src/runtime/mod.rs` 删除 `include!("run.rs")`。
3. 从 `src/runtime/mod.rs` 删除 `include!("mutation.rs")`。
4. 删除 drained 文件 `src/runtime/backtest.rs`。
5. 删除 drained 文件 `src/runtime/run.rs`。
6. 删除 drained 文件 `src/runtime/mutation.rs`。

本批次不新建 planned code file。`runtime.parent_include_cleanup` 是 parent support cleanup leaf，不是新 Rust 模块。

---

## 保留 surface

BE-001CV-03 后继续由既有 child module 承接 public surface:

```rust
pub(crate) use backtest_execution_start::start_backtest_run;
pub(crate) use backtest_record_store::{
    discard_backtest_record, get_backtest_detail, list_backtests, save_backtest_record,
};
pub(crate) use backtest_replay::get_backtest_replay;
pub(crate) use backtest_experiment_sweep::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
    start_backtest_experiment,
};
pub(crate) use run_record_store::{discard_run_record, get_run_detail, list_runs, save_run_record};
pub(crate) use run_replay_status::{get_run_replay, get_run_status};
pub(crate) use run_session_start::start_test_run;
pub(crate) use run_v4_handoff::start_v4_runtime_run;
pub(crate) use mutation_ai_proposal::{
    approve_ai_proposal, claim_ai_proposal_review, create_runtime_ai_proposal,
    get_runtime_ai_proposal_detail, get_runtime_approval_detail, list_runtime_ai_proposals,
    list_runtime_approvals, reject_ai_proposal,
};
pub(crate) use mutation_parameter_mutation::{
    activate_runtime_parameter_mutation, create_runtime_parameter_mutation,
    get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations,
    rollback_runtime_parameter_mutation,
};
```

本批次未修改 owner、visibility、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState、lock order 或 release transition guard。

---

## 等价结论

三条 drained `include!(...)` 已无有效 Rust item，删除后:

- `src/runtime/mod.rs` 仍保留所有 child module declaration 与受控 re-export。
- run / backtest / mutation public handler owner 仍是已 closeout child module。
- route registration、request/response schema、state owner、persistence owner 与 frontend caller 均未变更。
- 没有新增 sibling horizontal link，也没有启动 release transition。

---

## 验证要求

本实际 cleanup 批次提交前必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot runtime_run_guard_resets_on_drop
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001CW-01 backend.runtime 第九轮父叶残余判断
```

该判断只允许重新扫描 `src/runtime/mod.rs` 和 backend.runtime 模块树残余，决定 `backend.runtime stop_split` 是否可以转为 true。不得顺手处理 schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState、lock order 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CV-03 完成时，必须说明:

1. 本批次仅删除三条 drained `include!(...)` 与三个 drained 文件。
2. `runtime.parent_include_cleanup` 不是新 Rust module。
3. public handler owner、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、AppState、lock order 和 release transition guard 均未处理。
4. 下一步只能进入 BE-001CW-01 `backend.runtime` 第九轮父叶残余判断。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成或发布过渡已启动。

---

## 验收标准

1. `307-runtime.parent_include_cleanup清理记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mod.rs` 不再包含 `include!("backtest.rs")`、`include!("run.rs")` 或 `include!("mutation.rs")`。
3. `src/runtime/backtest.rs`、`src/runtime/run.rs` 与 `src/runtime/mutation.rs` 已删除。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
