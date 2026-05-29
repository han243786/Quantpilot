# v4.16.0 runtime.mutation.ai_proposal 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AY-02  
> 基线: `181-runtime.mutation.ai_proposal单子叶等价基线.md`、`src/runtime/mutation.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/mutation.rs`、`tests/api_ai_proposal.rs`  
> 判定: `runtime.mutation.ai_proposal` 抽离方案已建立。当前仍为 `no code movement`；只固定 BE-001AY-03 的目标文件、父级声明、public handler re-export、child import、迁移清单、保留清单、回退点和验证门禁。下一步只能进入 BE-001AY-03 实际抽离。  
> 代码动作: `no code movement`

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AY-02 AI proposal 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、public handler re-export、审批锁顺序、shared helper 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 子叶抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal` | 白箱方案 |

---

## 目标文件与父级声明

BE-001AY-03 只允许创建一个目标文件:

```text
src/runtime/mutation/ai_proposal.rs
```

`src/runtime/mod.rs` 只允许新增一个 child 声明:

```rust
#[path = "mutation/ai_proposal.rs"]
mod mutation_ai_proposal;
```

`src/runtime/mod.rs` 只允许新增一个 public handler re-export:

```rust
pub(crate) use mutation_ai_proposal::{
    approve_ai_proposal, claim_ai_proposal_review, create_runtime_ai_proposal,
    get_runtime_ai_proposal_detail, get_runtime_approval_detail, list_runtime_ai_proposals,
    list_runtime_approvals, reject_ai_proposal,
};
```

child 文件必须以父级白箱输入为唯一来源:

```rust
use super::*;
use futures_util::FutureExt;
```

`src/runtime/mutation.rs` 顶部的 `use futures_util::FutureExt;` 只允许随本次实际抽离迁入 child；不得在父级留下未使用 import。

---

## BE-001AY-03 允许迁移清单

只允许迁移以下 AI proposal / approval 专属项:

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
- `create_runtime_ai_proposal`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`

迁移后 `src/runtime/mutation/ai_proposal.rs` 必须继续通过 `use super::*;` 调用父级 shared helper 与 shared owner。

---

## 必须保留在父级的 shared helper

BE-001AY-03 不得迁移以下 shared helper:

- `canonical_runtime_parameter_version`
- `validate_runtime_parameter_mutation_target`
- `runtime_mode_from_events`
- `status_contract_value`
- `mutation_event_contract`
- `build_runtime_parameter_mutation_event`
- `append_parameter_mutation_events_to_run`
- `runtime_parameter_mutation_governance`
- `governance_with_parameter_version`

保留理由:

- `canonical_runtime_parameter_version` 和 `validate_runtime_parameter_mutation_target` 同时服务 parameter mutation 与 AI proposal。
- `append_parameter_mutation_events_to_run` 和 `governance_with_parameter_version` 同时服务 parameter mutation lifecycle 与 AI proposal source event append。
- parameter mutation 已 closeout，不能为了 AI proposal 抽离回改已关闭子叶。

---

## route 与 handler 等价约束

BE-001AY-03 迁移后，以下 route 到 handler 的绑定必须保持不变:

| route | method | handler |
| --- | --- | --- |
| `/api/runtime/ai-proposals` | GET | `list_runtime_ai_proposals` |
| `/api/runtime/ai-proposals` | POST | `create_runtime_ai_proposal` |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `get_runtime_ai_proposal_detail` |
| `/api/v1/ai/approvals` | GET | `list_runtime_approvals` |
| `/api/v1/ai/approvals/:approval_id` | GET | `get_runtime_approval_detail` |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `approve_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `reject_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `claim_ai_proposal_review` |

`src/backend/runtime/routes/mutation.rs` 不得修改 route path、method 或 handler name。

---

## 迁移时允许的最小适配

只允许以下编译适配:

- 将 child 内 sandbox loader 调用调整为 child scope 可解析的形式，例如 `sandbox_verification::load_sandbox_report_from_disk` 或等价路径。
- 将 `use futures_util::FutureExt;` 移入 child，保证 `.catch_unwind()` trait 在 child scope 可用。
- 仅因 rustfmt 产生的换行变化可以接受。

不得借机改变:

- `approval_records -> ai_proposals` lock order
- sandbox verification retry / catch_unwind / JoinHandle monitoring
- `sandbox_report_url` 回写
- static check details
- lifecycle sequence
- approval id generation
- persistence file name
- response schema

---

## 非目标

本批和 BE-001AY-03 均不得迁移或修改:

- `runtime.mutation.parameter_mutation`
- `runtime.mutation.parameter_mutation.proposal_creation`
- `runtime.mutation.parameter_mutation.record_query`
- `runtime.mutation.parameter_mutation.transition_lifecycle`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade
- runtime persistence owner
- report / evidence / experiment / ops routes
- release transition guard

不得把 `runtime.mutation.ai_proposal` 直接拆成 static_check、record_query、approval_review、status_transition、approval_persistence 或 sandbox_trigger。继续细拆只能在 BE-001AY-04 单叶 closeout 后按递归流程判断。

---

## 回退点

如果 BE-001AY-03 编译或等价检查失败，只允许回退本批新增/修改:

- `#[path = "mutation/ai_proposal.rs"] mod mutation_ai_proposal;`
- `pub(crate) use mutation_ai_proposal::{...};`
- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation.rs` 中由本次 AI proposal 迁移造成的删减
- `use futures_util::FutureExt;` 的迁移

不得回退或改写已 closeout 的 parameter mutation 子树，不得修改 route facade、schema、AppState、frontend caller 或 release transition guard。

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

AI 声称 BE-001AY-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal` 抽离方案，仍为 `no code movement`；目标文件尚未创建，handler/helper 尚未迁移。下一步只能进入 BE-001AY-03 实际抽离。不得宣称 AI proposal 已抽离、approval review 已单独拆分、AppState/schema/frontend caller 已改变、parameter mutation 已回改、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `182-runtime.mutation.ai_proposal抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 child 声明、public handler re-export、child import、允许迁移清单、保留清单和回退点已冻结。
3. 治理门禁能发现本文档、`no code movement`、下一批 BE-001AY-03、目标文件、关键 handler/helper、shared helper 保留、锁顺序和验证门禁缺失。
4. 本批验证通过后，后续才能进入 BE-001AY-03 实际抽离。
