# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AG-03  
> 基线: `136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md`、`137-runtime.mutation.parameter_mutation.transition_lifecycle抽离方案.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle` 实际抽离完成。下一步只能进入 BE-001AG-04 单叶 closeout，判断本叶是否还值得继续细拆。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AG-03 transition lifecycle 实际抽离 | 落地 |
| 规范矩阵 | 父子通信、可见性、状态机事务边界、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle` | 白箱节点落地 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle` | 记录真实文件 |

---

## 实际抽离结果

| 项 | 结果 |
| --- | --- |
| 新增文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级文件 | `src/runtime/mutation/parameter_mutation.rs` |
| 父级声明 | `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;` |
| 父级 handler 出口 | `pub(crate) use transition_lifecycle::{activate_runtime_parameter_mutation, rollback_runtime_parameter_mutation};` |
| 父级 boundary 出口 | `use transition_lifecycle::validate_runtime_parameter_mutation_boundary;` |
| runtime facade | `src/runtime/mod.rs` 保持 `pub(crate) use mutation_parameter_mutation` |
| route facade | `src/backend/runtime/routes/mutation.rs` 未改变 |

裸 `mod transition_lifecycle;` 会被 Rust 解析到 `src/runtime/mutation/transition_lifecycle.rs`，因此本批使用显式 `#[path = "parameter_mutation/transition_lifecycle.rs"]` 保持 BE-001AG-02 的嵌套目标路径。

---

## 已迁移函数

| 函数 | 目标可见性 | 说明 |
| --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `pub(super)` | 供父级 `create_runtime_parameter_mutation` 继续复用 |
| `resolve_runtime_parameter_mutation_boundary` | private | transition boundary resolution |
| `evaluate_runtime_parameter_mutation_safe_window` | private | transition safe window |
| `runtime_parameter_mutation_rollback_record_id` | private | rollback proposal id |
| `mutation_lifecycle_entry` | private | lifecycle event metadata |
| `persist_runtime_parameter_mutation_transition` | private | mutation record persist + scoped cache write |
| `auto_snapshot_on_activation` | private | activation side effect helper |
| `activate_runtime_parameter_mutation` | `pub(crate)` | activation route handler |
| `rollback_runtime_parameter_mutation` | `pub(crate)` | rollback route handler |

---

## 父级保留边界

`src/runtime/mutation/parameter_mutation.rs` 继续保留:

- `create_runtime_parameter_mutation`
- `list_runtime_parameter_mutations`
- `get_runtime_parameter_mutation_detail`
- `runtime_parameter_mutation_record_id`

`src/runtime/mutation.rs` 继续保留 shared helper:

- `canonical_runtime_parameter_version`
- `validate_runtime_parameter_mutation_target`
- `build_runtime_parameter_mutation_event`
- `append_parameter_mutation_events_to_run`
- `governance_with_parameter_version`
- `runtime_parameter_mutation_governance`
- `mutation_event_contract`
- `status_contract_value`
- `runtime_mode_from_events`

AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner、snapshot owner、route facade 和 release transition guard 均未迁移。

---

## HTTP / 状态机等价

| Route | Method | Handler |
| --- | --- | --- |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` |

request type 保持:

| Handler | Request type |
| --- | --- |
| `activate_runtime_parameter_mutation` | `ActivateRuntimeParameterMutationRequest` |
| `rollback_runtime_parameter_mutation` | `RollbackRuntimeParameterMutationRequest` |

状态机保持 `SafeWindowDenied`、`ActivationScheduled`、`Activated`、`ActivationFailed`、`RollbackScheduled`、`RolledBack`、`RollbackFailed`，safe window reason code、run event append、metrics、snapshot side effect 和 response schema 未改变。

---

## 验证记录

| 命令 | 结果 |
| --- | --- |
| `cargo fmt` | PASS |
| `cargo check -p quantpilot` | PASS |
| `cargo test -p quantpilot --test api_mutation` | PASS, 9 tests |
| `cargo test -p quantpilot --test api_ai_proposal` | PASS, 4 tests |
| `cargo test -p quantpilot --test api_evidence_contract` | PASS, 2 tests |
| `cargo test -p quantpilot --test api_run` | PASS, 14 tests |

提交前还必须通过:

- `cargo fmt --check`
- `cargo test --no-run`
- `tools\check-utf8.ps1`
- `tools\check-matrix-governance.ps1`
- `tools\check-full-feature-tree.ps1`
- `git diff --check`

---

## 下一步

下一批进入 BE-001AG-04 `runtime.mutation.parameter_mutation.transition_lifecycle` 单叶 closeout。closeout 只判断本叶是否还值得继续细拆；不得顺手迁移 proposal record、safe window boundary 子叶、auto snapshot side effect 子叶、AI proposal、approval review、schema、frontend caller 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001AG-03 完成时，必须说明 transition lifecycle 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`，但尚未完成单叶 closeout。不得宣称 `runtime.mutation.parameter_mutation` 父叶完成、proposal create/list/detail 已迁移、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 进入模块树和全量树。
2. `138-runtime.mutation.parameter_mutation.transition_lifecycle抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 父级 `src/runtime/mutation/parameter_mutation.rs` 只保留 proposal record / create / list / detail，并通过 child re-export 维持 activation / rollback handler 出口。
4. route facade、runtime facade、AppState、schema、frontend caller 和发布过渡保护未改变。
5. 本批验证通过后，后续才能进入 BE-001AG-04 单叶 closeout。
