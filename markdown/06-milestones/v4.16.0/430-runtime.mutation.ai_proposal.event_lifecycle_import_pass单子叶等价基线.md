# v4.16.0 runtime.mutation.ai_proposal.event_lifecycle_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EV-01
> 父叶判定: `429-runtime.mutation.ai_proposal_import_pass第五轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.event_lifecycle_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.event_lifecycle_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EV-02 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EV-01 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | explicit import pass / event contract freeze / no release transition | 冻结事件 lifecycle 输入面 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.event_lifecycle_import_pass` | event lifecycle 白箱 |
| 模块树 | `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | 新基线 |

---

## 基线结论

本批只冻结 `src/runtime/mutation/ai_proposal/event_lifecycle.rs` 的当前等价边界，不改 Rust 代码。

```text
runtime.mutation.ai_proposal.event_lifecycle_import_pass baseline_frozen
runtime.mutation.ai_proposal.event_lifecycle_import_pass no_code_movement
src/runtime/mutation/ai_proposal/event_lifecycle.rs
current_parent_import_bridge: use super::*
next_step: BE-001EV-02 extraction plan
```

`event_lifecycle.rs` 是 ai proposal write path 的事件白箱，负责从 proposal record 与目标状态生成前端 runtime event、lifecycle entry，并在状态转换时保持 disk record 与 in-memory index 同步。它不是 route facade，也不是 approval persistence owner。

---

## 白箱节点

| 项 | 当前边界 |
| --- | --- |
| 输入 | `RuntimeAiProposalRecord`、`RuntimeAiProposalStatus`、event timestamp、lifecycle sequence、message、`AppState`、`auth::UserId` |
| 输出 | `FrontendRuntimeEvent`、`RuntimeAiProposalLifecycleEntry`、`Result<(), (StatusCode, String)>` |
| 处理者 | `ai_proposal_event_contract`、`build_runtime_ai_proposal_event`、`ai_proposal_lifecycle_entry`、`persist_runtime_ai_proposal_transition` |
| 调用方 | `proposal_creation.rs`、`approval_review.rs`、ai proposal parent facade |
| 禁止事项 | 不改 event type、不改 reason code、不改 payload schema、不改 severity/summary、不改持久化顺序、不新增 sibling 横向连接 |

---

## 当前 public / 可见入口

本子叶对父模块暴露 3 个 `pub(super)` helper:

```text
build_runtime_ai_proposal_event
ai_proposal_lifecycle_entry
persist_runtime_ai_proposal_transition
```

文件内私有 helper:

```text
ai_proposal_event_contract
```

---

## 当前隐式输入面

当前文件顶部仍为:

```rust
use super::*;
```

BE-001EV-03 预期只把该 parent wildcard import 收敛为显式输入面。预期输入面包括:

```rust
use crate::{
    auth, io_error, persist_runtime_ai_proposal_record, AppState, FrontendRuntimeEvent,
    RuntimeAiProposalLifecycleEntry, RuntimeAiProposalRecord, RuntimeAiProposalStatus,
    RuntimeEventEnvelope,
};
use axum::http::StatusCode;
use serde_json::json;
```

该预期仅作为输入面基线，真正代码改写必须等 BE-001EV-03。

---

## 等价边界

### Event contract

必须保持状态到 event type / reason code 的映射:

```text
Submitted -> AIProposalCreated / AI_PROPOSAL_CREATED
Draft -> AIProposalCreated / AI_PROPOSAL_CREATED
Denied -> AIProposalDenied / AI_PROPOSAL_DENIED
StaticCheckPassed -> AIProposalStaticCheckPassed / AI_PROPOSAL_STATIC_CHECK_PASSED
StaticCheckFailed -> AIProposalStaticCheckFailed / AI_PROPOSAL_STATIC_CHECK_FAILED
Expired -> AIProposalDenied / AI_PROPOSAL_EXPIRED
Approved -> AIProposalApproved / AI_PROPOSAL_APPROVED
```

不得改变 `ai_proposal_event_contract` 的状态映射和返回顺序。

### Event payload

必须保持 `build_runtime_ai_proposal_event` 的 payload 字段:

```text
ai_proposal_id
status
reason_code
source_kind
source_id
graph_id
source_evidence
target
old_parameter_version
proposed_parameter_version
denial_reason
static_check
model
prompt_hash
evidence_hash
actor
reason
governance
config_domain_binding
```

必须保持 `event_id` 格式:

```text
event_{ai_proposal_id}_{reason_code}_{event_time_ms}
```

必须保持 severity:

```text
Denied -> Warn
StaticCheckFailed -> Warn
other status -> Info
```

### Lifecycle entry

必须保持 `ai_proposal_lifecycle_entry` 的字段投影:

```text
status passthrough
event_id from event
sequence_no passthrough
occurred_at_ms from event.event_time_ms
reason_code from ai_proposal_event_contract
message passthrough
```

### Transition persistence

必须保持 `persist_runtime_ai_proposal_transition` 的副作用顺序:

```text
persist_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), record)
map error through io_error
state.ai_proposals.write().await.insert(auth::scoped_key(user_id, &record.ai_proposal_id), record.clone())
Ok(())
```

不得改变 disk-first 顺序、scoped key 规则或 record clone 语义。

---

## 不变量

```text
no_event_contract_rewrite
no_event_payload_rewrite
no_event_id_format_rewrite
no_event_severity_rewrite
no_lifecycle_sequence_rewrite
no_persistence_order_rewrite
no_status_semantics_rewrite
no_visibility_rewrite
no_sibling_owner_migration
old_three_leaf_pause_target_cancelled
```

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不删除 `use super::*`。
3. 不改函数体、测试、可见性、event type 或 reason code。
4. 不处理其他 ai proposal child import residual。
5. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
6. 不处理 `src/runtime/mod.rs` root parent bridge。
7. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
8. 不新增 sibling 横向连接。
9. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001EV-01 完成时，必须说明:

1. 本批只是 `no code movement` 单子叶等价基线。
2. `event_lifecycle.rs` 仍未实际删除 `use super::*`。
3. 下一步只能进入 BE-001EV-02 `runtime.mutation.ai_proposal.event_lifecycle_import_pass` 抽离方案。
4. 不得宣称 event_lifecycle import、ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `430-runtime.mutation.ai_proposal.event_lifecycle_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `event_lifecycle.rs` 白箱输入、输出、处理者、调用方和禁止事项已冻结。
3. 下一步固定为 BE-001EV-02 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
