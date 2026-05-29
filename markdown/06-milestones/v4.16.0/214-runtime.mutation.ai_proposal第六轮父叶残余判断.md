# v4.16.0 runtime.mutation.ai_proposal 第六轮父叶残余判断
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BK-01  
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`188-runtime.mutation.ai_proposal.static_check单叶closeout.md`、`193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`、`198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`、`203-runtime.mutation.ai_proposal.record_query单叶closeout.md`、`208-runtime.mutation.ai_proposal.approval_review单叶closeout.md`、`213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review` 与 `approval_persistence` 均已 closeout 并设置 `stop_split: true`，但父叶仍存在 `sandbox_trigger`、`status_transition` 与 proposal create orchestration 等稳定残余职责，父叶继续保持 `stop_split: false`。下一步只能进入 BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BK-01 AI proposal 第六轮父叶残余判断 | 递归回到父叶 |
| 规范矩阵 | closed child 不回收、父级残余排序、禁止发布过渡 | 约束收紧 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 残余职责排序 |
| 模块树 | `runtime.mutation.ai_proposal` | `stop_split: false` |

---

## 已 closeout 子叶

| 子叶 | 状态 | 判定 |
| --- | --- | --- |
| `runtime.mutation.ai_proposal.static_check` | BE-001AZ-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.source_governance_identity` | BE-001BB-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.event_lifecycle` | BE-001BD-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.record_query` | BE-001BF-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.approval_review` | BE-001BH-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.approval_persistence` | BE-001BJ-04 closeout | `stop_split: true`，不继续细拆 |

---

## 当前父叶残余

`src/runtime/mutation/ai_proposal.rs` 当前仍直接承接以下稳定职责:

| 残余职责 | 当前函数 / 入口 | 是否值得继续拆 | 原因 |
| --- | --- | --- | --- |
| sandbox_trigger | `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 与 `create_runtime_ai_proposal` 内部 background sandbox task | 是，下一候选 | 本块同时持有 approve 前 sandbox gate、`RequestSandboxVerificationRequest`、`run_sandbox_verification`、3 次退避重试、`catch_unwind`、`JoinHandle` 监控、`sandbox_report_url` 回写、失败 lifecycle 与 `persist_approval` 副作用，已经形成独立外部证据 owner |
| status_transition | `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status` | 是，后续候选 | 状态迁移 guard 与 proposal status update 是稳定状态机 helper，但当前 approval review 与 sandbox gate 仍通过父级受控调用，适合在 sandbox 边界稳定后再拆 |
| proposal create orchestration | `create_runtime_ai_proposal` | 后续候选 | create flow 仍是事务编排 owner，负责 capability guard、source context、static check、proposal record、approval record 和 child helper 编排，应在 helper 残余收束后最后处理 |

---

## 下一候选判定

下一候选固定为:

```text
BE-001BL-01 runtime.mutation.ai_proposal.sandbox_trigger 单子叶等价基线
```

选择 `runtime.mutation.ai_proposal.sandbox_trigger` 的原因:

1. `ensure_ai_proposal_can_be_approved` 已经承担 approve 前的 config binding、static check 和 sandbox report verdict gate。
2. `load_sandbox_report_for_proposal` 持有 memory-first / disk fallback 的 sandbox report 读取边界。
3. `create_runtime_ai_proposal` 内部的 background sandbox task 负责 `RequestSandboxVerificationRequest`、`sandbox_verification::run_sandbox_verification`、`catch_unwind`、重试退避、`JoinHandle` panic 监控、`sandbox_report_url` 回写和 sandbox failed lifecycle。
4. `approval_persistence` 已完成 closeout，`persist_approval` 可以继续由父级受控连接；下一步拆 sandbox 时不需要横向 import `approval_persistence` sibling。
5. 本候选不改变 `AppState`、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 非目标边界

BE-001BK-01 不移动代码，也不创建 `sandbox_trigger.rs`。后续 BE-001BL-01 也只能建立等价基线，不得直接创建目标文件。

当前不得迁移或修改:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `RequestSandboxVerificationRequest`
- `run_sandbox_verification`
- `JoinHandle`
- `catch_unwind`
- `sandbox_report_url`
- `RuntimeApprovalLifecycleEntry`
- `persist_approval`
- `approval_review`
- `approval_persistence`
- `sandbox_trigger`
- `status_transition`
- proposal create orchestration
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回收或重拆 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review` 或 `approval_persistence` 已 closeout 子叶。

---

## 验证计划

本批 `no code movement`，只需要治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BK-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal` 第六轮父叶残余判断，`sandbox_trigger` 尚未建立基线也尚未抽离。不得宣称 `runtime.mutation.ai_proposal` 父级完成、sandbox_trigger 已迁移、status_transition 已迁移、proposal create orchestration 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `214-runtime.mutation.ai_proposal第六轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树继续将 `runtime.mutation.ai_proposal` 标记为 `stop_split: false`。
3. 下一候选固定为 BE-001BL-01 `runtime.mutation.ai_proposal.sandbox_trigger` 单子叶等价基线。
4. 本批不产生代码变更，也不回收已 closeout 子叶。
