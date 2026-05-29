# v4.16.0 runtime.mutation.ai_proposal.source_governance_identity 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BB-03  
> 基线: `190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md`、`191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`tests/api_ai_proposal.rs`  
> 判定: `runtime.mutation.ai_proposal.source_governance_identity` 第一轮实际抽离完成。source context、governance projection 与 proposal record identity helper 已迁入 child 文件；event lifecycle、record query、approval review、approval persistence、sandbox trigger、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001BB-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BB-03 source_governance_identity 实际抽离 | 已落地 |
| 规范矩阵 | 父子通信、`pub(super)` struct / field visibility、非目标边界 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.source_governance_identity` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.source_governance_identity` | 白箱抽离完成 |

---

## 实际文件变更

新增 child 文件:

```text
src/runtime/mutation/ai_proposal/source_governance_identity.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增 path-attributed child:

```rust
#[path = "ai_proposal/source_governance_identity.rs"]
mod source_governance_identity;
```

父级通过受控 helper import 继续调用 source / governance / identity helper:

```rust
use source_governance_identity::{
    load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
    runtime_ai_proposal_record_id,
};
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
```

---

## 已迁移 helper

| helper | BE-001BB-03 visibility | 调整原因 |
| --- | --- | --- |
| `RuntimeAiProposalSourceContext` | `pub(super)` | 父级 create flow 需要接收返回类型并读取字段 |
| `RuntimeAiProposalSourceContext.graph_id` | `pub(super)` | 父级 create flow 写入 proposal source evidence |
| `RuntimeAiProposalSourceContext.event_count` | `pub(super)` | 父级 create flow 写入 source evidence 并驱动 static check |
| `RuntimeAiProposalSourceContext.current_sequence_no` | `pub(super)` | 父级 event lifecycle 继续续写 sequence |
| `RuntimeAiProposalSourceContext.governance` | `pub(super)` | 父级 proposal governance projection 继续读取 |
| `load_runtime_ai_proposal_source_context` | `pub(super)` | 只服务父级 `create_runtime_ai_proposal` |
| `runtime_ai_proposal_governance` | `pub(super)` | 只服务父级 `create_runtime_ai_proposal` |
| `runtime_ai_proposal_record_id` | `pub(super)` | 只服务父级 `create_runtime_ai_proposal` |

BE-001BB-03 未移动单测。现有 API 回归继续通过 public handler 证明行为等价。

---

## 等价保持

已保持:

- `RuntimeEvidenceSourceKind::Run` 继续使用 `load_run_record_from_state`
- `RuntimeEvidenceSourceKind::Backtest` 继续使用 `load_backtest_record_from_state`
- `current_sequence_no` 继续优先取最后一个 event 的 `envelope.sequence_no`
- 无事件时继续 fallback 到 `events.len() as u64`
- `event_count` 继续保持 `events.len()`
- proposal governance 字段映射不变
- proposal id 继续使用 `ai_proposal_{created_at_ms}_{digest[..12]}`
- `create_runtime_ai_proposal` handler 事务主体不迁移
- route facade `src/backend/runtime/routes/mutation.rs` 不改变

---

## 未迁移边界

BE-001BB-03 未迁移或修改:

- `create_runtime_ai_proposal`
- `ai_proposal_static_check_result`
- `validate_ai_model_identity`
- `validate_hash_identity`
- `build_runtime_ai_proposal_event`
- `ai_proposal_lifecycle_entry`
- `persist_runtime_ai_proposal_transition`
- `load_runtime_ai_proposal_for_user`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
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

不得把本批解释为 event_lifecycle、record_query、approval_review、approval_persistence、sandbox_trigger 或 status_transition 已拆分。

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

若 BE-001BB-03 后续验证失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
- `src/runtime/mutation/ai_proposal.rs` 中的 `#[path = "ai_proposal/source_governance_identity.rs"] mod source_governance_identity;`
- `src/runtime/mutation/ai_proposal.rs` 中的 `use source_governance_identity::{...};`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 source context / governance / record identity helper 迁移造成的删除

不得回退 `runtime.mutation.ai_proposal.static_check` 已完成抽离，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BB-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.source_governance_identity` 第一轮实际抽离。不得宣称本叶已完成 closeout、event lifecycle / approval review / record query 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或 Rust backend 重构已经完成。

---

## 验收标准

1. `192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/source_governance_identity.rs` 存在，并承载 source context、governance projection 与 proposal record identity helper。
3. 父级只通过 path-attributed child 和受控 `pub(super)` helper import 调用 child。
4. event_lifecycle、record_query、approval_review、approval_persistence、sandbox_trigger 和 status_transition 未被宣称完成。
5. 验证通过后，后续只能进入 BE-001BB-04 单叶 closeout，判断本 child 是否值得继续细拆。
