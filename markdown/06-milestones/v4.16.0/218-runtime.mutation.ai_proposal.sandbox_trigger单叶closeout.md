# v4.16.0 runtime.mutation.ai_proposal.sandbox_trigger 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BL-04  
> 基线: `215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md`、`216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md`、`217-runtime.mutation.ai_proposal.sandbox_trigger抽离记录.md`、`src/runtime/mutation/ai_proposal/sandbox_trigger.rs`  
> 判定: `runtime.mutation.ai_proposal.sandbox_trigger` 单叶 closeout 完成，设置 `stop_split: true`。sandbox report fallback、approve 前 gate、background sandbox verification task、retry / panic guard、`sandbox_report_url` 回写和 failed lifecycle 共同构成同一 external sandbox evidence owner；继续拆成 report_loader / approval_gate / background_task 微叶不会产生新的稳定状态 owner、锁 owner、schema owner、route facade 或 runtime persistence owner，只会增加父子接线和治理挂载面。下一步只能进入 BE-001BM-01 `runtime.mutation.ai_proposal` 第七轮父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BL-04 sandbox_trigger 单叶 closeout | 收口 |
| 规范矩阵 | stop_split、父子通信、external sandbox evidence owner、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.sandbox_trigger` | 白箱 closeout |
| 模块树 | `runtime.mutation.ai_proposal.sandbox_trigger` | 设置 `stop_split: true` |

---

## closeout 结论

`runtime.mutation.ai_proposal.sandbox_trigger` 已完成当前范围内的等价基线、抽离方案和实际抽离。

本叶设置:

```text
stop_split: true
```

原因:

- `load_sandbox_report_for_proposal` 与 `ensure_ai_proposal_can_be_approved` 都服务 approve 前 sandbox evidence gate，拆开不会形成独立 route 或 schema owner。
- `spawn_ai_proposal_sandbox_verification` 与 gate 共享同一个 proposal id、sandbox report URL、approval lifecycle 和 persisted approval side effect。
- `RequestSandboxVerificationRequest`、`run_sandbox_verification`、`JoinHandle`、`catch_unwind`、`tokio::time::sleep` retry、`sandbox_report_url` 与 `SANDBOX_VERIFICATION_FAILED` 是同一 sandbox verification contract 的执行与反馈两端。
- `persist_approval` 仍由父级受控名称提供；本叶不拥有 approval_persistence sibling，也不拥有 runtime persistence owner。
- 继续拆为 report_loader / approval_gate / background_task 微叶不会减少依赖，反而会增加父级 import、visibility 和治理索引面。

---

## 已落地文件

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
```

父级保留:

```rust
#[path = "ai_proposal/sandbox_trigger.rs"]
mod sandbox_trigger;

use sandbox_trigger::{
    ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification,
};
```

child 保持:

```rust
use super::*;
use futures_util::FutureExt;
```

受控 import 单行锚点: `use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification}`。

---

## 等价确认

已确认:

- `load_sandbox_report_for_proposal` 继续先读 `state.sandbox_reports`，再读 `sandbox_report_store_dir` disk fallback。
- `ensure_ai_proposal_can_be_approved` 继续保持 config binding -> static check -> sandbox report required -> sandbox verdict gate 顺序。
- `StatusCode::LOCKED` 与 `strategy_config_ai_binding_required`、`ai_proposal_static_check_required`、`ai_proposal_sandbox_required`、`ai_proposal_sandbox_failed` 不变。
- `spawn_ai_proposal_sandbox_verification` 继续构造 `RequestSandboxVerificationRequest { backtest_id: None, proposal_id: pid.clone() }`。
- sandbox runner 继续调用 `sandbox_verification::run_sandbox_verification(&state_clone, &sandbox_request)`。
- panic guard 继续使用 `std::panic::AssertUnwindSafe(...).catch_unwind().await`。
- retry 次数继续为 3；前两次失败后继续通过 `tokio::time::sleep` 以 `500 * (attempt + 1)` ms 退避。
- 成功后继续写回 `sandbox_report_url`，三次失败后继续追加 `RuntimeApprovalLifecycleEntry` 与 `SANDBOX_VERIFICATION_FAILED`。
- `approval_review` 继续经父级 `use super::*` 访问 `ensure_ai_proposal_can_be_approved`，不横向 import sibling。

---

## 未迁移边界

本 closeout 不迁移:

- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `status_transition`
- proposal create orchestration 主体
- `approval_persistence`
- `approval_review`
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

BE-001BL-03 实际抽离后已验证:

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
BE-001BM-01 runtime.mutation.ai_proposal 第七轮父叶残余判断
```

该父叶残余判断只能评估 `status_transition` 与 proposal create orchestration 等剩余稳定职责，不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence` 或 `sandbox_trigger`。

---

## 幻觉检查点

AI 声称 BE-001BL-04 完成时，必须说明 `runtime.mutation.ai_proposal.sandbox_trigger` 已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 status_transition、proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner、release transition 或 Rust backend 重构已完成。

---

## 验收标准

1. `218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.sandbox_trigger` 标记为 `stop_split: true`。
3. 下一步固定为 BE-001BM-01 `runtime.mutation.ai_proposal` 第七轮父叶残余判断。
4. sandbox_trigger 不再继续细拆，除非未来有新的独立状态/锁/schema/route owner 证据并重新走提案流程。
