# v4.16.0 runtime.parent_include_cleanup 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CV-02
> 基准: `305-runtime.parent_include_cleanup单子叶等价基线.md`、`304-backend.runtime第八轮父叶残余判断.md`
> 目标子叶: `runtime.parent_include_cleanup`
> 模块树坐标: `root.backend.runtime.runtime.parent_include_cleanup`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CV-02 `runtime.parent_include_cleanup` 抽离方案 | 方案优化 |
| 规范矩阵 | drained include cleanup、文件删除顺序、回退点、release transition guard | 执行边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_include_cleanup` | cleanup plan |
| 模块树 | `runtime.parent_include_cleanup` | 方案登记 |

---

## 方案判定

采用 single cleanup batch。下一批 BE-001CV-03 才允许执行真实 cleanup:

1. 从 `src/runtime/mod.rs` 删除 `include!("backtest.rs")`。
2. 从 `src/runtime/mod.rs` 删除 `include!("run.rs")`。
3. 从 `src/runtime/mod.rs` 删除 `include!("mutation.rs")`。
4. 删除 drained 文件 `src/runtime/backtest.rs`。
5. 删除 drained 文件 `src/runtime/run.rs`。
6. 删除 drained 文件 `src/runtime/mutation.rs`。

本方案不新建 planned code file。`runtime.parent_include_cleanup` 是 parent support cleanup leaf，不是新的 Rust 模块。

---

## 最小变更范围

### 允许修改

```text
src/runtime/mod.rs
src/runtime/backtest.rs
src/runtime/run.rs
src/runtime/mutation.rs
```

### 允许删除的父级片段

```rust
// Backtest + Experiment handlers
include!("backtest.rs");
```

```rust
// Run + SSE handlers
include!("run.rs");
```

```rust
// Mutation + Proposal + Approval handlers
include!("mutation.rs");
```

若 BE-001CV-03 删除注释行，应仅限这三处已经失效的 include 分组注释，不得重排其他 re-export。

---

## 保留 surface

BE-001CV-03 后仍必须保留:

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

这些出口分别由已 closeout child 拥有，BE-001CV-03 不得修改 owner 或 visibility。

---

## 排除项

- 不迁移 handler、helper、schema、state、persistence 或 storage lifecycle owner。
- 不重命名 `runtime.run.*`、`runtime.backtest.*`、`runtime.mutation.*` child。
- 不修改 `src/runtime/query_support.rs`、`src/runtime/response_support.rs`、`src/runtime/run_guard.rs` 或 `src/runtime/experiment_limit.rs`。
- 不修改 route facade、frontend caller、`AppState`、lock order 或 release transition guard。
- 不启动发布版本过渡，不新增 sibling child 横向连接。

---

## 回退点

若 BE-001CV-03 失败，回退范围仅限:

1. 在 `src/runtime/mod.rs` 放回 `include!("backtest.rs")`。
2. 在 `src/runtime/mod.rs` 放回 `include!("run.rs")`。
3. 在 `src/runtime/mod.rs` 放回 `include!("mutation.rs")`。
4. 恢复 `src/runtime/backtest.rs`，内容为 drained parent include 注释。
5. 恢复 `src/runtime/run.rs`，内容为 drained include 注释。
6. 恢复 `src/runtime/mutation.rs`，内容为 drained include 注释。

不得回退已 closeout 的 child module、public re-export 或治理文档。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前仍需执行:

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

BE-001CV-03 实际 cleanup 后必须至少执行同一组门禁。

---

## 下一步

下一步只能进入:

```text
BE-001CV-03 runtime.parent_include_cleanup 实际 cleanup
```

BE-001CV-03 只允许执行本方案列出的三条 include 删除和三个 drained 文件删除。不得顺手处理 `backend.runtime` 父叶 closeout、模块命名重排、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CV-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. BE-001CV-03 才允许删除三条 drained `include!(...)` 与三个 drained 文件。
3. 本方案不新建 planned code file。
4. public handler owner、route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均未处理。
5. 下一步只能进入 BE-001CV-03 实际 cleanup。

不得宣称 parent include 已删除、`backend.runtime` 已完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `306-runtime.parent_include_cleanup抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案只允许三条 include 删除与三个 drained 文件删除。
3. 全局递归下一步固定为 BE-001CV-03 `runtime.parent_include_cleanup` 实际 cleanup。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
