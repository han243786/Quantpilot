# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AG-02  
> 基线: `136-runtime.mutation.parameter_mutation.transition_lifecycle单子叶等价基线.md`、`src/runtime/mutation/parameter_mutation.rs`、`src/runtime/mutation.rs`、`tests/api_mutation.rs`  
> 判定: 只建立 transition lifecycle 抽离方案，当前 `no code movement`。下一步只能进入 BE-001AG-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AG-02 transition lifecycle 抽离方案 | 扩展 |
| 规范矩阵 | 父子通信、可见性、状态机事务边界、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle` | 补充抽离计划 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle` | 进入实际抽离前置方案 |

---

## 抽离目标

下一批 BE-001AG-03 只允许创建一个 transition child:

| 项 | 决定 |
| --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级文件 | `src/runtime/mutation/parameter_mutation.rs` |
| 父级声明 | `#[path = "parameter_mutation/transition_lifecycle.rs"] mod transition_lifecycle;` |
| 父级 handler 出口 | `pub(crate) use transition_lifecycle::{activate_runtime_parameter_mutation, rollback_runtime_parameter_mutation};` |
| 父级 boundary 出口 | `use transition_lifecycle::validate_runtime_parameter_mutation_boundary;` |
| 子级导入 | `use super::*;` |
| 下一批次 | BE-001AG-03 实际抽离 |

该目标文件承接 activation / rollback lifecycle 的事务编排，不拥有 proposal create/list/detail，不拥有 AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner 或 release transition guard。

---

## 迁移清单

BE-001AG-03 只允许迁移以下函数:

| 函数 | 目标可见性 | 迁移原因 |
| --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `pub(super)` | transition 与父级 create handler 都需要 boundary validation |
| `resolve_runtime_parameter_mutation_boundary` | private | 只服务 activation / rollback transition |
| `evaluate_runtime_parameter_mutation_safe_window` | private | 只服务 transition safe window |
| `runtime_parameter_mutation_rollback_record_id` | private | 只服务 rollback proposal id |
| `mutation_lifecycle_entry` | private | 只服务 transition lifecycle event metadata |
| `persist_runtime_parameter_mutation_transition` | private | 只服务 activation / rollback record persist |
| `auto_snapshot_on_activation` | private | 只服务 activation side effect |
| `activate_runtime_parameter_mutation` | `pub(crate)` | route facade public handler |
| `rollback_runtime_parameter_mutation` | `pub(crate)` | route facade public handler |

不得迁移以下函数或 owner:

- `create_runtime_parameter_mutation`
- `list_runtime_parameter_mutations`
- `get_runtime_parameter_mutation_detail`
- `runtime_parameter_mutation_record_id`
- `canonical_runtime_parameter_version`
- `validate_runtime_parameter_mutation_target`
- `build_runtime_parameter_mutation_event`
- `append_parameter_mutation_events_to_run`
- `governance_with_parameter_version`
- `runtime_parameter_mutation_governance`
- `mutation_event_contract`
- `status_contract_value`
- `runtime_mode_from_events`
- AI proposal、approval review、AppState、schema、frontend caller、route facade、test fixture、release transition guard

---

## 父子通信规则

```text
src/backend/runtime/routes/mutation.rs
  -> src/runtime/mod.rs
  -> src/runtime/mutation/parameter_mutation.rs
  -> src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
  -> parent shared helper / AppState / runtime persistence / run evidence
```

BE-001AG-03 必须保持父级 `runtime.mutation.parameter_mutation` 为唯一对外承接点。route facade 继续只调用 `src/runtime/mod.rs` re-export 出来的 handler 名称。transition child 不得横向接管 AI proposal、approval review、runtime persistence、snapshot owner、frontend caller 或 release transition guard。

---

## 预期文件结构

```text
src/runtime/mod.rs
  #[path = "mutation/parameter_mutation.rs"]
  mod mutation_parameter_mutation;
  pub(crate) use mutation_parameter_mutation::{
      activate_runtime_parameter_mutation,
      create_runtime_parameter_mutation,
      get_runtime_parameter_mutation_detail,
      list_runtime_parameter_mutations,
      rollback_runtime_parameter_mutation,
  };

src/runtime/mutation/parameter_mutation.rs
  use super::*;
  #[path = "parameter_mutation/transition_lifecycle.rs"]
  mod transition_lifecycle;
  pub(crate) use transition_lifecycle::{
      activate_runtime_parameter_mutation,
      rollback_runtime_parameter_mutation,
  };
  use transition_lifecycle::validate_runtime_parameter_mutation_boundary;
  // create/list/detail/proposal record helpers remain here

src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
  use super::*;
  pub(super) fn validate_runtime_parameter_mutation_boundary(...)
  pub(crate) async fn activate_runtime_parameter_mutation(...)
  pub(crate) async fn rollback_runtime_parameter_mutation(...)
```

---

## 等价约束

HTTP route 必须保持:

| Route | Method | Handler |
| --- | --- | --- |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` |

request type 必须保持:

| Handler | Request type |
| --- | --- |
| `activate_runtime_parameter_mutation` | `ActivateRuntimeParameterMutationRequest` |
| `rollback_runtime_parameter_mutation` | `RollbackRuntimeParameterMutationRequest` |

状态机必须保持:

- `SafeWindowDenied`
- `ActivationScheduled`
- `Activated`
- `ActivationFailed`
- `RollbackScheduled`
- `RolledBack`
- `RollbackFailed`

safe window 必须保持:

- runtime status 非 `paused` / `idle` / `stopped` / `ready` 时拒绝。
- open order、outstanding risk violation、stale data、exposure limit、cooldown remaining 的 reason code、message、retryable、`retry_after_ms` 不变。
- `next_cycle_start` 继续解析为 `current_sequence_no + 2`。
- `manual_pause` 继续不设置 resolved sequence。
- `sequence_cursor` 与 `sequence_cursor:<u64>` 语义不变。

side effect 必须保持:

- `append_parameter_mutation_events_to_run` 仍由父级 shared helper 执行。
- `auto_snapshot_on_activation` 只移动 helper，不迁移 snapshot owner。
- `persist_runtime_parameter_mutation_transition` 不改变 `auth::scoped_key(user_id, proposal_id)`。
- metrics 计数不改变。
- response schema 不改变。

---

## 回退点

若 BE-001AG-03 出现 visibility 或生命周期编译问题，只回退 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 和父级 `mod transition_lifecycle` / `use transition_lifecycle::*` 相关声明，不得回退 BE-001AF 已完成的 `parameter_mutation.rs` 子模块抽离。

---

## 回归保护

| 证据 | 覆盖范围 |
| --- | --- |
| `cargo fmt --check` | Rust 格式 |
| `cargo check -p quantpilot` | type / visibility |
| `cargo test --no-run` | 测试编译 |
| `cargo test -p quantpilot --test api_mutation` | activation / rollback lifecycle 主证据 |
| `cargo test -p quantpilot --test api_ai_proposal` | mutation shared helper 邻接证据 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence / report side effect |
| `cargo test -p quantpilot --test api_run` | run record append |
| `tools\check-utf8.ps1` | UTF-8 |
| `tools\check-matrix-governance.ps1` | 三矩阵登记 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 |
| `git diff --check` | whitespace |

---

## 下一步

下一批进入 BE-001AG-03 `runtime.mutation.parameter_mutation.transition_lifecycle` 实际抽离，只允许按本方案创建 `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 并迁移清单内函数。不得顺手整理 proposal record、AI proposal、approval review、shared persistence/governance、schema、frontend caller 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001AG-02 完成时，必须说明本批只建立抽离方案，仍为 `no code movement`。不得宣称 transition lifecycle 已抽离、目标文件已创建、Rust 编译已针对目标文件通过、safe window owner 已迁移、auto snapshot owner 已迁移、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

---

## 验收标准

1. `137-runtime.mutation.parameter_mutation.transition_lifecycle抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确 BE-001AG-03 的目标文件、父级声明、handler re-export、boundary validation 父级可见性和迁移清单。
3. 方案明确 `create_runtime_parameter_mutation` 仍留父级，但通过 `validate_runtime_parameter_mutation_boundary` 继续复用 boundary validation。
4. 治理门禁能发现 `no code movement`、BE-001AG-03 下一步、关键 handler/helper、route、状态机、发布过渡保护和测试证据。
5. 本批验证通过后，后续才能进入 BE-001AG-03 实际抽离。
