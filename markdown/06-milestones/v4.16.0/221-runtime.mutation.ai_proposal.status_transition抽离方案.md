# v4.16.0 runtime.mutation.ai_proposal.status_transition 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BN-02  
> 基线: `219-runtime.mutation.ai_proposal第七轮父叶残余判断.md`、`220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: 固定 BE-001BN-03 的实际抽离方案。下一步只允许创建 `src/runtime/mutation/ai_proposal/status_transition.rs`，迁移 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status`；父级只保留 path-attributed child、受控 helper import 和既有调用面。当前 `no code movement`。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BN-02 status_transition 抽离方案 | 方案固化 |
| 规范矩阵 | 父子通信、状态机 guard、受控 helper import、closed child 不横连 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.status_transition` | 抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.status_transition` | 计划物理抽离 |

---

## 目标文件与父级声明

BE-001BN-03 允许创建:

```text
src/runtime/mutation/ai_proposal/status_transition.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 固定新增:

```rust
#[path = "ai_proposal/status_transition.rs"]
mod status_transition;
```

父级固定通过受控 helper import 连接 child:

```rust
use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
```

child 固定:

```rust
use super::*;
```

`is_valid_ai_proposal_transition` 不从 child 暴露给父级，保持 child-private；`ai_proposal_approved_status` 与 `update_ai_proposal_status` 只用 `pub(super)` 暴露给父级。

---

## 允许迁移清单

BE-001BN-03 只允许迁移以下内容:

| 内容 | 目标形态 | 可见性 | 说明 |
| --- | --- | --- | --- |
| `ai_proposal_approved_status` | 迁入 `status_transition.rs` | `pub(super) fn` | 供 approval_review 经父级受控 helper 名称访问 |
| `is_valid_ai_proposal_transition` | 迁入 `status_transition.rs` | child private `fn` | 仅被 `update_ai_proposal_status` 调用 |
| `update_ai_proposal_status` | 迁入 `status_transition.rs` | `pub(super) async fn` | 供 approval_review 经父级受控 helper 名称访问 |

父级不得迁移 `create_runtime_ai_proposal` 主体，不得迁移 proposal create orchestration，也不得改变 approval_review handler 文件。

---

## 等价要求

BE-001BN-03 必须保持以下行为等价:

1. `ai_proposal_approved_status` 继续返回 `RuntimeAiProposalStatus::Approved`。
2. `is_valid_ai_proposal_transition` 继续只允许 `(Submitted, StaticCheckPassed | StaticCheckFailed)` 与 `(StaticCheckPassed, Approved | Denied | Expired)`。
3. `update_ai_proposal_status` 继续通过 `state.ai_proposals.write().await` 获取写锁。
4. `update_ai_proposal_status` 继续通过 `auth::scoped_key(user_id, proposal_id)` 定位记录。
5. missing record 继续 no-op，不返回错误。
6. 非法转换继续调用 `safe_eprintln!("[ai_proposal] 非法状态转换: {:?} → {:?} (proposal_id={})", ...)` 并 return。
7. 合法转换继续执行 `record.status = status`。
8. 合法转换继续执行 `record.updated_at_ms = current_time_ms()`。
9. `approval_review::approve_ai_proposal` 与 `reject_ai_proposal` 继续经父级 `use super::*` 调用状态 helper，不得横向 import `status_transition` sibling。

---

## 父子通信规则

`runtime.mutation.ai_proposal.status_transition` 只能由父级 `runtime.mutation.ai_proposal` 连接:

- `approval_review` 继续通过 `use super::*` 调用 `ai_proposal_approved_status` 与 `update_ai_proposal_status`。
- `status_transition` 通过 `use super::*` 访问 `AppState`、`auth::UserId`、`RuntimeAiProposalStatus`、`current_time_ms` 与 `safe_eprintln!`。
- `status_transition` 不得横向 import `approval_review`、`sandbox_trigger`、`approval_persistence`、`record_query`、`event_lifecycle`、`static_check` 或 `source_governance_identity` sibling。
- 发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 非目标边界

BE-001BN-03 不得迁移或修改:

- `create_runtime_ai_proposal`
- proposal create orchestration
- `approval_review`
- `sandbox_trigger`
- `approval_persistence`
- `record_query`
- `event_lifecycle`
- `static_check`
- `source_governance_identity`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回改已 closeout 子叶。

---

## 回退点

如果 BE-001BN-03 编译或测试失败，回退方式固定为:

1. 删除 `src/runtime/mutation/ai_proposal/status_transition.rs`。
2. 将 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status` 原样放回 `src/runtime/mutation/ai_proposal.rs`。
3. 删除父级 `mod status_transition` 与 `use status_transition::{...};`。
4. 重新运行同一组验证门禁。

---

## 验证计划

BE-001BN-03 实际抽离必须运行:

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

本 BE-001BN-02 方案批次仍为 `no code movement`，提交前只需要治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步固定为:

```text
BE-001BN-03 runtime.mutation.ai_proposal.status_transition 实际抽离
```

---

## 幻觉检查点

AI 声称 BE-001BN-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.status_transition` 抽离方案，尚未创建 `status_transition.rs`，也尚未迁移 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 或 `update_ai_proposal_status`。不得宣称 proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `221-runtime.mutation.ai_proposal.status_transition抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001BN-03 的目标文件、父级声明、helper import、允许迁移清单、非目标和回退点已固定。
3. 本批不产生 Rust 代码变更，不创建 `status_transition.rs`。
4. 下一步固定为 BE-001BN-03 实际抽离。
