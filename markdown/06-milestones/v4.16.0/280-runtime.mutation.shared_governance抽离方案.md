# v4.16.0 runtime.mutation.shared_governance 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CL-02
> 基准: `279-runtime.mutation.shared_governance单子叶等价基线.md`、`278-backend.runtime第三轮父叶残余判断.md`、`13-递归模块化全局根流程.md`
> 目标子叶: `runtime.mutation.shared_governance`
> 模块树坐标: `root.backend.runtime.runtime.mutation.shared_governance`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CL-02 `runtime.mutation.shared_governance` 抽离方案 | 单子叶抽离方案 |
| 规范矩阵 | 父子通信、helper visibility、禁止跳步、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.mutation.shared_governance` | 固定 planned child 与父级接入方式 |
| 模块树 | `runtime.mutation.shared_governance` | 白箱方案登记 |

---

## 当前真实结构

本批仍为 `no code movement`。实际 Rust 结构保持:

```text
src/runtime/mod.rs
src/runtime/mutation.rs
```

`src/runtime/mod.rs` 当前通过 parent include 承接 mutation shared helper:

```rust
include!("mutation.rs");
```

`src/runtime/mutation.rs` 仍持有 9 个 shared governance helper，以及暂不处理的 query DTO:

- `canonical_runtime_parameter_version`
- `validate_runtime_parameter_mutation_target`
- `runtime_mode_from_events`
- `status_contract_value`
- `mutation_event_contract`
- `build_runtime_parameter_mutation_event`
- `append_parameter_mutation_events_to_run`
- `runtime_parameter_mutation_governance`
- `governance_with_parameter_version`
- `OpsDailyQuery`
- `AuditWeeklyQuery`
- `ResearchMonthlyQuery`

---

## 目标 child 与父级声明

BE-001CL-03 的唯一 planned child 文件为:

```text
src/runtime/mutation/shared_governance.rs
```

BE-001CL-03 只能在 `src/runtime/mod.rs` 增加受控 child 声明:

```rust
#[path = "mutation/shared_governance.rs"]
mod mutation_shared_governance;
```

父级只能用 plain `use mutation_shared_governance::{...};` 把 caller-facing helper 拉回 `src/runtime/mod.rs` 的父级受控 surface，保持现有 child 调用方通过 `use super::*` 取名:

```rust
use mutation_shared_governance::{
    append_parameter_mutation_events_to_run,
    build_runtime_parameter_mutation_event,
    canonical_runtime_parameter_version,
    governance_with_parameter_version,
    mutation_event_contract,
    runtime_parameter_mutation_governance,
    validate_runtime_parameter_mutation_target,
};
```

`runtime_mode_from_events` 与 `status_contract_value` 迁入 child 后仍是 `pub(super)` helper，但只被同一 child 内部调用，不强制回填到父级 import surface，避免 unused import warning。不得在本子叶引入 `pub(crate) use` 对外扩大 runtime surface。新 child 内部函数 visibility 固定为 `pub(super)`，child 文件内允许 `use super::*;` 继承父级类型与 helper。

`include!("mutation.rs")` 保留，直到 `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery` 以及后续 query/guard/response support 残余另起父叶判断。

---

## 允许迁移清单

BE-001CL-03 仅允许迁移以下 9 个 helper 到 `src/runtime/mutation/shared_governance.rs`:

| helper | 迁移后 visibility | 迁移约束 |
| --- | --- | --- |
| `canonical_runtime_parameter_version` | `pub(super)` | 不得改变 canonical payload、digest prefix 或 error mapping |
| `validate_runtime_parameter_mutation_target` | `pub(super)` | 不得放宽 module key、node id、path 或 capability gate |
| `runtime_mode_from_events` | `pub(super)` | 不得改变 default `paper` |
| `status_contract_value` | `pub(super)` | 不得改变 status contract spelling |
| `mutation_event_contract` | `pub(super)` | 不得改变 event type 或 reason code |
| `build_runtime_parameter_mutation_event` | `pub(super)` | 不得改变 event id、payload fields、severity 或 envelope default |
| `append_parameter_mutation_events_to_run` | `pub(super)` | 不得改变 sequence、mode、governance envelope、persistence condition 或 lock owner |
| `runtime_parameter_mutation_governance` | `pub(super)` | 不得改变 capability/deployment/strategy/permission boundary mapping |
| `governance_with_parameter_version` | `pub(super)` | 不得改变除 parameter version 以外的 governance fields |

---

## 调用方兼容策略

BE-001CL-03 不修改调用方文件:

- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/ai_proposal/proposal_creation.rs`

这些调用方继续依赖父级 `use super::*`。迁移后调用路径必须保持:

```text
runtime.mutation.parameter_mutation / runtime.mutation.ai_proposal
  -> src/runtime/mod.rs controlled helper surface
  -> runtime.mutation.shared_governance
```

开发者未明确进入发布版本过渡前，不得让 parameter mutation child、AI proposal child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner、`AppState` 或任何横向 sibling 直接连接该 child。

---

## 明确排除

- 不处理 query DTO: `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`。
- 不处理 run guard: `RunInProgressGuard`。
- 不处理 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、replay option、experiment limit 或 response support。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。
- 不回改 `runtime.mutation.parameter_mutation`、`runtime.mutation.ai_proposal`、`runtime.report_ops`、`runtime.evidence_health`、`backend.runtime.routes.mutation` 或其他 closed child。
- 不启动发布版本过渡，也不提出横向连接优化。

---

## 验证要求

BE-001CL-02 本身不动 Rust 代码，但提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001CL-03 实际抽离后也必须重复以上门禁，重点观察 `api_mutation` 与 `api_ai_proposal`。

---

## 回退点

若 BE-001CL-03 实际抽离后任一门禁失败，回退范围只允许覆盖:

1. 删除 `src/runtime/mutation/shared_governance.rs`。
2. 移除 `src/runtime/mod.rs` 中 `#[path = "mutation/shared_governance.rs"] mod mutation_shared_governance;`。
3. 移除 `use mutation_shared_governance::{...};`。
4. 把 9 个 helper 原样放回 `src/runtime/mutation.rs`。

不得借回退处理 query DTO、run guard、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

---

## 下一步

下一步只允许进入:

```text
BE-001CL-03 runtime.mutation.shared_governance 实际抽离
```

BE-001CL-03 只能创建 planned child 文件、迁移 9 个 helper、补父级受控 child 声明和 plain `use`。不得宣称 `backend.runtime` 已完成，不得处理 query/guard/response support。

---

## 幻觉检查点

AI 声称 BE-001CL-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. `src/runtime/mutation/shared_governance.rs` 尚未创建。
3. 9 个 shared governance helper 仍在 `src/runtime/mutation.rs`。
4. BE-001CL-03 才允许创建 child 文件并迁移 helper。
5. 父级接入方式固定为 `src/runtime/mod.rs` 的 `#[path = "mutation/shared_governance.rs"] mod mutation_shared_governance;` 与 plain `use mutation_shared_governance::{...};`。
6. child 内 helper visibility 固定为 `pub(super)`，调用方继续通过 `use super::*`。
7. `include!("mutation.rs")` 暂时保留，因为 `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery` 仍未处理。
8. query DTO、run guard、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 和 release transition guard 均未迁移。

不得宣称 helper 已抽离、`backend.runtime` 已完成、parent support 已整体抽离、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `280-runtime.mutation.shared_governance抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001CL-03 的唯一 planned child、父级声明、plain `use`、`pub(super)` visibility、允许迁移清单和排除项。
3. 模块树不登记尚未创建的 Rust child 真实文件路径。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
