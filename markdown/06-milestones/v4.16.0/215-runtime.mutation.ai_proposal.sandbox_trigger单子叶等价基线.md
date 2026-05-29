# v4.16.0 runtime.mutation.ai_proposal.sandbox_trigger 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BL-01  
> 基线: `214-runtime.mutation.ai_proposal第六轮父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线，冻结 approve 前 sandbox gate、sandbox report memory-first / disk fallback、create path background sandbox verification task、retry / panic guard、`sandbox_report_url` 回写、失败 lifecycle 和 approval persistence 副作用边界。当前 `no code movement`，下一步只能进入 BE-001BL-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BL-01 sandbox_trigger 单子叶等价基线 | 递归进入子叶 |
| 规范矩阵 | 父子通信、外部证据 gate、background task 语义、closed child 不横连 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.sandbox_trigger` | 新增白箱候选 |
| 模块树 | `runtime.mutation.ai_proposal.sandbox_trigger` | `stop_split: pending` |

---

## 目标白箱

```text
root.backend.runtime.mutation.ai_proposal.sandbox_trigger
```

当前目标仍在父文件中:

```text
src/runtime/mutation/ai_proposal.rs
```

计划目标文件为:

```text
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
```

该文件只能在后续实际抽离批次中创建。BE-001BL-01 不创建文件、不移动代码、不改 handler、不改测试。

---

## 当前函数边界

| 函数 / 入口 | 当前位置 | 输入 | 输出 | 等价约束 |
| --- | --- | --- | --- | --- |
| `load_sandbox_report_for_proposal` | `src/runtime/mutation/ai_proposal.rs` | `&AppState`、`proposal_id: &str` | `Result<SandboxVerificationReport, (StatusCode, String)>` | 必须先查 `state.sandbox_reports` memory cache，再从 `state.sandbox_report_store_dir` 调用 `sandbox_verification::load_sandbox_report_from_disk` |
| `ensure_ai_proposal_can_be_approved` | `src/runtime/mutation/ai_proposal.rs` | `&AppState`、`&RuntimeAiProposalRecord` | `Result<(), (StatusCode, String)>` | 必须保持 config binding、static check、sandbox report existence 与 `SandboxVerdict::CandidateUnderperforms` 四段 gate |
| background sandbox verification task | `create_runtime_ai_proposal` 内部 | `AppState` clone、proposal id、approval record | async side effect | 必须保持 `RequestSandboxVerificationRequest`、`run_sandbox_verification`、3 次退避重试、`catch_unwind`、`JoinHandle` monitoring、`sandbox_report_url` 回写和失败 lifecycle |

---

## 冻结的 gate 语义

`ensure_ai_proposal_can_be_approved` 必须继续保持以下顺序:

1. `proposal.config_domain_binding.is_none()` 时返回 `StatusCode::LOCKED`，错误码为 `strategy_config_ai_binding_required`。
2. `proposal.status != RuntimeAiProposalStatus::StaticCheckPassed` 时返回 `StatusCode::LOCKED`，错误码为 `ai_proposal_static_check_required`。
3. `load_sandbox_report_for_proposal` 失败时返回 `StatusCode::LOCKED`，错误码为 `ai_proposal_sandbox_required`。
4. `sandbox_report.verdict == SandboxVerdict::CandidateUnderperforms` 时返回 `StatusCode::LOCKED`，错误码为 `ai_proposal_sandbox_failed`。
5. 只有四段 gate 全部通过时，approve 路径才允许继续进入 approval review 的 quorum / status side effect。

---

## 冻结的 background task 语义

`create_runtime_ai_proposal` 只在 `RuntimeAiProposalStatus::StaticCheckPassed` 时触发 sandbox task。该 task 的等价语义冻结为:

1. sandbox request 固定为 `RequestSandboxVerificationRequest { backtest_id: None, proposal_id: pid.clone() }`。
2. sandbox runner 固定调用 `sandbox_verification::run_sandbox_verification(&state_clone, &sandbox_request)`。
3. panic guard 固定使用 `std::panic::AssertUnwindSafe(...).catch_unwind().await`。
4. retry 固定为 3 次，失败后在前两次之间执行 `tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt + 1) as u64)).await`。
5. 成功后必须在 `state_clone.approval_records.write().await` 内找到同 proposal id 的 approval，并把 `sandbox_report_url` 更新为 `/api/v1/ai/proposals/{pid}/sandbox-report`。
6. 成功后必须继续调用 `persist_approval(&state_clone.approval_store_dir, &approval).await`，不得绕过 `runtime.mutation.ai_proposal.approval_persistence`。
7. 三次失败后必须追加 `RuntimeApprovalLifecycleEntry`，`reason_code` 固定为 `SANDBOX_VERIFICATION_FAILED`，message 固定说明 3 次尝试全部失败且审批通过路径保持阻断。
8. 三次失败后必须持久化更新后的 approval，并保留 `safe_eprintln!` failure evidence。
9. outer `tokio::spawn` 必须继续监视 inner handle，`handle.await` 出错时保留 sandbox task 异常日志。该监控语义在文档中记为 `JoinHandle monitoring`。

---

## 父级调用边界

`runtime.mutation.ai_proposal.sandbox_trigger` 后续只能由父级 `runtime.mutation.ai_proposal` 连接。当前调用方冻结如下:

| 调用点 | 调用函数 / 入口 | 等价要求 |
| --- | --- | --- |
| approval review child | `approval_review` 通过 `use super::*` 调用 `ensure_ai_proposal_can_be_approved` | sibling 不得横向 import planned sandbox child |
| proposal create orchestration | `create_runtime_ai_proposal` 内部触发 sandbox task | 只允许在后续方案中提取 sandbox task helper，不得迁移整个 create handler |
| approval persistence child | `persist_approval` 经父级受控 helper import 被 sandbox task 调用 | sandbox child 不得横向 import `approval_persistence` sibling |

`src/runtime/mutation/ai_proposal/approval_review.rs` 继续只能经父级 `use super::*` 访问 sandbox gate。后续即使创建 `sandbox_trigger` child，也不得让 `approval_review` 直接横向 import sibling。

---

## 非目标边界

BE-001BL-01 不迁移、不改写、不重排以下节点:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `status_transition`
- proposal create orchestration
- `persist_approval`
- `load_approval_from_disk`
- `approval_persistence`
- `approval_review`
- `AppState`
- schema owner
- frontend caller
- route facade
- runtime persistence owner
- release transition guard

不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review` 或 `approval_persistence`。不得宣称 Rust backend 重构完成。

---

## 下一步

下一步固定为:

```text
BE-001BL-02 runtime.mutation.ai_proposal.sandbox_trigger 抽离方案
```

BE-001BL-02 只能建立抽离方案，固定目标文件、父级 path-attributed child、helper import / visibility、允许迁移清单、回退点与验证门禁。实际创建 `sandbox_trigger.rs` 必须等待后续实际抽离批次。

---

## 验证计划

本批 `no code movement`，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BL-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线，尚未创建 `sandbox_trigger.rs`，也尚未迁移 `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 或 create path background sandbox task。不得宣称 sandbox_trigger 已抽离、status_transition 已迁移、proposal create orchestration 已拆分、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `215-runtime.mutation.ai_proposal.sandbox_trigger单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.sandbox_trigger` 白箱候选，状态为 `stop_split: pending`。
3. 本批不产生 Rust 代码变更，不创建 `sandbox_trigger.rs`。
4. 下一步固定为 BE-001BL-02 `runtime.mutation.ai_proposal.sandbox_trigger` 抽离方案。
