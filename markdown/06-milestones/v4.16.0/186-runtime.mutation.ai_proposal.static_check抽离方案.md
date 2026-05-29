# v4.16.0 runtime.mutation.ai_proposal.static_check 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AZ-02  
> 基线: `185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check` 抽离方案已建立。当前仍为 `no code movement`；只固定 BE-001AZ-03 的目标文件、父级声明、helper import、visibility、迁移清单、非目标、回退点和验证门禁。下一步只能进入 BE-001AZ-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AZ-02 static_check 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、`pub(super)` helper visibility、测试归属、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.static_check` | 子叶抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.static_check` | 白箱方案 |

---

## 目标文件与父级声明

BE-001AZ-03 只允许创建一个目标文件:

```text
src/runtime/mutation/ai_proposal/static_check.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 只允许新增一个 child 声明:

```rust
#[path = "ai_proposal/static_check.rs"]
mod static_check;
```

父级只允许新增受控 helper import:

```rust
use static_check::{ai_proposal_static_check_result, validate_ai_model_identity};
```

child 文件必须继续只通过父级白箱输入取依赖:

```rust
use super::*;
```

需要被父级调用的 helper 只允许使用 `pub(super)`:

```rust
pub(super) fn validate_ai_model_identity(...)
pub(super) fn ai_proposal_static_check_result(...)
```

其余 helper 默认保持 private。

---

## BE-001AZ-03 允许迁移清单

只允许迁移以下 validation / analysis helper:

- `validate_hash_identity`
- `is_valid_hash_identity`
- `validate_ai_model_identity`
- `ai_proposal_static_check_result`
- `is_v4_ai_proposal_target`
- `expected_config_domain_for_target`
- `validate_ai_proposal_config_domain_binding`
- `analyze_v4_backtest_artifact_for_ai`

允许把 child 内部单测中只覆盖 static check / v4 analysis 的测试随 helper 一起迁移:

- `v4_ai_proposal_static_check_requires_backtest_source`
- `ai_proposal_static_check_requires_config_domain_binding`
- `ai_proposal_static_check_accepts_matching_config_domain_binding`
- `v4_artifact_analysis_summarizes_trajectory_and_fill_rate`

不得迁移 approval gate 测试 `ai_proposal_approval_requires_binding_and_sandbox_report`，它属于 approval_review / sandbox gate 后续候选。

---

## visibility 规则

| helper | BE-001AZ-03 visibility | 原因 |
| --- | --- | --- |
| `validate_ai_model_identity` | `pub(super)` | `create_runtime_ai_proposal` 必须在父叶继续调用 |
| `ai_proposal_static_check_result` | `pub(super)` | `create_runtime_ai_proposal` 必须在父叶继续调用 |
| `validate_hash_identity` | private | 只服务 child 内部 digest 校验 |
| `is_valid_hash_identity` | private | 只服务 child 内部 digest 校验 |
| `is_v4_ai_proposal_target` | private | 只服务 static check aggregate |
| `expected_config_domain_for_target` | private | 只服务 config binding |
| `validate_ai_proposal_config_domain_binding` | private | 只服务 static check aggregate |
| `analyze_v4_backtest_artifact_for_ai` | private + `#[allow(dead_code)]` | 当前只被 child 内部单测覆盖，不暴露给父级 |

---

## route / handler 等价约束

BE-001AZ-03 迁移后，以下 public handler 的签名、调用方和行为必须保持不变:

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

BE-001AZ-03 不得迁移或修改:

- `create_runtime_ai_proposal`
- source/governance/id helper
- event/lifecycle helper
- proposal list/detail
- approval list/detail
- approve/reject/claim
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade
- runtime persistence owner
- release transition guard

不得把本批解释为 approval_review、record_query、source_governance_identity、event_lifecycle、approval_persistence 或 sandbox_trigger 已拆分。

---

## 回退点

若 BE-001AZ-03 编译或等价检查失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/static_check.rs`
- `#[path = "ai_proposal/static_check.rs"] mod static_check;`
- `use static_check::{ai_proposal_static_check_result, validate_ai_model_identity};`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 static_check 迁移造成的删除

不得回改 BE-001AY-03 已完成的 AI proposal child 抽离，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

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

AI 声称 BE-001AZ-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.static_check` 抽离方案，仍为 `no code movement`；目标文件尚未创建，helper 尚未迁移。下一步只能进入 BE-001AZ-03 实际抽离。不得宣称 static_check 已抽离、approval review 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `186-runtime.mutation.ai_proposal.static_check抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 child 声明、helper import、`pub(super)` visibility、允许迁移清单和回退点已冻结。
3. 治理门禁能发现本方案、`no code movement`、下一批 BE-001AZ-03、目标文件、关键 helper、非目标边界和验证门禁缺失。
4. 本批验证通过后，后续才能进入 BE-001AZ-03 实际抽离。
