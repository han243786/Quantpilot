# v4.16.0 runtime.mutation.ai_proposal.sandbox_trigger 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BL-03  
> 基线: `215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md`、`216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`  
> 判定: `runtime.mutation.ai_proposal.sandbox_trigger` 第一轮实际抽离完成。`load_sandbox_report_for_proposal` 与 `ensure_ai_proposal_can_be_approved` 已迁入 child 文件，create path 的 background sandbox verification task 已收束为 `spawn_ai_proposal_sandbox_verification` helper。status_transition、proposal create orchestration、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001BL-04 单叶 closeout。  
> 代码动作: code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BL-03 sandbox_trigger 实际抽离 | 物理抽离 |
| 规范矩阵 | 父级受控 helper import、closed child 不横连、background task 等价 | 约束执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.sandbox_trigger` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.sandbox_trigger` | 白箱抽离完成 |

---

## 文件变更

新增:

```text
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增:

```rust
#[path = "ai_proposal/sandbox_trigger.rs"]
mod sandbox_trigger;

use sandbox_trigger::{
    ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification,
};
```

child 固定:

```rust
use super::*;
use futures_util::FutureExt;
```

受控 import 单行锚点: `use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification}`。

父级删除 `futures_util::FutureExt` import；`FutureExt` 随 `catch_unwind` 使用点迁入 child。

---

## 实际迁移清单

已从父级迁入 child:

- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `spawn_ai_proposal_sandbox_verification`

迁移后 `load_sandbox_report_for_proposal` 保持 child private `async fn`；`ensure_ai_proposal_can_be_approved` 保持 `pub(super) async fn`；`spawn_ai_proposal_sandbox_verification` 保持 `pub(super) fn`。`approval_review` 仍经父级 `use super::*` 访问 `ensure_ai_proposal_can_be_approved`，没有横向 import `sandbox_trigger` sibling。`sandbox_trigger` 仍经父级受控名称调用 `persist_approval`，没有横向 import `approval_persistence` sibling。

---

## 行为等价说明

approve 前 sandbox gate 行为保持不变:

1. `ensure_ai_proposal_can_be_approved` 仍按 config binding -> static check -> sandbox report required -> sandbox verdict 顺序执行。
2. `StatusCode::LOCKED` 与 `strategy_config_ai_binding_required`、`ai_proposal_static_check_required`、`ai_proposal_sandbox_required`、`ai_proposal_sandbox_failed` 错误码 / message 不变。
3. `load_sandbox_report_for_proposal` 仍先读 `state.sandbox_reports`，再通过 `sandbox_verification::load_sandbox_report_from_disk(state.sandbox_report_store_dir.as_ref(), proposal_id).await` fallback。

background sandbox verification 行为保持不变:

1. 父级 `create_runtime_ai_proposal` 只把内联 task 替换为 `spawn_ai_proposal_sandbox_verification(state.clone(), proposal_id.clone())`。
2. helper 仍构造 `RequestSandboxVerificationRequest { backtest_id: None, proposal_id: pid.clone() }`。
3. sandbox runner 仍调用 `sandbox_verification::run_sandbox_verification(&state_clone, &sandbox_request)`。
4. panic guard 仍使用 `std::panic::AssertUnwindSafe(...).catch_unwind().await`。
5. retry 次数仍为 3；前两次失败后继续通过 `tokio::time::sleep` 以 `500 * (attempt + 1)` ms 退避。
6. 成功后继续把匹配 proposal id 的 approval `sandbox_report_url` 更新为 `/api/v1/ai/proposals/{pid}/sandbox-report`，并通过 `persist_approval` 持久化。
7. 三次失败后继续追加 `RuntimeApprovalLifecycleEntry`，`reason_code` 保持 `SANDBOX_VERIFICATION_FAILED`，并通过 `persist_approval` 持久化。
8. outer task 继续监视 inner `JoinHandle`；`handle.await` 出错时继续 `safe_eprintln!`。

---

## 调用面保持

| 调用点 | 等价结果 |
| --- | --- |
| `create_runtime_ai_proposal` | 继续在 approval record 与 proposal transition 持久化后触发 sandbox verification |
| `approval_review` | 继续经父级 `use super::*` 调用 `ensure_ai_proposal_can_be_approved` |
| `approval_persistence` | 仍只由父级受控名称提供 `persist_approval`，child 不横向 import sibling |
| route facade | `src/backend/runtime/routes/mutation.rs` 不变 |

---

## 非目标边界

BE-001BL-03 未迁移或修改:

- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `status_transition`
- proposal create orchestration 主体
- approval record construction
- `persist_approval`
- `load_approval_from_disk`
- `approval_persistence`
- `approval_review`
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

已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review` 与 `approval_persistence` 未回收、未重拆。

---

## 验证计划

实际抽离批次必须运行:

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

## 下一步

下一步只能进入:

```text
BE-001BL-04 runtime.mutation.ai_proposal.sandbox_trigger 单叶 closeout
```

该 closeout 只能判断本叶是否停止细分，不得继续迁移 status_transition、proposal create orchestration、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BL-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.sandbox_trigger` 第一轮实际抽离，尚未完成单叶 closeout。不得宣称 status_transition、proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变，也不得宣称 Rust backend 重构完成。

---

## 验收标准

1. `src/runtime/mutation/ai_proposal/sandbox_trigger.rs` 存在，并承接 sandbox approve gate 与 background sandbox verification helper。
2. 父级 `src/runtime/mutation/ai_proposal.rs` 只保留 path-attributed child、受控 helper import 和 `spawn_ai_proposal_sandbox_verification(state.clone(), proposal_id.clone())` 调用。
3. `approval_review` 仍经父级受控 helper 名称访问，不横向 import sibling。
4. `217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
5. 下一步固定为 BE-001BL-04 单叶 closeout。
