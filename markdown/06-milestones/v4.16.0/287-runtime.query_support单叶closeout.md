# v4.16.0 runtime.query_support 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CN-04  
> 基准: `286-runtime.query_support抽离记录.md`、`285-runtime.query_support抽离方案.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.query_support`  
> 判定: `runtime.query_support stop_split: true`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CN-04 `runtime.query_support` 单叶 closeout | closeout |
| 规范矩阵 | 单叶停止条件、父子通信、DTO/field/helper visibility、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.query_support` | 子叶收口 |
| 模块树 | `runtime.query_support` | `stop_split: true` |

---

## closeout 判定

`runtime.query_support` 当前不继续拆成 replay query / mutation query / report query / normalization 微叶，设置:

```text
runtime.query_support stop_split: true
```

理由:

1. 当前 child 只有 7 个 Query DTO、`clean_optional_filter` 与 `normalized_replay_options`，全部围绕 HTTP query parsing 与 replay option normalization。
2. 这些 item 没有独立状态、锁顺序、持久化 owner、schema owner、runtime persistence owner、storage lifecycle owner、frontend caller 或 release transition guard。
3. 继续拆成 report query、mutation query、replay query 微叶会增加父级 import、治理登记和 caller surface，但不会形成新的稳定白箱。
4. DTO 类型本体保持 `pub(crate)` 是 Axum `pub(crate)` handler 签名所需；字段/helper 已收敛为 `pub(super)`，边界已足够窄。

---

## 当前真实结构

已落地 child:

```text
src/runtime/query_support.rs
```

child 内部保持:

- DTO: `RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`、`OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`。
- DTO 类型本体: `pub(crate)`。
- DTO 字段: `pub(super)`。
- helper: `pub(super)`。
- `DEFAULT_REPLAY_PAGE_SIZE` 与 `MAX_REPLAY_PAGE_SIZE`: private const。

父级 `src/runtime/mod.rs` 保留受控 child 声明:

```rust
mod query_support;
```

父级只用普通 `use query_support::{...};` 回填 caller-facing query surface，不使用 `pub(crate) use`。

`MAX_EXPERIMENT_VARIANTS` 仍留在 `src/runtime/mod.rs`。

---

## 调用方等价

以下调用方仍通过 `use super::*` 访问父级受控 surface:

- `src/runtime/run/replay_status.rs`
- `src/runtime/backtest/replay.rs`
- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`

父子通信路径保持:

```text
runtime child callers
  -> src/runtime/mod.rs controlled query surface
  -> runtime.query_support
```

开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连该 child。

---

## 明确排除

- 不继续细拆 replay query / mutation query / report query / normalization 微叶。
- 不修改 `src/runtime/query_support.rs`。
- 不修改调用方文件，不改变 `use super::*` 兼容路径。
- 不迁移 `RunInProgressGuard`。
- 不迁移 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 `include!("backtest.rs")`。
- 不迁移 schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` closeout，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CO-01 backend.runtime 第五轮父叶残余判断
```

BE-001CO-01 需要重新审视 `backend.runtime` 父级在 `runtime.query_support` closeout 后是否仍存在值得抽离的 run guard / response support / parent include residual。不得从 `runtime.query_support` 继续细拆，不得启动 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CN-04 完成时，必须说明:

1. 本批次是 `no code movement` closeout。
2. `runtime.query_support stop_split: true`。
3. 不继续拆 replay query / mutation query / report query / normalization 微叶。
4. `src/runtime/query_support.rs` 仍承接 7 个 Query DTO 与两个 normalization helper。
5. DTO 类型本体仍为 `pub(crate)`，字段/helper 仍为 `pub(super)`。
6. 调用方仍通过 `use super::*`，父级仍是普通 `use query_support::{...};`。
7. `RunInProgressGuard`、`DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`MAX_EXPERIMENT_VARIANTS` 与 parent include deletion 均未处理。
8. 下一步只能进入 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断。

不得宣称 `backend.runtime` 已完成、run guard/response support 已处理、parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `287-runtime.query_support单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.query_support` 设置为 `stop_split: true`。
3. 全局递归下一步固定为 BE-001CO-01 `backend.runtime` 第五轮父叶残余判断。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
