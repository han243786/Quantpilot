# v4.16.0 runtime.mutation.ai_proposal.sandbox_trigger 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BL-02  
> 基线: `214-runtime.mutation.ai_proposal第六轮父叶残余判断.md`、`215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`  
> 判定: 固定 BE-001BL-03 的实际抽离方案。下一步只允许创建 `src/runtime/mutation/ai_proposal/sandbox_trigger.rs`，迁移 `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved`，并把 `create_runtime_ai_proposal` 内部 background sandbox verification task 收束为父级调用的 `spawn_ai_proposal_sandbox_verification` helper。当前 `no code movement`。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BL-02 sandbox_trigger 抽离方案 | 方案固化 |
| 规范矩阵 | 父子通信、受控 helper import、background task 等价、closed child 不横连 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.sandbox_trigger` | 抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.sandbox_trigger` | 计划物理抽离 |

---

## 目标文件与父级声明

BE-001BL-03 允许创建:

```text
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 固定新增:

```rust
#[path = "ai_proposal/sandbox_trigger.rs"]
mod sandbox_trigger;
```

父级固定通过受控 helper import 连接 child:

```rust
use sandbox_trigger::{ensure_ai_proposal_can_be_approved, spawn_ai_proposal_sandbox_verification};
```

`load_sandbox_report_for_proposal` 不从 child 暴露给父级，保持 child-private；`ensure_ai_proposal_can_be_approved` 与 `spawn_ai_proposal_sandbox_verification` 只用 `pub(super)` 暴露给父级。

`futures_util::FutureExt` import 随 `catch_unwind` 使用点迁入 `sandbox_trigger.rs`，父级若不再使用则删除父级 import。

---

## 允许迁移清单

BE-001BL-03 只允许迁移 / 提取以下内容:

| 内容 | 目标形态 | 可见性 | 说明 |
| --- | --- | --- | --- |
| `load_sandbox_report_for_proposal` | 迁入 `sandbox_trigger.rs` | child private `async fn` | 仅被 `ensure_ai_proposal_can_be_approved` 调用 |
| `ensure_ai_proposal_can_be_approved` | 迁入 `sandbox_trigger.rs` | `pub(super) async fn` | 供父级 re-import，`approval_review` 仍经 `use super::*` 调用 |
| background sandbox task | 提取为 `spawn_ai_proposal_sandbox_verification` | `pub(super) fn` | 父级 create flow 在 approval record 与 proposal transition 持久化后调用 |

父级 `create_runtime_ai_proposal` 只允许把原内联 sandbox task 替换为:

```rust
spawn_ai_proposal_sandbox_verification(state.clone(), proposal_id.clone());
```

不得移动 `create_runtime_ai_proposal` 的 capability guard、source context、static check、record construction、approval record construction、approval persistence、proposal transition persistence 或 final response。

---

## 等价要求

BE-001BL-03 必须保持以下行为等价:

1. `ensure_ai_proposal_can_be_approved` 的四段 gate 顺序不变: config binding -> static check -> sandbox report required -> sandbox verdict。
2. `StatusCode::LOCKED`、`strategy_config_ai_binding_required`、`ai_proposal_static_check_required`、`ai_proposal_sandbox_required`、`ai_proposal_sandbox_failed` 错误码与 message 不变。
3. `load_sandbox_report_for_proposal` 继续先读 `state.sandbox_reports`，再调用 `sandbox_verification::load_sandbox_report_from_disk(state.sandbox_report_store_dir.as_ref(), proposal_id).await`。
4. `spawn_ai_proposal_sandbox_verification` 继续构造 `RequestSandboxVerificationRequest { backtest_id: None, proposal_id: pid.clone() }`。
5. sandbox runner 继续调用 `sandbox_verification::run_sandbox_verification(&state_clone, &sandbox_request)`。
6. panic guard 继续使用 `std::panic::AssertUnwindSafe(...).catch_unwind().await`。
7. retry 次数继续为 3；前两次失败后继续通过 `tokio::time::sleep` 以 `500 * (attempt + 1)` ms 退避。
8. 成功后继续更新同 proposal id approval 的 `sandbox_report_url` 为 `/api/v1/ai/proposals/{pid}/sandbox-report`。
9. 成功和失败后继续通过父级受控 helper `persist_approval` 持久化 approval，不得横向 import `approval_persistence` sibling。
10. 三次失败后继续追加 `RuntimeApprovalLifecycleEntry`，`reason_code` 为 `SANDBOX_VERIFICATION_FAILED`，并保留失败 message。
11. outer task 继续监视 inner `JoinHandle`，`handle.await` 出错时继续 `safe_eprintln!`。

---

## 父子通信规则

`runtime.mutation.ai_proposal.sandbox_trigger` 只能由父级 `runtime.mutation.ai_proposal` 连接:

- `approval_review` 继续通过 `use super::*` 调用 `ensure_ai_proposal_can_be_approved`，不得直接 import `sandbox_trigger`。
- `sandbox_trigger` 通过 `use super::*` 访问 `AppState`、`RequestSandboxVerificationRequest`、`RuntimeApprovalLifecycleEntry`、`persist_approval` 等父级受控名称。
- `sandbox_trigger` 不得横向 import `approval_persistence`、`approval_review`、`record_query`、`event_lifecycle`、`static_check` 或 `source_governance_identity` sibling。
- 发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 非目标边界

BE-001BL-03 不得迁移或修改:

- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `status_transition`
- proposal create orchestration 的主体
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

不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review` 或 `approval_persistence`。

---

## 回退点

如果 BE-001BL-03 编译或测试失败，回退方式固定为:

1. 删除 `src/runtime/mutation/ai_proposal/sandbox_trigger.rs`。
2. 将 `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 和 background sandbox task 原样放回 `src/runtime/mutation/ai_proposal.rs`。
3. 删除父级 `mod sandbox_trigger` 与 `use sandbox_trigger::{...};`。
4. 将 `use futures_util::FutureExt;` 放回父级。
5. 重新运行同一组验证门禁。

---

## 验证计划

BE-001BL-03 实际抽离必须运行:

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

本 BE-001BL-02 方案批次仍为 `no code movement`，提交前只需要治理门禁:

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
BE-001BL-03 runtime.mutation.ai_proposal.sandbox_trigger 实际抽离
```

---

## 幻觉检查点

AI 声称 BE-001BL-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案，尚未创建 `sandbox_trigger.rs`，也尚未迁移 `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 或 background sandbox task。不得宣称 sandbox_trigger 已抽离、status_transition 已迁移、proposal create orchestration 已拆分、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `216-runtime.mutation.ai_proposal.sandbox_trigger抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001BL-03 的目标文件、父级声明、helper import、`FutureExt` 迁移、允许迁移清单、非目标和回退点已固定。
3. 本批不产生 Rust 代码变更，不创建 `sandbox_trigger.rs`。
4. 下一步固定为 BE-001BL-03 实际抽离。
