# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AL-03  
> 基准: `151-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离方案.md`、`150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` actual extraction 已完成。下一步只能进入 BE-001AL-04 单叶 closeout，判断本叶是否还值得继续细拆。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AL-03 rollback_flow 实际抽离 | 落地 |
| 规范矩阵 | 父子通信、handler re-export、rollback 状态机等价 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 实际文件落位 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 新增实际 Rust 文件 |

---

## 实际变更

本批创建:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`

本批移动:

- `rollback_runtime_parameter_mutation`

父级 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 新增:

```rust
#[path = "transition_lifecycle/rollback_flow.rs"]
mod rollback_flow;

pub(crate) use rollback_flow::rollback_runtime_parameter_mutation;
```

父级仍保留:

- `runtime_parameter_mutation_rollback_record_id`
- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`
- `auto_snapshot_on_activation`
- `boundary_safety` helper 受控导入
- `activation_flow` child 和 activation handler re-export

---

## 等价保持声明

`rollback_flow.rs` 使用 `use super::*;` 调用父级保留的 helper。函数体未改变 rollback request capability guard、activated-only gate、ledger lookup、unknown target / no-op error、safe-window denied 分支、RollbackScheduled / RolledBack / RollbackFailed 状态机、run event append、rollback metrics、transition persistence 和 response schema。

父级仍是唯一受控出口:

```text
backend.runtime.routes.mutation
  -> crate::runtime::rollback_runtime_parameter_mutation
  -> runtime.mutation.parameter_mutation
  -> runtime.mutation.parameter_mutation.transition_lifecycle
  -> rollback_flow::rollback_runtime_parameter_mutation
```

---

## 真实文件

| 文件 | 角色 |
| --- | --- |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` | 父级 owner，保留 shared helper、path child 和 re-export |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` | rollback public handler child |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` | sibling activation public handler child |
| `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` | sibling boundary/safe-window helper child |
| `src/runtime/mutation/parameter_mutation.rs` | 上层 parameter mutation owner |
| `src/backend/runtime/routes/mutation.rs` | route facade |
| `tests/api_mutation.rs` | rollback 主回归证据 |

---

## 本批不做

- 不迁移 `runtime_parameter_mutation_rollback_record_id`。
- 不迁移 `mutation_lifecycle_entry`。
- 不迁移 `persist_runtime_parameter_mutation_transition`。
- 不迁移 `auto_snapshot_on_activation` helper body。
- 不迁移 `activate_runtime_parameter_mutation`。
- 不迁移 `boundary_safety` helper。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
- 不主动提出发布版本过渡或横向性能连接。

---

## 回归保护

| 证据 | 覆盖范围 |
| --- | --- |
| `cargo fmt --check` | Rust 格式不漂移 |
| `cargo check -p quantpilot` | type / visibility 不漂移 |
| `cargo test --no-run` | 测试编译不漂移 |
| `cargo test -p quantpilot --test api_mutation` | activation / safe-window / rollback 主证据 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接 shared helper 不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effect 不漂移 |
| `cargo test -p quantpilot --test api_run` | run record append 不漂移 |
| `tools\check-utf8.ps1` | UTF-8 |
| `tools\check-matrix-governance.ps1` | 三矩阵登记 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 |
| `git diff --check` | whitespace |

---

## 下一步

下一批进入 BE-001AL-04 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单叶 closeout。只能判断本叶是否还值得继续细拆；不得顺手迁移 rollback id helper、transition persistence、snapshot helper body、schema/frontend caller、AI proposal、approval review、AppState 或启动发布过渡。

---

## 幻觉检查点

AI 声称 BE-001AL-03 完成时，必须说明 `rollback_flow` 已实际抽离到 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`，但尚未完成单叶 closeout。不得宣称 rollback helper 已迁移、snapshot helper body 已迁移、parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `152-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 进入全量树覆盖。
3. 父级只通过 path-attributed child 和 handler re-export 暴露 rollback handler。
4. 本批只迁移 `rollback_runtime_parameter_mutation`。
5. 本批验证通过后，后续才能进入 BE-001AL-04 单叶 closeout。
