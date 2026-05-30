# v4.16.0 runtime.mutation.shared_governance 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CL-03
> 基准: `280-runtime.mutation.shared_governance抽离方案.md`、`279-runtime.mutation.shared_governance单子叶等价基线.md`
> 目标子叶: `runtime.mutation.shared_governance`
> 模块树坐标: `root.backend.runtime.runtime.mutation.shared_governance`
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CL-03 `runtime.mutation.shared_governance` 实际抽离 | 单子叶抽离记录 |
| 规范矩阵 | 父子通信、helper visibility、warning-free parent surface、release transition guard | 落地 |
| 引导矩阵 | `root.backend.runtime.runtime.mutation.shared_governance` | child 文件落地 |
| 模块树 | `runtime.mutation.shared_governance` | 白箱真实文件登记 |

---

## 代码变更

本批创建:

```text
src/runtime/mutation/shared_governance.rs
```

并更新:

```text
src/runtime/mod.rs
src/runtime/mutation.rs
```

`src/runtime/mod.rs` 新增:

```rust
#[path = "mutation/shared_governance.rs"]
mod mutation_shared_governance;
```

父级以 plain `use mutation_shared_governance::{...};` 回填 caller-facing helper surface。为保持 warning-free，`runtime_mode_from_events` 与 `status_contract_value` 只留在 child 内部调用，不回填到父级 import。

`src/runtime/mutation.rs` 只保留 report query DTO:

- `OpsDailyQuery`
- `AuditWeeklyQuery`
- `ResearchMonthlyQuery`

`include!("mutation.rs")` 继续保留，后续 query/guard/response support 由下一轮父叶残余判断处理。

---

## 已迁移 helper

以下 9 个 helper 已迁入 `src/runtime/mutation/shared_governance.rs`，visibility 均为 `pub(super)`:

| helper | 父级 import | 迁移结果 |
| --- | --- | --- |
| `canonical_runtime_parameter_version` | 是 | caller-facing helper，供 proposal create path 使用 |
| `validate_runtime_parameter_mutation_target` | 是 | caller-facing helper，供 proposal create path 使用 |
| `runtime_mode_from_events` | 否 | child-internal helper，供 event append 使用 |
| `status_contract_value` | 否 | child-internal helper，供 event builder 使用 |
| `mutation_event_contract` | 是 | caller-facing helper，供 event builder 与 transition persistence 使用 |
| `build_runtime_parameter_mutation_event` | 是 | caller-facing helper，供 proposal / activation / rollback 使用 |
| `append_parameter_mutation_events_to_run` | 是 | caller-facing helper，供 proposal / activation / rollback 使用 |
| `runtime_parameter_mutation_governance` | 是 | caller-facing helper，供 proposal / rollback 使用 |
| `governance_with_parameter_version` | 是 | caller-facing helper，供 proposal / activation / rollback 使用 |

---

## 调用方等价

以下调用方文件未改动，继续通过 `use super::*` 访问父级受控 surface:

- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`
- `src/runtime/mutation/ai_proposal/proposal_creation.rs`

父子通信路径保持:

```text
runtime.mutation.parameter_mutation / runtime.mutation.ai_proposal
  -> src/runtime/mod.rs controlled helper surface
  -> runtime.mutation.shared_governance
```

开发者未明确进入发布版本过渡前，不得让 sibling child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连该 child。

---

## 明确未迁移

- query DTO: `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`。
- run guard: `RunInProgressGuard`。
- response support: `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`。
- replay option、experiment limit、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 与 release transition guard。
- `runtime.mutation.parameter_mutation`、`runtime.mutation.ai_proposal`、`runtime.report_ops`、`runtime.evidence_health` 和 `backend.runtime.routes.mutation` closed child。

---

## 验证要求

本批提交前必须执行:

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

---

## 下一步

下一步只能进入:

```text
BE-001CL-04 runtime.mutation.shared_governance 单叶 closeout
```

BE-001CL-04 只判断本叶是否还值得继续拆分，不得直接回到 `backend.runtime` 父叶，不得处理 query/guard/response support。

---

## 幻觉检查点

AI 声称 BE-001CL-03 完成时，必须说明:

1. `src/runtime/mutation/shared_governance.rs` 已创建。
2. 9 个 shared governance helper 已从 `src/runtime/mutation.rs` 迁入 child。
3. helper visibility 为 `pub(super)`。
4. `runtime_mode_from_events` 与 `status_contract_value` 是 child-internal helper，没有回填到父级 import，以保持 warning-free。
5. `src/runtime/mod.rs` 只新增 child 声明与 caller-facing plain import。
6. `src/runtime/mutation.rs` 仍保留 `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`，`include!("mutation.rs")` 仍保留。
7. 调用方文件未改动，仍通过 `use super::*`。
8. query DTO、run guard、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 和 release transition guard 均未迁移。
9. 下一步只能进入 BE-001CL-04 单叶 closeout。

不得宣称 `backend.runtime` 已完成、parent support 已整体抽离、query/guard/response support 已处理、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `src/runtime/mutation/shared_governance.rs` 进入模块树和全量树。
2. `281-runtime.mutation.shared_governance抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 9 个 helper 只从父级抽离到 child，不改变事件、governance、target validation、canonical version 或 persistence 条件。
4. Rust 等价测试、治理门禁、全量树覆盖和 `git diff --check` 均通过。
