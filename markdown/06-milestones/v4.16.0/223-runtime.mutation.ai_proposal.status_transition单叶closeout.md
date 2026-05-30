# v4.16.0 runtime.mutation.ai_proposal.status_transition 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BN-04  
> 基线: `220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md`、`221-runtime.mutation.ai_proposal.status_transition抽离方案.md`、`222-runtime.mutation.ai_proposal.status_transition抽离记录.md`、`src/runtime/mutation/ai_proposal/status_transition.rs`  
> 判定: `runtime.mutation.ai_proposal.status_transition` 单叶 closeout 完成，设置 `stop_split: true`。approved projection、transition guard 与 scoped status side effect 共同构成同一 AI proposal status machine helper owner；继续拆成 approved_projection / transition_guard / status_writer 微叶不会形成新的稳定状态 owner、锁 owner、schema owner、route facade 或 runtime persistence owner，只会增加父子接线与治理挂载面。下一步只能进入 BE-001BO-01 `runtime.mutation.ai_proposal` 第八轮父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BN-04 status_transition 单叶 closeout | 收口 |
| 规范矩阵 | stop_split、父子通信、状态机 helper owner、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.status_transition` | 白箱 closeout |
| 模块树 | `runtime.mutation.ai_proposal.status_transition` | 设置 `stop_split: true` |

---

## closeout 结论

`runtime.mutation.ai_proposal.status_transition` 已完成当前范围内的等价基线、抽离方案和实际抽离。
本叶设置:

```text
stop_split: true
```

原因:

- `ai_proposal_approved_status` 只是 approval 成功后的 Approved projection，不形成独立 route、schema 或 persistence owner。
- `is_valid_ai_proposal_transition` 是 `update_ai_proposal_status` 的内部 guard，拆出会迫使父级暴露更多微 helper。
- `update_ai_proposal_status` 拥有 `state.ai_proposals` 写锁、`auth::scoped_key` lookup、非法转换日志和 `updated_at_ms` side effect；它是状态写入的最小稳定边界。
- 三个 helper 共享同一 `RuntimeAiProposalStatus` 状态机语义，继续拆分不会降低耦合，只会增加父级 import、visibility 和治理索引面。
- `approval_review` 仍经父级 `use super::*` 访问受控 helper，未形成 sibling 横向连接。

---

## 已落地文件

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/status_transition.rs
src/runtime/mutation/ai_proposal/approval_review.rs
```

父级保留:

```rust
#[path = "ai_proposal/status_transition.rs"]
mod status_transition;

use status_transition::{ai_proposal_approved_status, update_ai_proposal_status};
```

child 保持:

```rust
use super::*;
```

`is_valid_ai_proposal_transition` 保持 child private；`ai_proposal_approved_status` 保持 `pub(super) fn`；`update_ai_proposal_status` 保持 `pub(super) async fn`。

---

## 等价确认

已确认:

- `ai_proposal_approved_status` 继续返回 `RuntimeAiProposalStatus::Approved`。
- `is_valid_ai_proposal_transition` 继续只允许 `(Submitted, StaticCheckPassed | StaticCheckFailed)`。
- `is_valid_ai_proposal_transition` 继续只允许 `(StaticCheckPassed, Approved | Denied | Expired)`。
- `update_ai_proposal_status` 继续通过 `state.ai_proposals.write().await` 获取写锁。
- `update_ai_proposal_status` 继续通过 `auth::scoped_key(user_id, proposal_id)` 定位记录。
- missing record 继续 no-op。
- 非法转换继续调用 `safe_eprintln!` 并 return。
- 合法转换继续写入 `record.status = status` 与 `record.updated_at_ms = current_time_ms()`。
- `approval_review` 继续经父级 `use super::*` 调用 `ai_proposal_approved_status` 与 `update_ai_proposal_status`，不横向 import `status_transition` sibling。

---

## 未迁移边界

本 closeout 不迁移:

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
- schema owner
- frontend caller
- route facade
- runtime persistence owner
- release transition guard

---

## 验证证据

BE-001BN-03 实际抽离后已验证:

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

本 closeout 批次为 `no code movement`，提交前继续执行治理门禁。

---

## 下一步

下一步只能进入:

```text
BE-001BO-01 runtime.mutation.ai_proposal 第八轮父叶残余判断
```

该父叶残余判断只能评估 proposal create orchestration 是否值得作为下一候选；不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 或 `status_transition`。

---

## 幻觉检查点

AI 声称 BE-001BN-04 完成时，必须说明 `runtime.mutation.ai_proposal.status_transition` 已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner、release transition 或 Rust backend 重构已完成。

---

## 验收标准

1. `223-runtime.mutation.ai_proposal.status_transition单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.status_transition` 标记为 `stop_split: true`。
3. 下一步固定为 BE-001BO-01 `runtime.mutation.ai_proposal` 第八轮父叶残余判断。
4. status_transition 不再继续细拆，除非未来有新的独立状态、锁、schema、route 或 persistence owner 证据并重新走提案流程。
