# v4.16.0 runtime.mutation.ai_proposal.static_check 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AZ-03  
> 基线: `185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md`、`186-runtime.mutation.ai_proposal.static_check抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check` 第一轮实际抽离完成。static check helper 与对应单测已迁入 child 文件；approval review、record query、source governance、event lifecycle、sandbox trigger、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001AZ-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AZ-03 static_check 实际抽离 | 已落地 |
| 规范矩阵 | 父子通信、`pub(super)` helper visibility、测试归属、非目标边界 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.static_check` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.static_check` | 白箱抽离完成 |

---

## 实际文件变更

新增 child 文件:

```text
src/runtime/mutation/ai_proposal/static_check.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增 path-attributed child:

```rust
#[path = "ai_proposal/static_check.rs"]
mod static_check;
```

父级通过受控 helper import 继续调用 static check:

```rust
use static_check::{
    ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
};
```

child 只通过父级白箱输入取依赖:

```rust
use super::*;
```

---

## 已迁移 helper

| helper | BE-001AZ-03 visibility | 调整原因 |
| --- | --- | --- |
| `validate_hash_identity` | `pub(super)` | `create_runtime_ai_proposal` 仍直接校验 `prompt_hash` / `evidence_hash`，因此必须由父级受控调用 |
| `validate_ai_model_identity` | `pub(super)` | `create_runtime_ai_proposal` 继续做 model identity 入参校验 |
| `ai_proposal_static_check_result` | `pub(super)` | `create_runtime_ai_proposal` 继续生成 candidate static check |
| `is_valid_hash_identity` | private | 仅服务 child 内部 digest 校验 |
| `is_v4_ai_proposal_target` | private | 仅服务 child 内部 v4 target detection |
| `expected_config_domain_for_target` | private | 仅服务 child 内部 config domain binding |
| `validate_ai_proposal_config_domain_binding` | private | 仅服务 child 内部 static check aggregate |
| `analyze_v4_backtest_artifact_for_ai` | private + `#[allow(dead_code)]` | 当前只由 child 内部测试覆盖，不暴露给父级 |

与 186 号方案相比，`validate_hash_identity` 的 visibility 从计划 private 调整为 `pub(super)`。这是编译期真实边界发现: 父级 create flow 在进入 static aggregate 前仍必须单独校验 prompt/evidence hash。该调整仍符合父子通信硬规则，未形成横向连接。

---

## 测试迁移

以下 static check / v4 analysis 单测已随 child 迁移:

- `v4_ai_proposal_static_check_requires_backtest_source`
- `ai_proposal_static_check_requires_config_domain_binding`
- `ai_proposal_static_check_accepts_matching_config_domain_binding`
- `v4_artifact_analysis_summarizes_trajectory_and_fill_rate`

以下 approval gate 单测仍保留在父级，作为 approval_review / sandbox gate 后续候选边界:

- `ai_proposal_approval_requires_binding_and_sandbox_report`

---

## 未迁移边界

BE-001AZ-03 未迁移或修改:

- `create_runtime_ai_proposal`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得把本批解释为 approval_review、record_query、source_governance_identity、event_lifecycle、approval_persistence 或 sandbox_trigger 已拆分。

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
- `cargo test --no-run`
- `cargo test -p quantpilot --test api_ai_proposal`

---

## 回退点

若 BE-001AZ-03 后续验证失败，只允许回退本批新增/修改:

- `src/runtime/mutation/ai_proposal/static_check.rs`
- `src/runtime/mutation/ai_proposal.rs` 中的 `#[path = "ai_proposal/static_check.rs"] mod static_check;`
- `src/runtime/mutation/ai_proposal.rs` 中的 `use static_check::{...};`
- `src/runtime/mutation/ai_proposal.rs` 中由本批 static_check helper / static check 单测迁移造成的删除

不得回退 BE-001AY-03 已完成的 AI proposal child 抽离，不得改 route facade、schema、AppState、frontend caller、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001AZ-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.static_check` 第一轮实际抽离。不得宣称 static_check 已完成 closeout、approval review 已拆分、record query 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `187-runtime.mutation.ai_proposal.static_check抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/ai_proposal/static_check.rs` 存在，并承载 static check helper 与对应单测。
3. 父级只通过 path-attributed child 和受控 `pub(super)` helper import 调用 child。
4. approval gate 单测仍在父级，approval_review / sandbox_trigger 未被宣称完成。
5. 验证通过后，后续只能进入 BE-001AZ-04 单叶 closeout，判断本 child 是否值得继续细拆。
