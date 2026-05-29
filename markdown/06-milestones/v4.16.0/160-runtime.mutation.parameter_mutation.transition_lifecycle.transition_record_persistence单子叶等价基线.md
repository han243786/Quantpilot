# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AP-01  
> 基准: `159-runtime.mutation.parameter_mutation.transition_lifecycle第四轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`、`src/frontend_api_types.rs`、`src/runtime_persistence.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 单子叶等价基线已建立。下一步只能进入 BE-001AP-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AP-01 transition_record_persistence 等价基线 | 基线 |
| 规范矩阵 | 三档执行、父子通信、stop_split、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 新候选白箱 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` | 冻结输入输出 |

---

## 真实文件

- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/frontend_api_types.rs`
- `src/runtime_persistence.rs`
- `tests/api_mutation.rs`

当前只建立等价基线，目标文件尚未创建；ASCII guard: `target file not created`。不得创建 `src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs`。

---

## 白箱边界

| 项 | 当前基线 |
| --- | --- |
| 候选方法 | `mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition` |
| 输入 | `RuntimeParameterMutationStatus`、`FrontendRuntimeEvent`、`sequence_no`、message、`AppState`、`auth::UserId`、`RuntimeParameterMutationRecord` |
| 调用方 | `activation_flow::activate_runtime_parameter_mutation`、`rollback_flow::rollback_runtime_parameter_mutation` 经父级 `transition_lifecycle` helper |
| 输出 | `RuntimeParameterMutationLifecycleEntry`、persisted mutation record、`state.parameter_mutations` in-memory index |
| 返回 | sync lifecycle entry；async persistence `Result<(), (StatusCode, String)>` |
| 当前 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 父级 |

---

## 输入基线

| 输入 | 来源 | 约束 |
| --- | --- | --- |
| `RuntimeParameterMutationStatus` | activation / rollback state machine | 只能使用既有 status，不新增或重命名状态 |
| `FrontendRuntimeEvent` | `build_runtime_parameter_mutation_event` | `event_id` 与 `event_time_ms` 必须原样进入 lifecycle entry |
| `sequence_no` | activation / rollback caller | 不重新计算，不改变 schedule / activation / rollback 的 +1 语义 |
| message | caller-provided literal 或 safe-window message | 不统一改写，不翻译，不裁剪 |
| `AppState` | route handler state | 只读取 `mutation_store_dir` 并写 `parameter_mutations` |
| `auth::UserId` | route auth | 只用于 `auth::scoped_key(user_id, &record.proposal_id)` |
| `RuntimeParameterMutationRecord` | transition state machine | persist clone，不能改变 record schema 或 field 值 |

---

## 输出基线

| 输出 | 当前语义 |
| --- | --- |
| lifecycle status | 与传入 `RuntimeParameterMutationStatus` 完全一致 |
| lifecycle reason_code | 来自 `mutation_event_contract(status)` 的第二返回值 |
| lifecycle event_id | `event.event_id.clone()` |
| lifecycle sequence_no | caller 传入值 |
| lifecycle occurred_at_ms | `event.event_time_ms` |
| lifecycle message | caller 传入 message |
| persisted record | `persist_runtime_parameter_mutation_record(state.mutation_store_dir.as_ref(), record).await` |
| persistence error | `map_err(io_error)`，不吞错 |
| in-memory index | `state.parameter_mutations.write().await.insert(auth::scoped_key(user_id, &record.proposal_id), record.clone())` |

---

## 调用点基线

| 调用点 | 文件 | 语义 |
| --- | --- | --- |
| activation safe-window denial | `activation_flow.rs` | push `SafeWindowDenied` lifecycle entry 后 append event，再 persist transition |
| activation scheduled | `activation_flow.rs` | push `ActivationScheduled` lifecycle entry 后继续构造 event batch |
| activation applied | `activation_flow.rs` | push `Activated` lifecycle entry，与 active parameter version 一起 append |
| activation failed | `activation_flow.rs` | push `ActivationFailed` lifecycle entry，保留 source governance |
| rollback safe-window denial | `rollback_flow.rs` | push `SafeWindowDenied` lifecycle entry 后 append event，再 persist transition |
| rollback scheduled | `rollback_flow.rs` | push `RollbackScheduled` lifecycle entry 后继续构造 event batch |
| rollback applied | `rollback_flow.rs` | push `RolledBack` lifecycle entry，与 rollback parameter version 一起 append |
| rollback failed | `rollback_flow.rs` | push `RollbackFailed` lifecycle entry，保留 source governance |

---

## 时序基线

```text
activation_flow.rs / rollback_flow.rs
  -> build_runtime_parameter_mutation_event
  -> mutation_lifecycle_entry
     -> mutation_event_contract(status)
     -> RuntimeParameterMutationLifecycleEntry
  -> append_parameter_mutation_events_to_run
  -> persist_runtime_parameter_mutation_transition
     -> persist_runtime_parameter_mutation_record
     -> state.parameter_mutations insert
```

必须保持 event append 与 transition persistence 的当前顺序。BE-001AP-01 不重排事务，不改锁顺序，不改变 persistence error 的传播方式。

---

## 父子通信规则

`transition_record_persistence` 只能作为 `transition_lifecycle` 的 child 被父级管理。后续若实际抽离，`activation_flow` 与 `rollback_flow` 仍只能经父级 `transition_lifecycle` 的受控 helper 调用，不得让 route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 或发布过渡连接直接依赖本叶。

ASCII guard: `release transition guard`。

---

## 排除边界

- 不迁移 Rust 代码。
- 不创建目标文件。
- 不迁移 `runtime_parameter_mutation_rollback_record_id`。
- 不迁移 `activate_runtime_parameter_mutation` 或 `rollback_runtime_parameter_mutation`。
- 不迁移 `validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary`、`evaluate_runtime_parameter_mutation_safe_window`。
- 不迁移 `auto_snapshot_on_activation`。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。

---

## 验证记录

| 命令 | 结果 |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo check -p quantpilot` | PASS |
| `cargo test --no-run` | PASS |
| `cargo test -p quantpilot --test api_mutation` | PASS |
| `cargo test -p quantpilot --test api_ai_proposal` | PASS |
| `cargo test -p quantpilot --test api_evidence_contract` | PASS |
| `cargo test -p quantpilot --test api_run` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1` | PASS |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1` | PASS |
| `git diff --check` | PASS |

---

## 下一步

下一批进入 BE-001AP-02 `runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence` 抽离方案。只能固定目标文件、父级 path attribute、helper visibility、调用面和回退点；不得移动代码。

---

## 幻觉检查点

AI 声称 BE-001AP-01 完成时，必须说明当前只是等价基线，`mutation_lifecycle_entry` 与 `persist_runtime_parameter_mutation_transition` 仍留在 `transition_lifecycle` 父级，目标文件未创建，下一步只能进入 BE-001AP-02 抽离方案。不得宣称 transition persistence helper 已抽离、rollback id 已迁移、parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `160-runtime.mutation.parameter_mutation.transition_lifecycle.transition_record_persistence单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树记录 `transition_record_persistence` 已建立等价基线，但代码未移动。
3. 全量树记录 BE-001AP-01 并把下一步固定为 BE-001AP-02 抽离方案。
4. 本批无代码移动。
5. 本批验证通过后，后续才能进入 BE-001AP-02。
