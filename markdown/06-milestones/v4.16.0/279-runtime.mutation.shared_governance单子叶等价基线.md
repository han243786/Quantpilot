# v4.16.0 runtime.mutation.shared_governance 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CL-01  
> 基准: `278-backend.runtime第三轮父叶残余判断.md`、`180-runtime.mutation.parameter_mutation第三轮父叶残余判断.md`、`229-runtime.mutation.ai_proposal父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.mutation.shared_governance`  
> 模块树坐标: `root.backend.runtime.runtime.mutation.shared_governance`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、禁止跳步、共享 helper 迁移边界、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.mutation.shared_governance` | 新增 planned 子叶坐标 |
| 模块树 | `runtime.mutation.shared_governance` | 白箱登记 |

---

## 当前真实结构

已 closeout sibling / 父级:

- `runtime.mutation.parameter_mutation stop_split: true`
- `runtime.mutation.ai_proposal stop_split: true`
- `backend.runtime.routes.mutation stop_split: true`
- `backend.runtime stop_split: false`

本批冻结的父级残余仍在:

```text
src/runtime/mutation.rs
```

planned child 文件尚未创建。BE-001CL-01 只建立等价基线，不创建 `src/runtime/mutation/shared_governance.rs`，不迁移 helper。

---

## 白箱边界

| helper | 输入 | 状态读取 / 依赖 | 输出 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `canonical_runtime_parameter_version` | `RuntimeParameterMutationTarget`、`Value` | `canonical_json_sha256_digest`、`json!` canonical payload | `sha256:{digest}` | 不得改变 digest input、prefix 或 error mapping |
| `validate_runtime_parameter_mutation_target` | `RuntimeParameterMutationTarget` | `SUPPORTED_FRONTEND_MODULE_KEYS` | `Ok(())` 或 `json_bad_request` | 不得放宽 node/module/path required gate 或 capability gate |
| `runtime_mode_from_events` | `&[FrontendRuntimeEvent]` | event envelope mode | runtime mode string, default `paper` | 不得改变 default mode |
| `status_contract_value` | `RuntimeParameterMutationStatus` | status enum | contract status string | 不得改 status contract spelling |
| `mutation_event_contract` | `RuntimeParameterMutationStatus` | status enum | event type / reason code pair | 不得改 event type 或 reason code |
| `build_runtime_parameter_mutation_event` | `RuntimeParameterMutationRecord`、status、event time | `mutation_event_contract`、`status_contract_value` | `FrontendRuntimeEvent` | 不得改变 payload fields、severity mapping、event id 或 envelope default |
| `append_parameter_mutation_events_to_run` | `AppState`、`UserId`、source id、events、optional active parameter version | `load_run_record_from_state`、`attach_runtime_event_envelope`、`validate_runtime_event_envelopes`、in-memory run store、optional `persist_run_record` | `Ok(())` 或 error | 不得改变 sequence、mode、governance envelope、persistence condition 或 lock owner |
| `runtime_parameter_mutation_governance` | source governance、old parameter version、proposed parameter version | governance snapshot fields | `RuntimeParameterMutationGovernance` | 不得改变 capability/deployment/strategy/permission boundary mapping |
| `governance_with_parameter_version` | governance snapshot、parameter version | clone of existing snapshot | `RuntimeGovernanceSnapshot` with replaced parameter version | 不得改变其他 governance fields |

---

## 调用方基线

| 调用方文件 | 当前调用 | 依赖意义 | 禁止事项 |
| --- | --- | --- | --- |
| `src/runtime/mutation/parameter_mutation/proposal_creation.rs` | target validation、canonical version、mutation governance、event build、governance override、event append | parameter mutation create lifecycle | 不得把 proposal creation 拆回 root 或横连 AI proposal |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` | event build、governance override、event append | activation lifecycle event / audit / parameter version write | 不得改 activation state machine |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` | mutation governance、event build、governance override、event append | rollback lifecycle event / audit / parameter version write | 不得改 rollback state machine |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs` | `mutation_event_contract` reason code | lifecycle entry reason code | 不得改变 transition persistence contract |
| `src/runtime/mutation/ai_proposal/proposal_creation.rs` | target validation、canonical version、governance override、event append | AI proposal creates auditable source run event | 不得让 AI proposal child 直接横连 parameter mutation child |

---

## 现有等价证据

当前已有自动化覆盖:

- `tests/api_mutation.rs::runtime_parameter_mutation_creates_persisted_proposal_and_key_event`
- `tests/api_mutation.rs::runtime_parameter_mutation_activation_uses_explicit_boundary_versions_and_reports`
- `tests/api_mutation.rs::runtime_parameter_mutation_rolls_back_to_ledger_backed_prior_version`
- `tests/api_mutation.rs::runtime_parameter_mutation_safe_window_denial_is_audited_without_activation`
- `tests/api_ai_proposal.rs::runtime_ai_proposal_creates_static_checked_record_and_key_events`
- `tests/api_ai_proposal.rs::runtime_ai_proposal_static_check_failed_candidate_is_auditable`

因此 BE-001CL-02 抽离方案不需要先补 endpoint smoke，但必须把 `api_mutation` 与 `api_ai_proposal` 作为实际抽离前后的硬门禁。

---

## 父子通信规则

`runtime.mutation.shared_governance` 后续若实际抽离，只能作为 `backend.runtime` 下的 mutation shared helper child。通信路径必须保持父级中介:

```text
runtime.mutation.parameter_mutation / runtime.mutation.ai_proposal
  -> src/runtime/mod.rs controlled helper surface
  -> runtime.mutation.shared_governance
```

开发者未明确进入发布版本过渡前，不得让 parameter mutation child、AI proposal child、route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连该 planned child。

---

## 明确排除

- 不创建 `src/runtime/mutation/shared_governance.rs`。
- 不迁移 `canonical_runtime_parameter_version`、`validate_runtime_parameter_mutation_target`、`runtime_mode_from_events`、`status_contract_value`、`mutation_event_contract`、`build_runtime_parameter_mutation_event`、`append_parameter_mutation_events_to_run`、`runtime_parameter_mutation_governance` 或 `governance_with_parameter_version`。
- 不处理 `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`、`MergeRecordsResponse`、`MergeRecordEntry`、`DiscardRuntimeArtifactResponse`、`RunInProgressGuard` 或 replay / experiment limit。
- 不回改 `runtime.mutation.parameter_mutation`、`runtime.mutation.ai_proposal`、`runtime.report_ops`、`runtime.evidence_health` 或 `backend.runtime.routes.mutation` closed child。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CL-02 runtime.mutation.shared_governance 抽离方案
```

BE-001CL-02 只能决定 planned child 文件、父级声明 / re-export、允许迁移清单、验证命令和回退点；不得直接宣称 helper 已迁移。

---

## 幻觉检查点

AI 声称 BE-001CL-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. planned child 文件 `src/runtime/mutation/shared_governance.rs` 尚未创建。
3. 9 个 shared governance helper 仍在 `src/runtime/mutation.rs`。
4. `api_mutation` 与 `api_ai_proposal` 是本叶等价硬门禁。
5. 下一步只能进入 BE-001CL-02 抽离方案。
6. query DTO、run guard、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 和 release transition guard 均未迁移。

不得宣称 helper 已抽离、`backend.runtime` 已完成、parent support 已整体抽离、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `279-runtime.mutation.shared_governance单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.shared_governance` planned 子叶白箱坐标，但不登记不存在的真实文件路径。
3. 治理门禁能阻止跳过 BE-001CL-02 直接创建 child 文件或迁移 helper。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
