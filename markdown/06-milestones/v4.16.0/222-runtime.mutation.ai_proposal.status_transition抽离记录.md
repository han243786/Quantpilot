# v4.16.0 runtime.mutation.ai_proposal.status_transition 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BN-03  
> 基线: `220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md`、`221-runtime.mutation.ai_proposal.status_transition抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: `runtime.mutation.ai_proposal.status_transition` 第一轮实际抽离完成。`ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status` 已迁入 child 文件；父级只保留 path-attributed child、受控 helper import 和既有调用面。proposal create orchestration、AppState、schema owner、frontend caller、route facade、runtime persistence owner 与 release transition guard 均未迁移。下一步只能进入 BE-001BN-04 单叶 closeout。  
> 代码动作: code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BN-03 status_transition 实际抽离 | 物理抽离 |
| 规范矩阵 | 父级受控 helper import、child private transition guard、closed child 不横连 | 约束执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.status_transition` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.status_transition` | 白箱抽离完成 |

---

## 文件变更

新增:

```text
src/runtime/mutation/ai_proposal/status_transition.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增:

```rust
#[path = "ai_proposal/status_transition.rs"]
mod status_transition;

use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
```

child 固定:

```rust
use super::*;
```

父级删除了原内联的 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status`。

---

## 实际迁移清单

已从父级迁入 child:

- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`

迁移后 `ai_proposal_approved_status` 保持 `pub(super) fn`，`update_ai_proposal_status` 保持 `pub(super) async fn`，`is_valid_ai_proposal_transition` 保持 child private `fn`。`approval_review` 仍经父级 `use super::*` 访问 `ai_proposal_approved_status` 与 `update_ai_proposal_status`，没有横向 import `status_transition` sibling。

---

## 行为等价说明

approved projection 行为保持不变:

1. `ai_proposal_approved_status` 继续返回 `RuntimeAiProposalStatus::Approved`。
2. `approval_review::approve_ai_proposal` 继续经父级受控 helper 名称调用 approved projection。

状态迁移矩阵保持不变:

1. `is_valid_ai_proposal_transition` 继续只允许 `(Submitted, StaticCheckPassed | StaticCheckFailed)`。
2. `is_valid_ai_proposal_transition` 继续只允许 `(StaticCheckPassed, Approved | Denied | Expired)`。
3. 该 guard 仍只被 `update_ai_proposal_status` 内部调用，保持 child private。

状态写入副作用保持不变:

1. `update_ai_proposal_status` 继续通过 `state.ai_proposals.write().await` 获取写锁。
2. proposal lookup 继续使用 `auth::scoped_key(user_id, proposal_id)`。
3. missing record 继续 no-op。
4. 非法转换继续调用 `safe_eprintln!("[ai_proposal] 非法状态转换: {:?} → {:?} (proposal_id={})", ...)` 并 return。
5. 合法转换继续执行 `record.status = status`。
6. 合法转换继续执行 `record.updated_at_ms = current_time_ms()`。

---

## 调用面保持
| 调用点 | 等价结果 |
| --- | --- |
| `approval_review::approve_ai_proposal` | 继续经父级 `use super::*` 调用 `ai_proposal_approved_status` 与 `update_ai_proposal_status` |
| `approval_review::reject_ai_proposal` | 继续经父级 `use super::*` 调用 `update_ai_proposal_status` |
| `create_runtime_ai_proposal` | 未迁移，proposal create orchestration 保持在父级 |
| route facade | `src/backend/runtime/routes/mutation.rs` 不变 |

---

## 非目标边界

BE-001BN-03 未迁移或修改:

- proposal create orchestration
- `create_runtime_ai_proposal`
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

已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence` 与 `sandbox_trigger` 未回收、未重拆。

---

## 验证记录

实际抽离批次已运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

治理门禁将在本批治理登记完成后运行:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BN-04 runtime.mutation.ai_proposal.status_transition 单叶 closeout
```

该 closeout 只能判断本叶是否停止细分，不得继续迁移 proposal create orchestration、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BN-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.status_transition` 第一轮实际抽离，尚未完成单叶 closeout。不得宣称 proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变，也不得宣称 Rust backend 重构完成。

---

## 验收标准

1. `src/runtime/mutation/ai_proposal/status_transition.rs` 存在，并承接 approved projection、状态迁移矩阵和 scoped in-memory status side effect。
2. 父级 `src/runtime/mutation/ai_proposal.rs` 只保留 path-attributed child、受控 helper import 和原有调用面。
3. `approval_review` 仍经父级受控 helper 名称访问，不横向 import sibling。
4. `222-runtime.mutation.ai_proposal.status_transition抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
5. 下一步固定为 BE-001BN-04 单叶 closeout。
