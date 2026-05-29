# v4.16.0 runtime.mutation.ai_proposal.event_lifecycle 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BD-02  
> 基线: `195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案已建立。当前仍为 `no code movement`；只固定 BE-001BD-03 的目标文件、父级 child 声明、helper import、`pub(super)` visibility、迁移清单、非目标、回退点和验证门禁。下一步只能进入 BE-001BD-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BD-02 event_lifecycle 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、`pub(super)` helper visibility、event contract private、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.event_lifecycle` | 子叶抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.event_lifecycle` | 白箱方案 |

---

## 目标文件与父级声明

BE-001BD-03 只允许创建一个目标文件:

```text
src/runtime/mutation/ai_proposal/event_lifecycle.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 只允许新增一个 child 声明:

```rust
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;
```

父级只允许新增受控 helper import:

```rust
use event_lifecycle::{
    ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
    persist_runtime_ai_proposal_transition,
};
```

child 文件必须继续只通过父级白箱输入取依赖:

```rust
use super::*;
```

需要被父级调用的 helper 只允许使用 `pub(super)`:

```rust
pub(super) fn build_runtime_ai_proposal_event(...)
pub(super) fn ai_proposal_lifecycle_entry(...)
pub(super) async fn persist_runtime_ai_proposal_transition(...)
```

`ai_proposal_event_contract` 只服务 child 内部 event builder / lifecycle entry，必须保持 private。

---

## BE-001BD-03 允许迁移清单

只允许迁移以下 event / lifecycle helper:

- `ai_proposal_event_contract`
- `build_runtime_ai_proposal_event`
- `ai_proposal_lifecycle_entry`
- `persist_runtime_ai_proposal_transition`

BE-001BD-03 不迁移单测。现有 API 回归继续通过 public handler 证明等价:

- `create_runtime_ai_proposal`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`

---

## visibility 规则

| helper | BE-001BD-03 visibility | 原因 |
| --- | --- | --- |
| `ai_proposal_event_contract` | private | 只服务 child 内部 event builder / lifecycle entry |
| `build_runtime_ai_proposal_event` | `pub(super)` | 父级 create / approval transition 需要生成 runtime event |
| `ai_proposal_lifecycle_entry` | `pub(super)` | 父级 create / approval transition 需要写 lifecycle entry |
| `persist_runtime_ai_proposal_transition` | `pub(super)` | 父级 create / approval transition 需要持久化 proposal record |

---

## route / handler 等价约束

BE-001BD-03 迁移后，以下 public handler 的签名、调用方和行为必须保持不变:

- `create_runtime_ai_proposal`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`

`src/backend/runtime/routes/mutation.rs`、`src/runtime/mod.rs` 的 route-facing re-export 不得改变。

---

## 非目标

BE-001BD-03 不得迁移或修改:

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

## 回退点

若 BE-001BD-03 编译或等价检查失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/event_lifecycle.rs`
- `#[path = "ai_proposal/event_lifecycle.rs"] mod event_lifecycle;`
- `use event_lifecycle::{...};`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 event_lifecycle 迁移造成的删除

不得回改 `runtime.mutation.ai_proposal.static_check` 或 `runtime.mutation.ai_proposal.source_governance_identity` 已完成抽离，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 验证计划

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

---

## 幻觉检查点

AI 声称 BE-001BD-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.event_lifecycle` 抽离方案，仍为 `no code movement`；目标文件尚未创建，helper 尚未迁移。下一步只能进入 BE-001BD-03 实际抽离。不得宣称 event_lifecycle 已抽离、record query / approval review 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 child 声明、helper import、`pub(super)` visibility、允许迁移清单和回退点已冻结。
3. 治理门禁能发现本方案、`no code movement`、下一批 BE-001BD-03、目标文件、关键 helper、非目标边界和验证门禁缺失。
4. 本批验证通过后，后续才能进入 BE-001BD-03 实际抽离。
