# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AG-01  
> 基准: `135-runtime.mutation.parameter_mutation单叶closeout.md`、`134-runtime.mutation.parameter_mutation抽离记录.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线。当前只冻结 activation / rollback lifecycle、safe window、boundary resolution、transition persistence、auto snapshot side effect、run event append 和相关回归证据；本批 `no code movement`。下一步只能进入 BE-001AG-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AG-01 transition lifecycle 单子叶等价基线 | 扩展 |
| 规范矩阵 | 父子通信、事务状态、safe window、event append、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle` | 新增白箱节点 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle` | 建立单子叶基线 |

---

## 白箱边界

`runtime.mutation.parameter_mutation.transition_lifecycle` 是 `runtime.mutation.parameter_mutation` 下的 transition 子流，冻结两条 public transition handler:

- `activate_runtime_parameter_mutation`
- `rollback_runtime_parameter_mutation`

本节点只负责从已存在的 parameter mutation proposal 进入 activation / rollback transition，不拥有 proposal create/list/detail，不拥有 AI proposal、approval review、schema、frontend caller、AppState owner、runtime persistence owner 或发布过渡连接。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle` |
| 父模块 | `runtime.mutation.parameter_mutation` |
| 路由入口 | `backend.runtime.routes.mutation` |
| 当前 owner 文件 | `src/runtime/mutation/parameter_mutation.rs` |
| 父级 shared owner 文件 | `src/runtime/mutation.rs` |
| 目标文件路径 | BE-001AG-02 决定，候选为 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| route facade | `src/backend/runtime/routes/mutation.rs` |
| runtime facade | `src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.parameter_mutation.transition_lifecycle` |
| 测试证据 | `tests/api_mutation.rs`、`tests/api_ai_proposal.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs` |
| 下一批次 | BE-001AG-02 抽离方案 |

---

## 输入输出冻结

| Handler | 输入 | 输出 | 必须保持 |
| --- | --- | --- | --- |
| `activate_runtime_parameter_mutation` | `auth::UserId`、`AppState`、proposal id、`ActivateRuntimeParameterMutationRequest` | `RuntimeParameterMutationRecord` 或 safe window denial error | status transition、safe window audit、activation event、run record append、auto snapshot side effect |
| `rollback_runtime_parameter_mutation` | `auth::UserId`、`AppState`、proposal id、`RollbackRuntimeParameterMutationRequest` | `RuntimeParameterMutationRecord` 或 safe window / rollback target error | rollback ledger lookup、rollback event、run record append、target parameter version |

HTTP route 必须保持:

| Route | Method | Handler |
| --- | --- | --- |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` |

---

## 状态机冻结

| Transition | 起点 | 终点 / 分支 | 事件 |
| --- | --- | --- | --- |
| activation safe window denied | `Proposed` 或 `SafeWindowDenied` | `SafeWindowDenied` | `ParameterMutationSafeWindowDenied` / `PARAMETER_MUTATION_SAFE_WINDOW_DENIED` |
| activation scheduled | `Proposed` 或 `SafeWindowDenied` | `ActivationScheduled` | `ParameterMutationActivationScheduled` / `PARAMETER_MUTATION_ACTIVATION_SCHEDULED` |
| activation applied | `ActivationScheduled` with `next_cycle_start` | `Activated` | `ParameterMutationActivated` / `PARAMETER_MUTATION_ACTIVATED` |
| activation failed | resolved sequence already behind schedule event | `ActivationFailed` | `ParameterMutationActivationFailed` / `PARAMETER_MUTATION_ACTIVATION_FAILED` |
| rollback scheduled | `Activated` | `RollbackScheduled` | `ParameterMutationRollbackScheduled` / `PARAMETER_MUTATION_ROLLBACK_SCHEDULED` |
| rollback applied | `RollbackScheduled` with `next_cycle_start` | `RolledBack` | `ParameterMutationRolledBack` / `PARAMETER_MUTATION_ROLLED_BACK` |
| rollback failed | resolved rollback sequence already behind schedule event | `RollbackFailed` | `ParameterMutationRollbackFailed` / `PARAMETER_MUTATION_ROLLBACK_FAILED` |

safe window helper 必须继续覆盖:

- runtime status 非 `paused` / `idle` / `stopped` / `ready` 时拒绝。
- open order、outstanding risk violation、stale data、exposure limit、cooldown remaining 均按现有 reason code 返回。
- `retryable` 与 `retry_after_ms` 语义不变。

boundary helper 必须继续覆盖:

- 拒绝 `immediate`。
- 接受 `next_cycle_start`、`manual_pause`、`sequence_cursor` 和 `sequence_cursor:<u64>`。
- `next_cycle_start` 解析为 `current_sequence_no + 2`。
- `manual_pause` 不设置 resolved sequence。

---

## 关键 helper 冻结

本节点基线冻结下列 helper，但本批不移动:

| Helper | 当前 owner | 冻结点 |
| --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `src/runtime/mutation/parameter_mutation.rs` | activation boundary validation |
| `resolve_runtime_parameter_mutation_boundary` | `src/runtime/mutation/parameter_mutation.rs` | current sequence -> resolved boundary |
| `evaluate_runtime_parameter_mutation_safe_window` | `src/runtime/mutation/parameter_mutation.rs` | safe window decision |
| `mutation_lifecycle_entry` | `src/runtime/mutation/parameter_mutation.rs` | lifecycle event metadata |
| `persist_runtime_parameter_mutation_transition` | `src/runtime/mutation/parameter_mutation.rs` | mutation record persist + scoped cache write |
| `runtime_parameter_mutation_rollback_record_id` | `src/runtime/mutation/parameter_mutation.rs` | rollback proposal id |
| `auto_snapshot_on_activation` | `src/runtime/mutation/parameter_mutation.rs` | config generation, snapshot persist, in-memory snapshot insert |

本节点继续依赖父级 shared helper，不得在 BE-001AG-01 私有化:

- `build_runtime_parameter_mutation_event`
- `append_parameter_mutation_events_to_run`
- `governance_with_parameter_version`
- `runtime_parameter_mutation_governance`
- `mutation_event_contract`
- `status_contract_value`
- `runtime_mode_from_events`

---

## 状态 owner 与副作用

| Owner | 当前位置 | 约束 |
| --- | --- | --- |
| mutation ledger store | `state.mutation_store_dir` / `src/runtime_persistence.rs` | 不迁移 file layout 或 persistence helper |
| run record append | `append_parameter_mutation_events_to_run` | 仍由父级 shared helper 执行，不改变 active parameter version 写回 |
| metrics | `state.evidence_metrics` | activation scheduled/applied/failed、safe window denied、rollback attempt/scheduled/applied/failed 计数不变 |
| config generation | `state.config_generation`、`state.config_generation_history` | 只由 `auto_snapshot_on_activation` side effect 触达 |
| snapshot store | `state.snapshot_store_dir`、`state.snapshots` | 不迁移 snapshot owner |
| scoped cache | `state.parameter_mutations` | `auth::scoped_key(user_id, proposal_id)` 不变 |

---

## 排除边界

- 不移动 `create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail`。
- 不移动 proposal record id helper: `runtime_parameter_mutation_record_id`。
- 不迁移 AI proposal handler、approval review handler、shared persistence/governance helper、AppState、schema、frontend caller、tests 或 fixture。
- 不改 `src/backend/runtime/routes/mutation.rs` route facade。
- 不启动发布过渡，不提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 父子通信规则

```text
backend.runtime.routes.mutation
  -> src/runtime/mod.rs re-export
  -> runtime.mutation.parameter_mutation
  -> transition_lifecycle candidate
  -> parent shared helper / AppState / runtime persistence / run evidence
```

BE-001AG-01 只建立基线，不能创建 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`。BE-001AG-02 才能决定目标路径、迁移清单和回退点。

---

## 回归保护

| 证据 | 覆盖范围 |
| --- | --- |
| `cargo fmt --check` | Rust 格式不漂移 |
| `cargo check -p quantpilot` | type / visibility 不漂移 |
| `cargo test --no-run` | 测试编译不漂移 |
| `cargo test -p quantpilot --test api_mutation` | activation / rollback lifecycle 主证据 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接 shared helper 不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence / report side effect 不漂移 |
| `cargo test -p quantpilot --test api_run` | run record append 不漂移 |
| `tools\check-utf8.ps1` | UTF-8 |
| `tools\check-matrix-governance.ps1` | 三矩阵登记 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 |
| `git diff --check` | whitespace |

---

## 下一步

下一批进入 BE-001AG-02 `runtime.mutation.parameter_mutation.transition_lifecycle` 抽离方案。该方案必须先决定:

| 决策点 | 候选 | 暂定约束 |
| --- | --- | --- |
| 目标文件路径 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` | 由 `parameter_mutation.rs` 私有 `mod transition_lifecycle` 承接 |
| re-export 面 | `pub(crate) use transition_lifecycle::{activate_runtime_parameter_mutation, rollback_runtime_parameter_mutation};` | 只暴露两条 transition handler |
| 是否迁移 safe window / boundary helper | 随 transition lifecycle 移动 | 若保留父级会让本叶继续过厚 |
| 是否迁移 `auto_snapshot_on_activation` | 随 activation 移动 | 不迁移 snapshot owner，只迁移调用 helper |
| 是否迁移 `runtime_parameter_mutation_rollback_record_id` | 随 rollback 移动 | create id helper 留在 proposal record 流 |
| shared helper | 继续留父级 | `append_parameter_mutation_events_to_run` 和 event/governance helper 不能私有化 |

---

## 幻觉检查点

AI 声称 BE-001AG-01 完成时，必须说明本批只建立 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线，仍为 `no code movement`。不得宣称 transition lifecycle 已抽离、目标文件已创建、safe window 已迁移、auto snapshot owner 已迁移、AI proposal/approval 已拆分、AppState 或 schema 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.parameter_mutation.transition_lifecycle` 白箱节点。
3. 基线冻结 activation / rollback lifecycle、safe window、boundary、transition persistence、auto snapshot side effect、run event append 和排除边界。
4. 治理门禁能发现 `no code movement`、BE-001AG-02 下一步、关键 handler/helper、route、state owner、发布过渡保护和测试证据。
5. 本批验证通过后，后续才能进入 BE-001AG-02 抽离方案。
