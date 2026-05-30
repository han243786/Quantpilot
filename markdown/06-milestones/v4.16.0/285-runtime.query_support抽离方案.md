# v4.16.0 runtime.query_support 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CN-02  
> 基准: `284-runtime.query_support单子叶等价基线.md`、`283-backend.runtime第四轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.query_support`  
> 模块树坐标: `root.backend.runtime.runtime.query_support`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CN-02 `runtime.query_support` 抽离方案 | 单子叶抽离方案 |
| 规范矩阵 | 父子通信、field visibility、受控 import、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.query_support` | planned child 方案 |
| 模块树 | `runtime.query_support` | 白箱方案登记 |

---

## 当前真实结构

本批仍为 `no code movement`。实际 Rust 结构保持:

```text
src/runtime/mod.rs
src/runtime/run.rs
src/runtime/mutation.rs
```

planned child 文件尚未创建:

```text
src/runtime/query_support.rs
```

BE-001CN-03 才允许创建 planned child 文件并迁移 query DTO / normalization helper。

---

## 目标 child 与父级声明

BE-001CN-03 的唯一 planned child 文件为:

```text
src/runtime/query_support.rs
```

BE-001CN-03 只能在 `src/runtime/mod.rs` 增加受控 child 声明:

```rust
mod query_support;
```

父级 plain use 口径固定为普通 `use query_support::{...};` 回填 caller-facing query surface:

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

该 child 同时承接 filter normalization 与 replay option normalization，不得在 BE-001CN-03 将二者拆散。

不得使用 `pub(crate) use` 扩大 runtime 外部 API。调用方继续通过 `use super::*` 访问父级受控 surface。由于 Query DTO 出现在 `pub(crate)` handler 签名中，DTO 类型本体允许保持原有 `pub(crate)` 可见性；字段与 helper 必须收敛为 `pub(super)`。

---

## 允许迁移清单

BE-001CN-03 仅允许迁移以下 9 个 item 到 `src/runtime/query_support.rs`:

| item | 迁移后 visibility | field visibility | 迁移约束 |
| --- | --- | --- | --- |
| `RuntimeReplayQuery` | `pub(crate)` | `pub(super)` | 不得改变 cursor / checkpoint precedence、limit field 或 filter fields |
| `RuntimeParameterMutationListQuery` | `pub(crate)` | `pub(super)` | 不得改变 source / pagination semantics |
| `RuntimeAiProposalListQuery` | `pub(crate)` | `pub(super)` | 不得改变 source / status filter semantics |
| `RuntimeApprovalListQuery` | `pub(crate)` | `pub(super)` | 不得改变 `review_state` default 或 filter semantics |
| `OpsDailyQuery` | `pub(crate)` | `pub(super)` | 不得改变 optional date behavior |
| `AuditWeeklyQuery` | `pub(crate)` | `pub(super)` | 不得改变 optional week_start behavior |
| `ResearchMonthlyQuery` | `pub(crate)` | `pub(super)` | 不得改变 optional month behavior |
| `clean_optional_filter` | `pub(super)` | n/a | 不得改变 trim / empty-filter behavior |
| `normalized_replay_options` | `pub(super)` | n/a | 不得改变 `RuntimeReplayOptions` mapping、`DEFAULT_REPLAY_PAGE_SIZE`、`MAX_REPLAY_PAGE_SIZE`、cursor precedence、sequence cursor 或 key_only filter |

迁移后 `DEFAULT_REPLAY_PAGE_SIZE` 与 `MAX_REPLAY_PAGE_SIZE` 可以随 `normalized_replay_options` 一并迁入 child，visibility 固定为 private const。不得迁移 `MAX_EXPERIMENT_VARIANTS`。

---

## 调用方兼容策略

BE-001CN-03 不修改以下调用方文件:

- `src/runtime/run/replay_status.rs`
- `src/runtime/backtest/replay.rs`
- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`

这些调用方继续依赖 `use super::*`。迁移后的调用路径必须保持:

```text
runtime child callers
  -> src/runtime/mod.rs controlled query surface
  -> runtime.query_support
```

开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连该 child。

---

## 明确排除

- 不在 BE-001CN-02 创建 `src/runtime/query_support.rs`。
- 不在 BE-001CN-02 迁移 query DTO/helper。
- BE-001CN-03 不迁移 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`RunInProgressGuard` 或 `MAX_EXPERIMENT_VARIANTS`。
- BE-001CN-03 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 drained `include!("backtest.rs")`。
- BE-001CN-03 不回改 `backend.runtime.routes`、`runtime.report_ops`、`runtime.evidence_health`、`runtime.backtest`、`runtime.mutation.parameter_mutation`、`runtime.mutation.ai_proposal` 或 `runtime.mutation.shared_governance` closed child。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。

---

## 回退点

若 BE-001CN-03 实际抽离后任一门禁失败，回退范围只允许覆盖:

1. 删除 `src/runtime/query_support.rs`。
2. 移除 `src/runtime/mod.rs` 中的 `mod query_support;`。
3. 移除 `use query_support::{...};`。
4. 将允许迁移清单中的 9 个 item 原样放回 `src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/mutation.rs`。

不得借回退处理 response support、run guard、experiment limit、parent include 删除、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 验证要求

BE-001CN-02 本身不动 Rust 代码，但提交前仍需执行:

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

BE-001CN-03 实际抽离后也必须重复以上门禁，重点观察 field visibility 是否保持 warning-free 且不扩大 API。

---

## 下一步

下一步只允许进入:

```text
BE-001CN-03 runtime.query_support 实际抽离
```

BE-001CN-03 只能创建 planned child、迁移允许清单中的 9 个 item、按方案设置 `pub(super)` visibility，并保持调用方文件不变。不得处理 response support、run guard、experiment limit、drained parent include 删除或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CN-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. `src/runtime/query_support.rs` 尚未创建。
3. query DTO/helper 仍在 `src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/mutation.rs`。
4. BE-001CN-03 才允许创建 child 文件并迁移允许清单中的 9 个 item。
5. 迁移后 item visibility 固定为 `pub(super)`，query DTO fields 也优先使用 `pub(super)`。
6. 调用方文件仍通过 `use super::*`，不得改成横向直连 child。
7. response support、run guard、experiment limit、parent include 删除、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 和 release transition guard 均未迁移。

不得宣称 query support 已抽离、`backend.runtime` 已完成、parent support 已整体抽离、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `285-runtime.query_support抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001CN-03 的唯一 planned child、父级声明、plain import、DTO `pub(crate)` shell、field/helper `pub(super)` visibility、允许迁移清单和排除项。
3. 模块树不登记尚未创建的 `src/runtime/query_support.rs` 真实文件路径。
4. 治理门禁、Rust 等价测试、全量树覆盖和 `git diff --check` 均通过。
