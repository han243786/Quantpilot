# v4.16.0 runtime.mutation.ai_proposal 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AY-03  
> 方案: `182-runtime.mutation.ai_proposal抽离方案.md`  
> 判定: `runtime.mutation.ai_proposal` 第一轮实际抽离已完成；AI proposal / approval handlers 与专属 helper 已迁入 `src/runtime/mutation/ai_proposal.rs`，父级通过 `mutation_ai_proposal` child 声明和 public handler re-export 保持 route 调用面。下一步只能进入 BE-001AY-04 单叶 closeout。  
> 代码动作: moved AI proposal / approval handlers

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AY-03 AI proposal 实际抽离 | 代码抽离 |
| 规范矩阵 | 父子通信、approval 锁顺序、public handler re-export、shared helper 保留 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 子叶落位 |
| 模块树 | `runtime.mutation.ai_proposal` | 白箱更新 |

---

## 实际文件变更

新增:

```text
src/runtime/mutation/ai_proposal.rs
```

父级 `src/runtime/mod.rs` 新增 child 声明:

```rust
#[path = "mutation/ai_proposal.rs"]
mod mutation_ai_proposal;
```

父级 `src/runtime/mod.rs` 新增兼容出口:

```rust
pub(crate) use mutation_ai_proposal::{
    approve_ai_proposal, claim_ai_proposal_review, create_runtime_ai_proposal,
    get_runtime_ai_proposal_detail, get_runtime_approval_detail, list_runtime_ai_proposals,
    list_runtime_approvals, reject_ai_proposal,
};
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
use futures_util::FutureExt;
```

---

## 已迁移 public handler

已从 `src/runtime/mutation.rs` 迁入 child:

- `pub(crate) async fn create_runtime_ai_proposal`
- `pub(crate) async fn list_runtime_ai_proposals`
- `pub(crate) async fn get_runtime_ai_proposal_detail`
- `pub(crate) async fn list_runtime_approvals`
- `pub(crate) async fn get_runtime_approval_detail`
- `pub(crate) async fn approve_ai_proposal`
- `pub(crate) async fn reject_ai_proposal`
- `pub(crate) async fn claim_ai_proposal_review`

route facade `src/backend/runtime/routes/mutation.rs` 未修改，仍经父级 re-export 调用同名 handler。

---

## 已迁移 helper 清单

已迁入 `src/runtime/mutation/ai_proposal.rs`:

- `validate_hash_identity`
- `is_valid_hash_identity`
- `validate_ai_model_identity`
- `ai_proposal_static_check_result`
- `is_v4_ai_proposal_target`
- `expected_config_domain_for_target`
- `validate_ai_proposal_config_domain_binding`
- `analyze_v4_backtest_artifact_for_ai`
- `RuntimeAiProposalSourceContext`
- `load_runtime_ai_proposal_source_context`
- `runtime_ai_proposal_governance`
- `runtime_ai_proposal_record_id`
- `ai_proposal_event_contract`
- `build_runtime_ai_proposal_event`
- `ai_proposal_lifecycle_entry`
- `persist_runtime_ai_proposal_transition`
- `load_runtime_ai_proposal_for_user`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`

---

## 父级保留 shared helper

`src/runtime/mutation.rs` 仍保留以下 shared helper，不随本批迁移:

- `canonical_runtime_parameter_version`
- `validate_runtime_parameter_mutation_target`
- `runtime_mode_from_events`
- `status_contract_value`
- `mutation_event_contract`
- `build_runtime_parameter_mutation_event`
- `append_parameter_mutation_events_to_run`
- `runtime_parameter_mutation_governance`
- `governance_with_parameter_version`

保留理由: 这些 helper 仍同时服务 parameter mutation 与 AI proposal source event / governance 语义，不能为了本子叶抽离回改已 closeout 的 parameter mutation 子树。

---

## 等价保持

已保持:

- `/api/runtime/ai-proposals` GET / POST route 到 `list_runtime_ai_proposals`、`create_runtime_ai_proposal`
- `/api/runtime/ai-proposals/:ai_proposal_id` route 到 `get_runtime_ai_proposal_detail`
- `/api/v1/ai/approvals` route 到 `list_runtime_approvals`
- `/api/v1/ai/approvals/:approval_id` route 到 `get_runtime_approval_detail`
- `/api/v1/ai/proposals/:proposal_id/approve` route 到 `approve_ai_proposal`
- `/api/v1/ai/proposals/:proposal_id/reject` route 到 `reject_ai_proposal`
- `/api/v1/ai/proposals/:proposal_id/claim` route 到 `claim_ai_proposal_review`
- `approval_records -> ai_proposals` lock order
- sandbox verification retry、`FutureExt` `catch_unwind`、JoinHandle monitoring 与 `sandbox_report_url` 回写
- `AppState` owner、schema owner、frontend caller、runtime persistence owner 与 release transition guard

---

## 未迁移边界

本批未迁移:

- `runtime.mutation.parameter_mutation`
- `runtime.mutation.parameter_mutation.proposal_creation`
- `runtime.mutation.parameter_mutation.record_query`
- `runtime.mutation.parameter_mutation.transition_lifecycle`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- runtime persistence owner `src/runtime_persistence.rs`
- route facade `src/backend/runtime/routes/mutation.rs`
- report / evidence / experiment / ops routes
- release transition guard

不得把本批解释为 approval review 已单独 closeout、static check 已继续细拆、state owner 已迁移或发布过渡已启动。

---

## 回退点

若 BE-001AY-04 closeout 或后续验证失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal.rs`
- `#[path = "mutation/ai_proposal.rs"] mod mutation_ai_proposal;`
- `pub(crate) use mutation_ai_proposal::{...};`
- `src/runtime/mutation.rs` 中由本批 AI proposal 迁移造成的删除
- `use futures_util::FutureExt` 的 child 迁移

不得回改已 closeout 的 parameter mutation 子树，不得修改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

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

AI 声称 BE-001AY-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal` 第一轮实际抽离；AI proposal / approval handlers 已迁入 child，但本叶尚未做 BE-001AY-04 单叶 closeout，也尚未判断 static_check、record_query、approval_review、status_transition、approval_persistence 或 sandbox_trigger 是否值得继续细拆。不得宣称 `AppState`、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `src/runtime/mutation/ai_proposal.rs` 已创建并承接 AI proposal / approval public handler 与专属 helper。
2. `src/runtime/mod.rs` 已通过 `mutation_ai_proposal` child 声明和 public handler re-export 保持 route 调用面。
3. `src/runtime/mutation.rs` 只保留 shared mutation helper 与父级 owner，不再直接持有 AI proposal 专属 handler/helper。
4. 下一步只能进入 BE-001AY-04 单叶 closeout。
