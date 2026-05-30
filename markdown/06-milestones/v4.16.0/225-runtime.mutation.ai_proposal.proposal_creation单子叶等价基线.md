# v4.16.0 runtime.mutation.ai_proposal.proposal_creation 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BP-01  
> 基线: `224-runtime.mutation.ai_proposal第八轮父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`src/runtime/mutation/ai_proposal/source_governance_identity.rs`、`src/runtime/mutation/ai_proposal/event_lifecycle.rs`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`、`src/runtime/mutation/ai_proposal/sandbox_trigger.rs`、`src/runtime/mutation/ai_proposal/status_transition.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线。当前 `no code movement`，只冻结 `create_runtime_ai_proposal` 的输入输出、调用顺序、状态副作用、`approval_records -> ai_proposals` 锁顺序、父子通信规则和非目标边界。下一步只能进入 BE-001BP-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BP-01 AI proposal proposal_creation 单子叶等价基线 | 基线建立 |
| 规范矩阵 | 父子通信、public handler、状态机副作用、锁顺序、发布过渡保护 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.proposal_creation` | 新候选叶子 |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation` | 建立白箱节点 |

---

## 当前真实边界

当前代码仍在父级:

```text
src/runtime/mutation/ai_proposal.rs
```

后续候选目标文件仅可规划为:

```text
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

本批次禁止创建该文件，禁止迁移 `create_runtime_ai_proposal`，禁止回改任何已 closeout child。

---

## 白箱职责

`runtime.mutation.ai_proposal.proposal_creation` 只拥有 AI proposal create orchestration:

- `create_runtime_ai_proposal`
- `CreateRuntimeAiProposalRequest` 输入校验与 canonicalization 编排
- `RuntimeAiProposalRecord` 构造
- `RuntimeAiProposalSourceEvidence` 构造
- StaticCheckPassed / StaticCheckFailed 分支
- `RuntimeApprovalRecord` 与 `RuntimeApprovalLifecycleEntry` 自动创建分支
- `approval_records -> ai_proposals` 锁顺序下的 proposal transition persistence
- sandbox verification trigger 串联

本节点不拥有 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger`、`status_transition`、`AppState`、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 输入输出基线

| 项 | 来源 / 去向 | 类型 | 必须保持 |
| --- | --- | --- | --- |
| request | route facade | `Json<CreateRuntimeAiProposalRequest>` | 不改变 payload、target、source_kind、source_id、old/new value、model、prompt/evidence hash 语义 |
| user | auth middleware | `auth::UserId` | 只用于 scoped record 与 actor 绑定，不迁移 auth owner |
| state | `AppState` | shared state | 只使用既有 store、locks、runtime config、approval/proposal maps，不改变 owner |
| response | frontend/tests | `Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)>` | 不改变 JSON shape、status code 或 error mapping |
| proposal side effect | state + disk | `RuntimeAiProposalRecord` | `state.ai_proposals` 与 transition persistence 行为不变 |
| approval side effect | state + disk | `RuntimeApprovalRecord` | 仅 StaticCheckPassed 分支创建 approval，`APPROVAL_CREATED` lifecycle 不变 |

---

## 冻结调用顺序

`create_runtime_ai_proposal` 的等价顺序必须保持:

1. `validate_runtime_capability_guard`。
2. `proposal_only` policy 检查。
3. `validate_runtime_parameter_mutation_target`。
4. old/new value、`validate_ai_model_identity`、prompt/evidence `validate_hash_identity`。
5. actor required check 与 `normalize_actor_identity`。
6. `load_runtime_ai_proposal_source_context`。
7. `canonical_runtime_parameter_version` old/proposed version 构造。
8. `ai_proposal_static_check_result`。
9. `current_time_ms()`。
10. `runtime_ai_proposal_record_id`。
11. `runtime_ai_proposal_governance`。
12. 构造 `RuntimeAiProposalSourceEvidence`。
13. 构造 `RuntimeAiProposalRecord`。
14. `build_runtime_ai_proposal_event` 创建 created/static check lifecycle event。
15. `ai_proposal_lifecycle_entry` 写入 proposal lifecycle。
16. `append_parameter_mutation_events_to_run`。
17. StaticCheckPassed 分支创建 `RuntimeApprovalRecord`。
18. StaticCheckPassed 分支创建 `RuntimeApprovalLifecycleEntry`，event type 固定为 `APPROVAL_CREATED`。
19. StaticCheckPassed 分支执行 `persist_approval`。
20. StaticCheckPassed 分支按 `approval_records -> ai_proposals` 顺序写入 `state.approval_records` 与 proposal transition。
21. StaticCheckPassed 分支执行 `persist_runtime_ai_proposal_transition`，状态为 `StaticCheckPassed`。
22. StaticCheckPassed 分支执行 `spawn_ai_proposal_sandbox_verification`。
23. StaticCheckFailed 分支执行 `persist_runtime_ai_proposal_transition`，状态为 `StaticCheckFailed`。
24. 返回 `Ok(Json(record))`。

---

## 关键 public/helper 方法

| 方法 | 输入 | 输出 / 副作用 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_ai_proposal` | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeAiProposalRequest>` | `Json<RuntimeAiProposalRecord>` 与 proposal/approval side effect | `backend.runtime.routes.mutation` | 不得改变 route-facing signature、response shape、锁顺序或状态分支 |
| `validate_runtime_capability_guard` | capability context | capability validation | create handler | 不得绕过 proposal capability boundary |
| `validate_runtime_parameter_mutation_target` | target | mutation target validation | create handler | 不得放宽 target 解析 |
| `validate_ai_model_identity` | AI model identity | validation result | create handler via parent | 不得绕过 provider/model/model_version 必填 |
| `validate_hash_identity` | prompt/evidence hash、target、label | validation result | create handler via parent | 不得移除 hash identity guard |
| `load_runtime_ai_proposal_source_context` | `AppState`、user、source kind/id | source context | create handler via parent | 不得迁移 source owner |
| `ai_proposal_static_check_result` | request、parameter versions、source event count | static check result | create handler via parent | 不得改变 StaticCheckPassed / StaticCheckFailed 规则 |
| `runtime_ai_proposal_record_id` | request、created_at、parameter version、static check | proposal id | create handler via parent | 不得改变 digest contract |
| `runtime_ai_proposal_governance` | source context、model、hash evidence | governance metadata | create handler via parent | 不得丢失 governance evidence |
| `persist_runtime_ai_proposal_transition` | state/store/proposal/status/event | transition persistence | create handler via parent | 不得绕过 transition event |
| `spawn_ai_proposal_sandbox_verification` | state clone、proposal id、approval record | background sandbox side effect | create handler via parent | 不得改变 sandbox retry / failure lifecycle |

---

## 父子通信规则

后续若进入实际抽离，父子通信必须保持:

```text
src/runtime/mod.rs
  -> runtime.mutation.ai_proposal public handlers
src/runtime/mutation/ai_proposal.rs
  -> proposal_creation::create_runtime_ai_proposal
src/runtime/mutation/ai_proposal/proposal_creation.rs
  -> parent-owned imports / helpers via use super::*
```

- `proposal_creation` 只能经父级 `runtime.mutation.ai_proposal` 暴露给 route facade。
- child 固定 `use super::*`，不得横向 import `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 或 `status_transition` sibling。
- 已 closeout child 继续由父级受控 import 和调用，不得为本叶开启 sibling 直连。
- `AppState`、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均不迁移。
- 发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 非目标

- 不移动 Rust 代码。
- 不创建 `src/runtime/mutation/ai_proposal/proposal_creation.rs`。
- 不迁移 `create_runtime_ai_proposal`。
- 不拆分 approval record construction。
- 不拆分 lifecycle event append。
- 不拆分 sandbox trigger call。
- 不改变 `approval_records -> ai_proposals` lock order。
- 不改变 `state.approval_records` 或 `auth::scoped_key` 语义。
- 不回改 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 或 `status_transition`。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner 或 route facade。
- 不启动发布过渡，不提出横向连接。

---

## 回归保护

本基线批次只跑治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 BE-001BP 实际抽离必须补跑:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 下一步

下一步只能进入:

```text
BE-001BP-02 runtime.mutation.ai_proposal.proposal_creation 抽离方案
```

BE-001BP-02 只能规划目标文件、父级 child 声明、handler re-export、`use super::*`、迁移清单、非目标和回退点；不得直接迁移代码。

---

## 幻觉检查点

AI 声称 BE-001BP-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线，仍为 `no code movement`；目标文件尚未创建，`create_runtime_ai_proposal` 尚未迁移。不得宣称 proposal_creation 已抽离、已 closeout、AppState/schema/frontend caller 已改变、route facade 已改变、runtime persistence owner 已迁移、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.proposal_creation` 白箱候选节点。
3. 基线冻结 create handler、record 字段、调用顺序、状态副作用、锁顺序和非目标边界。
4. 本批次无代码移动。
5. 本批次验证通过后，后续才能进入 BE-001BP-02。
