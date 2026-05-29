# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AJ-03  
> 基线: `145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md`、`146-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离方案.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` actual extraction 已完成。下一步只能进入 BE-001AJ-04 单叶 closeout，判断本叶是否还值得继续细拆。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AJ-03 activation_flow 实际抽离 | 落地 |
| 规范矩阵 | 父子通信、handler re-export、visibility、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 白箱节点落地 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 记录真实文件 |

---

## 实际抽离结果

| 项 | 结果 |
| --- | --- |
| 新增文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` |
| 父级文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级声明 | `#[path = "transition_lifecycle/activation_flow.rs"] mod activation_flow;` |
| 父级出口 | `pub(crate) use activation_flow::activate_runtime_parameter_mutation;` |
| child prelude | `use super::*;` |
| 迁移函数 | `activate_runtime_parameter_mutation` |
| 输入类型 | `ActivateRuntimeParameterMutationRequest` |
| 输出类型 | `RuntimeParameterMutationRecord` |

本批只迁移 activation public handler。`rollback_runtime_parameter_mutation`、`auto_snapshot_on_activation`、`persist_runtime_parameter_mutation_transition`、`mutation_lifecycle_entry`、rollback id helper 和 `boundary_safety` wrapper/helper 均仍由父级 `transition_lifecycle.rs` 管理。

---

## 等价约束

本批未改变:

- capability guard: `parameter_mutation_boundary_violation`。
- invalid status gate: 仅 `Proposed` / `SafeWindowDenied` 可进入 activation。
- safe-window denied: `SafeWindowDenied` lifecycle/event、metric、`parameter_mutation_safe_window_denied`。
- schedule branch: `ActivationScheduled` lifecycle/event、scheduled metric。
- next-cycle branch: `Activated` lifecycle/event、active parameter version、applied metric。
- failed boundary branch: `ActivationFailed` lifecycle/event、failure reason、failed metric。
- append order: `append_parameter_mutation_events_to_run` -> `persist_runtime_parameter_mutation_transition` -> `auto_snapshot_on_activation`。
- response schema、AppState、frontend caller、route facade、AI proposal、approval review 和 release transition guard。

---

## 父子通信结果

```text
parameter_mutation.rs
  -> transition_lifecycle::activate_runtime_parameter_mutation
transition_lifecycle.rs
  -> activation_flow::activate_runtime_parameter_mutation via re-export
activation_flow.rs
  -> parent-owned helpers via use super::*
```

`activation_flow` 只能由父级 `transition_lifecycle` 注册和 re-export。`src/runtime/mutation/parameter_mutation.rs`、route facade、AI proposal、approval review、schema、frontend caller 和发布过渡连接不得直接依赖本 child。

---

## 非目标

- 不迁移 `rollback_runtime_parameter_mutation`。
- 不迁移 `auto_snapshot_on_activation` helper body。
- 不迁移 `persist_runtime_parameter_mutation_transition`。
- 不迁移 `boundary_safety` helper。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
- 不主动提出发布版本过渡或横向性能连接。

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

下一批进入 BE-001AJ-04 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单叶 closeout。closeout 只判断本叶是否还值得继续细拆；不得顺手迁移 rollback flow、snapshot helper body、schema/frontend caller、route facade、AI proposal、approval review、AppState 或启动发布过渡。

---

## 幻觉检查点

AI 声称 BE-001AJ-03 完成时，必须说明 `activation_flow` 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`，但尚未完成单叶 closeout。不得宣称 rollback flow 已拆分、snapshot helper body 已迁移、parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` 进入模块树和全量树。
2. `147-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. 父级 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 使用 path-attributed child 与 `pub(crate) use activation_flow::activate_runtime_parameter_mutation` 保持上层调用面。
4. `activate_runtime_parameter_mutation` 的状态机、event append、metrics、transition persistence 和 snapshot trigger 顺序保持等价。
5. 本批验证通过后，后续才能进入 BE-001AJ-04 单叶 closeout。
