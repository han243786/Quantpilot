# v4.16.0 runtime.mutation.ai_proposal.event_lifecycle 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BD-03  
> 基线: `195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md`、`196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.ai_proposal.event_lifecycle` 第一轮实际抽离完成。event contract、runtime event builder、lifecycle entry 与 proposal transition persistence helper 已迁入 child 文件；record_query、approval_review、approval_persistence、sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001BD-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BD-03 event_lifecycle 实际抽离 | 已落地 |
| 规范矩阵 | 父子通信、`pub(super)` helper visibility、event contract private、非目标边界 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.event_lifecycle` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.event_lifecycle` | 白箱抽离完成 |

---

## 实际文件变更

新增 child 文件:

```text
src/runtime/mutation/ai_proposal/event_lifecycle.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增 path-attributed child:

```rust
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;
```

父级通过受控 helper import 继续调用 event / lifecycle helper:

```rust
use event_lifecycle::{
    ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
    persist_runtime_ai_proposal_transition,
};
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
```

对外给父级的 helper 保持受控可见性:

```rust
pub(super) fn build_runtime_ai_proposal_event(...)
pub(super) fn ai_proposal_lifecycle_entry(...)
pub(super) async fn persist_runtime_ai_proposal_transition(...)
```

---

## 已迁移 helper

| helper | BE-001BD-03 visibility | 调整原因 |
| --- | --- | --- |
| `ai_proposal_event_contract` | private | 只服务 child 内部 event builder / lifecycle entry |
| `build_runtime_ai_proposal_event` | `pub(super)` | 父级 create / approval transition 需要生成 runtime event |
| `ai_proposal_lifecycle_entry` | `pub(super)` | 父级 create / approval transition 需要写 lifecycle entry |
| `persist_runtime_ai_proposal_transition` | `pub(super) async` | 父级 create / approval transition 需要持久化 proposal record |

BE-001BD-03 未移动单测。现有 API 回归继续通过 public handler 证明行为等价。

---

## 等价保持

已保持:

- `RuntimeAiProposalStatus::Submitted` 与 `Draft` 继续映射到 `AIProposalCreated` / `AI_PROPOSAL_CREATED`
- `StaticCheckPassed` 继续映射到 `AIProposalStaticCheckPassed` / `AI_PROPOSAL_STATIC_CHECK_PASSED`
- `Approved` 继续映射到 `AIProposalApproved` / `AI_PROPOSAL_APPROVED`
- `Denied`、`StaticCheckFailed`、`Expired` 的 event type / reason code 映射不变
- event id 继续使用 `event_{ai_proposal_id}_{reason_code}_{event_time_ms}`
- `source_id` 继续取 `record.target.module_key`
- `node_id` 继续取 `record.target.node_id`
- `Denied` 与 `StaticCheckFailed` severity 继续为 `Warn`，其他状态继续为 `Info`
- event payload 字段保持原样
- `RuntimeEventEnvelope::default()` 保持原样
- lifecycle entry 继续使用 event id、sequence_no、occurred_at_ms 与 reason_code
- proposal transition persistence 继续先写 `persist_runtime_ai_proposal_record`，再写 `state.ai_proposals` 的 `auth::scoped_key`

---

## 未迁移边界
BE-001BD-03 未迁移或修改:

- `create_runtime_ai_proposal`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `load_runtime_ai_proposal_for_user`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得把本批解释为 record_query、approval_review、approval_persistence、sandbox_trigger 或 status_transition 已拆分。

---

## 等价验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

已先行验证:

- `cargo fmt --check`
- `cargo check -p quantpilot`

---

## 回退点
若 BE-001BD-03 后续验证失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/event_lifecycle.rs`
- `src/runtime/mutation/ai_proposal.rs` 中的 `#[path = "ai_proposal/event_lifecycle.rs"] mod event_lifecycle;`
- `src/runtime/mutation/ai_proposal.rs` 中的 `use event_lifecycle::{...};`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 event / lifecycle helper 迁移造成的删除

不得回退 `runtime.mutation.ai_proposal.static_check` 或 `runtime.mutation.ai_proposal.source_governance_identity` 已完成抽离，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BD-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.event_lifecycle` 第一轮实际抽离。不得宣称本叶已完成 closeout、record query / approval review / approval persistence / sandbox trigger 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或 Rust backend 重构已经完成。

---

## 验收标准

1. `197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/event_lifecycle.rs` 存在，并承载 event contract、runtime event builder、lifecycle entry 与 proposal transition persistence helper。
3. 父级只通过 path-attributed child 和受控 `pub(super)` helper import 调用 child。
4. record_query、approval_review、approval_persistence、sandbox_trigger 和 status_transition 未被宣称完成。
5. 验证通过后，后续只能进入 BE-001BD-04 单叶 closeout，判断本 child 是否值得继续细拆。
