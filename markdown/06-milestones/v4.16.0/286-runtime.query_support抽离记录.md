# v4.16.0 runtime.query_support 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CN-03
> 基准: `285-runtime.query_support抽离方案.md`、`284-runtime.query_support单子叶等价基线.md`
> 目标子叶: `runtime.query_support`
> 模块树坐标: `root.backend.runtime.runtime.query_support`
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CN-03 `runtime.query_support` 实际抽离 | 单子叶抽离记录 |
| 规范矩阵 | 父子通信、DTO visibility、field/helper visibility、release transition guard | 落地 |
| 引导矩阵 | `root.backend.runtime.runtime.query_support` | child 文件落地 |
| 模块树 | `runtime.query_support` | 白箱真实文件登记 |

---

## 代码变更

本批创建:

```text
src/runtime/query_support.rs
```

并更新:

```text
src/runtime/mod.rs
src/runtime/run.rs
src/runtime/mutation.rs
```

`src/runtime/mod.rs` 新增:

```rust
mod query_support;
```

父级以 plain use 回填 caller-facing query surface:

```rust
use query_support::{
    clean_optional_filter,
    normalized_replay_options,
    AuditWeeklyQuery,
    OpsDailyQuery,
    ResearchMonthlyQuery,
    RuntimeAiProposalListQuery,
    RuntimeApprovalListQuery,
    RuntimeParameterMutationListQuery,
    RuntimeReplayQuery,
};
```

调用方文件未改动，继续通过 `use super::*` 访问父级受控 surface。

---

## 已迁移 item

以下 9 个 item 已迁入 `src/runtime/query_support.rs`:

| item | 迁移后 visibility | field visibility | 等价约束 |
| --- | --- | --- | --- |
| `RuntimeReplayQuery` | `pub(crate)` | `pub(super)` | 保持 cursor / checkpoint precedence、limit、filter、key_only 语义 |
| `RuntimeParameterMutationListQuery` | `pub(crate)` | `pub(super)` | 保持 source / pagination semantics |
| `RuntimeAiProposalListQuery` | `pub(crate)` | `pub(super)` | 保持 source / status filter semantics |
| `RuntimeApprovalListQuery` | `pub(crate)` | `pub(super)` | 保持 `review_state` default 与 filter semantics |
| `OpsDailyQuery` | `pub(crate)` | `pub(super)` | 保持 optional date behavior |
| `AuditWeeklyQuery` | `pub(crate)` | `pub(super)` | 保持 optional week_start behavior |
| `ResearchMonthlyQuery` | `pub(crate)` | `pub(super)` | 保持 optional month behavior |
| `clean_optional_filter` | `pub(super)` | n/a | 保持 trim / empty-filter behavior |
| `normalized_replay_options` | `pub(super)` | n/a | 保持 `RuntimeReplayOptions` mapping、cursor precedence、sequence cursor、key_only filter、`DEFAULT_REPLAY_PAGE_SIZE` 与 `MAX_REPLAY_PAGE_SIZE` |

`DEFAULT_REPLAY_PAGE_SIZE` 与 `MAX_REPLAY_PAGE_SIZE` 已随 `normalized_replay_options` 迁入 child 并保持 private const。`MAX_EXPERIMENT_VARIANTS` 未迁移，仍留在 `src/runtime/mod.rs`。

---

## 可见性修正

BE-001CN-02 方案原本倾向将 DTO 类型本体降为 `pub(super)`，实际编译证明 Query DTO 会出现在 `pub(crate)` handler 签名中，route registration 需要能看到这些参数类型。因此 BE-001CN-03 采用以下硬规则:

1. DTO 类型本体保持 `pub(crate)`，等价于迁移前的可见性，不扩大原 API。
2. DTO 字段统一为 `pub(super)`，只允许 runtime 父级及其 child 读取。
3. `clean_optional_filter` 与 `normalized_replay_options` 为 `pub(super)`。
4. 父级不得使用 `pub(crate) use query_support::{...};`，只允许普通 `use query_support::{...};`。

---

## 调用方等价

以下调用方文件未改动:

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

开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、`AppState` 或任何横向连接直接依赖 `runtime.query_support`。

---

## 明确未迁移

- response support: `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`。
- run guard: `RunInProgressGuard`。
- experiment limit: `MAX_EXPERIMENT_VARIANTS`。
- parent include deletion: `include!("run.rs")`、`include!("mutation.rs")`、`include!("backtest.rs")` 均保留。
- schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard。

`src/runtime/mutation.rs` 已被抽空为 drained include，并保留 `// Drained include retained until the runtime parent support closeout.`，等待后续父叶残余判断决定是否处理 parent include。

---

## 验证要求

本批提交前必须执行:

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

下一步只能进入:

```text
BE-001CN-04 runtime.query_support 单叶 closeout
```

BE-001CN-04 只判断 query_support 本叶是否还值得继续拆分，不得跳过 closeout 处理 response support、run guard、experiment limit、parent include 删除或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CN-03 完成时，必须说明:

1. `src/runtime/query_support.rs` 已创建。
2. 7 个 Query DTO、`clean_optional_filter` 与 `normalized_replay_options` 已迁入 child。
3. DTO 类型本体为 `pub(crate)`，字段为 `pub(super)`；两个 helper 为 `pub(super)`。
4. 父级只保留 `mod query_support;` 与普通 `use query_support::{...};`，没有 `pub(crate) use`。
5. `DEFAULT_REPLAY_PAGE_SIZE` 与 `MAX_REPLAY_PAGE_SIZE` 已迁入 child，`MAX_EXPERIMENT_VARIANTS` 仍在父级。
6. 调用方文件未改动，仍通过 `use super::*`。
7. `include!("run.rs")`、`include!("mutation.rs")`、`include!("backtest.rs")` 均保留。
8. response support、run guard、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard 均未迁移。
9. 下一步只能进入 BE-001CN-04 单叶 closeout。

不得宣称 `backend.runtime` 已完成、parent support 已整体抽离、response/run guard 已处理、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `src/runtime/query_support.rs` 进入模块树和全量树。
2. `286-runtime.query_support抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. Query DTO、filter normalization、replay option normalization 只从父级抽离到 child，不改变 HTTP query parsing、pagination、filter 或 replay cursor semantics。
4. Rust 等价测试、治理门禁、全量树覆盖和 `git diff --check` 均通过。
