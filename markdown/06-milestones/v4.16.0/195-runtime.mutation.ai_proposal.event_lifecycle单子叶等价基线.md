# v4.16.0 runtime.mutation.ai_proposal.event_lifecycle 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BD-01  
> 基线: `194-runtime.mutation.ai_proposal第二轮父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`src/runtime/mutation/ai_proposal/source_governance_identity.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线。当前只冻结 AI proposal event contract、event payload、lifecycle entry 与 proposal transition persistence；本批 `no code movement`。下一步只能进入 BE-001BD-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BD-01 event_lifecycle 等价基线 | 新增基线 |
| 规范矩阵 | 父子通信、event contract 稳定性、lifecycle sequence、transition persistence、非目标边界 | 冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.event_lifecycle` | 新增白箱节点 |
| 模块树 | `runtime.mutation.ai_proposal.event_lifecycle` | 建立单子叶基线 |

---

## 基线范围

`runtime.mutation.ai_proposal.event_lifecycle` 是 `runtime.mutation.ai_proposal` 父叶在 source_governance_identity closeout 后的下一候选。它只冻结 AI proposal / approval 事务中的 event 与 lifecycle 写入层:

- `ai_proposal_event_contract`
- `build_runtime_ai_proposal_event`
- `ai_proposal_lifecycle_entry`
- `persist_runtime_ai_proposal_transition`

本批不创建 child 文件，不移动 helper，不改变 public handler。

---

## 输入输出冻结

| 类型 | 内容 | 等价约束 |
| --- | --- | --- |
| 输入 | `RuntimeAiProposalStatus` | event type、reason code 映射不变 |
| 输入 | `RuntimeAiProposalRecord`、status、event timestamp | event id、payload、severity、summary 语义不变 |
| 输入 | event、sequence number、message | lifecycle entry 字段映射不变 |
| 输入 | `AppState`、`auth::UserId`、proposal record | proposal record persistence 与 scoped in-memory update 不变 |
| 输出 | `(&'static str, &'static str)` | event type / reason code pair 不变 |
| 输出 | `FrontendRuntimeEvent` | payload schema 与 default envelope 不变 |
| 输出 | `RuntimeAiProposalLifecycleEntry` | status、event_id、sequence_no、reason_code 不变 |
| 输出 | persisted proposal record | disk write 与 `state.ai_proposals` scoped key 写入不变 |

---

## helper 细节冻结

### `ai_proposal_event_contract`

必须保持 status 映射:

- `Submitted` / `Draft` -> `AIProposalCreated` / `AI_PROPOSAL_CREATED`
- `Denied` -> `AIProposalDenied` / `AI_PROPOSAL_DENIED`
- `StaticCheckPassed` -> `AIProposalStaticCheckPassed` / `AI_PROPOSAL_STATIC_CHECK_PASSED`
- `StaticCheckFailed` -> `AIProposalStaticCheckFailed` / `AI_PROPOSAL_STATIC_CHECK_FAILED`
- `Expired` -> `AIProposalDenied` / `AI_PROPOSAL_EXPIRED`
- `Approved` -> `AIProposalApproved` / `AI_PROPOSAL_APPROVED`

### `build_runtime_ai_proposal_event`

必须保持:

- event id 格式 `event_{ai_proposal_id}_{reason_code}_{event_time_ms}`
- `source_id` 使用 `record.target.module_key`
- `node_id` 使用 `record.target.node_id`
- denied / static check failed severity 为 `Warn`
- 其他 status severity 为 `Info`
- payload 包含 proposal id、status、reason_code、source evidence、target、old/proposed parameter version、denial reason、static check、model、prompt/evidence hash、actor、reason、governance、config domain binding
- `envelope` 保持 `RuntimeEventEnvelope::default()`

### `ai_proposal_lifecycle_entry`

必须保持:

- `reason_code` 来自 `ai_proposal_event_contract(status)`
- `event_id` 来自 event
- `sequence_no` 使用调用方传入值
- `occurred_at_ms` 使用 event time
- `message` 只按调用方传入内容记录

### `persist_runtime_ai_proposal_transition`

必须保持:

- 先调用 `persist_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), record)`
- IO error 继续经 `io_error` 映射
- in-memory key 继续使用 `auth::scoped_key(user_id, &record.ai_proposal_id)`
- `state.ai_proposals` owner 不迁移

---

## 非目标边界

BE-001BD-01 不得移动或修改:

- `create_runtime_ai_proposal`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `load_runtime_ai_proposal_for_user`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得把本批解释为 record_query、approval_review、approval_persistence、sandbox_trigger 或 status_transition 已拆分。

---

## 验证计划

本批为 `no code movement`，只需文档治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 BE-001BD-02 抽离方案必须明确目标 child 文件、父级 child 声明、`pub(super)` visibility、迁移清单和回退点。BE-001BD-03 实际抽离必须补齐 Rust 编译与 API 回归测试。

---

## 幻觉检查点

AI 声称 BE-001BD-01 完成时，必须说明本批只建立 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线，并且为 `no code movement`。不得宣称 event_lifecycle helper 已迁移、目标文件已创建、record query / approval review 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.event_lifecycle` 白箱节点。
3. 本批只冻结 event contract、event payload、lifecycle entry、transition persistence 与非目标边界。
4. 下一步只能进入 BE-001BD-02 抽离方案。
