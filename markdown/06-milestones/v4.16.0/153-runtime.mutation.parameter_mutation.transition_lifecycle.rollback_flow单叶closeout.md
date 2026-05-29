# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AL-04  
> 基线: `150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md`、`151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md`、`152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单叶 closeout 完成，设置 `stop_split: true`。下一步只能进入 BE-001AM-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AL-04 rollback_flow 单叶 closeout | 收口 |
| 规范矩阵 | 三档执行、父子通信、stop_split、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 关闭当前递归叶 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 设置 `stop_split: true` |

---

## closeout 结论

| 项 | 结论 |
| --- | --- |
| 等价状态 | BE-001AL-03 实际抽离等价成立 |
| 真实文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` |
| 父级调用 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 通过 path-attributed child 和 handler re-export 调用 |
| 上层调用 | `src/runtime/mutation/parameter_mutation.rs` 仍只经 `transition_lifecycle::rollback_runtime_parameter_mutation` 受控出口 |
| 输入类型 | `RollbackRuntimeParameterMutationRequest` |
| 输出类型 | `RuntimeParameterMutationRecord` |
| stop_split | `true` |
| 下一步 | BE-001AM-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断 |

---

## 为什么停止细拆

本叶只拥有一个稳定 public handler:

- `rollback_runtime_parameter_mutation`

它内部的 capability guard、activated-only gate、source run load、ledger lookup、unknown target / no-op protection、safe-window denied、RollbackScheduled、RolledBack、RollbackFailed、`append_parameter_mutation_events_to_run`、rollback metrics 和 `persist_runtime_parameter_mutation_transition` 是同一条 rollback transaction 的连续状态机。继续拆成 ledger lookup branch、safe-window branch、scheduled branch、rolled_back branch 或 failed branch 会增加父级 import 与测试定位成本，但不会形成新的稳定 owner。

因此本叶 closeout 为 `stop_split: true`。后续要继续推进 `transition_lifecycle`，应回到父级残余判断，优先评估 parent-owned rollback id / lifecycle / transition persistence helper 与 `activation_snapshot_side_effect`，不能从本叶继续细拆。

---

## 等价保护

必须继续保持:

- capability denied 仍返回 `parameter_mutation_boundary_violation`，不写 rollback record。
- only activated gate 仍只允许 `Activated` proposal 进入 rollback。
- ledger lookup 仍通过 `list_runtime_parameter_mutation_records` 查找 rollback target。
- unknown target 仍返回 `parameter_mutation_rollback_unknown_version`。
- no-op target 仍返回 `parameter_mutation_rollback_noop`。
- safe-window denied 仍写 `SafeWindowDenied` lifecycle/event、denied metric，并返回 `parameter_mutation_safe_window_denied`。
- scheduled 分支仍写 `RollbackScheduled` lifecycle/event 和 scheduled metric。
- next cycle rollback 仍写 `RolledBack`、active parameter version、rollback lifecycle/event 和 applied metric。
- failed boundary 仍写 `RollbackFailed`、failure reason、failed lifecycle/event 和 failed metric。
- append 顺序仍为 `append_parameter_mutation_events_to_run` -> `persist_runtime_parameter_mutation_transition`。
- response schema、AppState、schema、frontend caller、route facade、AI proposal、approval review 和 release transition guard 不变。

---

## 父子通信规则

```text
parameter_mutation.rs
  -> transition_lifecycle::rollback_runtime_parameter_mutation
transition_lifecycle.rs
  -> rollback_flow::rollback_runtime_parameter_mutation
rollback_flow.rs
  -> parent-owned helpers via use super::*
```

`rollback_flow` 不得被 route facade、AI proposal、approval review、AppState、schema、frontend caller 或发布过渡连接直接依赖。发布过渡仍未启动，ASCII guard: `release transition guard`。

---

## 父级残余回流

本叶 closeout 后，`transition_lifecycle` 父级仍保持 `stop_split: false`，因为以下 parent-owned helper 尚未完成残余判断:

- `runtime_parameter_mutation_rollback_record_id`
- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`
- `auto_snapshot_on_activation`

这些 helper 同时影响 activation / rollback 或 snapshot side effect，不属于 `rollback_flow` 本批继续细拆范围。下一步只能进入 BE-001AM-01 父叶残余判断。

---

## 非目标

- 不移动 Rust 代码。
- 不拆 rollback handler 内部 branch。
- 不迁移 `runtime_parameter_mutation_rollback_record_id`。
- 不迁移 `mutation_lifecycle_entry`。
- 不迁移 `persist_runtime_parameter_mutation_transition`。
- 不迁移 `auto_snapshot_on_activation` helper body。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner、snapshot owner 或 route facade。
- 不启动发布过渡，不提出横向连接或性能旁路。

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

下一批进入 BE-001AM-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 父叶残余判断。只能判断 parent-owned helper 或 `activation_snapshot_side_effect` 是否值得继续细拆；不得直接移动代码。

---

## 幻觉检查点

AI 声称 BE-001AL-04 完成时，必须说明 `rollback_flow` 单叶已 closeout 并设置 `stop_split: true`，但 `transition_lifecycle` 父叶尚未完成。不得宣称 rollback helper 已迁移、snapshot helper body 已迁移、parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `153-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 标为 `stop_split: true`。
3. 全量树记录 BE-001AL-04 并把下一步固定为 BE-001AM-01 父叶残余判断。
4. 本批无代码移动。
5. 本批验证通过后，后续才能进入 BE-001AM-01。
