# v4.16.0 runtime.mutation.parameter_mutation 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AF-02  
> 基准: `132-runtime.mutation.parameter_mutation单子叶等价基线.md`、`src/runtime/mutation.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/mutation.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.parameter_mutation` 第一轮物理抽离方案。当前仍是 `no code movement`；下一批 BE-001AF-03 只允许按本文把 parameter mutation public handler 迁入目标子模块，并保持 AI proposal、approval review、AppState、schema、frontend caller、锁顺序、shared persistence/governance owner 和发布过渡连接不变。  
> 代码动作: `no code movement`

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AF-02 parameter mutation handler 抽离方案 | 扩展 |
| 规范矩阵 | 父子通信、共享 helper 保留、发布过渡保护、等价门禁 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation` | 补充抽离目标 |
| 模块树 | `runtime.mutation.parameter_mutation` | 从基线推进到抽离方案 |

---

## 适配性结论

`src/runtime/mod.rs` 是 runtime handler facade，当前通过 `include!("mutation.rs")` 暴露 mutation handler。为了避免在被 include 的 `src/runtime/mutation.rs` 内声明子模块时出现路径歧义，BE-001AF-03 的子模块声明必须落在 `src/runtime/mod.rs`，目标文件固定为:

`src/runtime/mutation/parameter_mutation.rs`

计划结构:

```rust
#[path = "mutation/parameter_mutation.rs"]
mod mutation_parameter_mutation;

pub(crate) use mutation_parameter_mutation::{
    activate_runtime_parameter_mutation,
    create_runtime_parameter_mutation,
    get_runtime_parameter_mutation_detail,
    list_runtime_parameter_mutations,
    rollback_runtime_parameter_mutation,
};
```

该声明必须位于 `include!("mutation.rs")` 之前。route facade `src/backend/runtime/routes/mutation.rs` 继续通过 `crate::runtime as runtime_handlers` 调用，不修改 HTTP route。

---

## 目标边界

| 项 | BE-001AF-03 处理方式 | 约束 |
| --- | --- | --- |
| 目标子文件 | `src/runtime/mutation/parameter_mutation.rs` | 新文件只承载 parameter mutation handler 与本叶私有 helper |
| 父级 facade | `src/runtime/mod.rs` | 新增 `#[path = "mutation/parameter_mutation.rs"]` 与 `pub(crate) use mutation_parameter_mutation` |
| handler source | `src/runtime/mutation.rs` | 删除已迁移 handler/helper，保留 AI proposal、approval review 和 shared helper |
| route facade | `src/backend/runtime/routes/mutation.rs` | 不改 route、不改 handler 调用名、不改注册顺序 |
| list query | `RuntimeParameterMutationListQuery` | 继续留在 `src/runtime/mod.rs`，BE-001AF-03 不迁移 schema/query owner |
| state owner | `AppState` | 不迁移 `parameter_mutations`、`runs`、store dir、snapshot/config generation state 或锁顺序 |
| schema owner | `src/frontend_api_types.rs` | 不改 request/response/event schema |
| release transition guard | release transition guard | 不主动提出横向连接、缓存旁路或性能连接 |

---

## 允许迁移清单

BE-001AF-03 只允许迁移下列 public handler:

| public handler | 输入 | 输出 | 迁移说明 |
| --- | --- | --- | --- |
| `create_runtime_parameter_mutation` | `AppState`、create request | `RuntimeParameterMutationRecord` | 迁入子模块，继续调用父级 shared helper 与 runtime persistence |
| `list_runtime_parameter_mutations` | `RuntimeParameterMutationListQuery` | mutation list | 迁入子模块，query 类型暂留 `src/runtime/mod.rs` |
| `get_runtime_parameter_mutation_detail` | proposal id | mutation detail | 迁入子模块，lookup 顺序不变 |
| `activate_runtime_parameter_mutation` | proposal id、activation body | activated record | 迁入子模块，safe window、event append、auto snapshot side effect 不变 |
| `rollback_runtime_parameter_mutation` | proposal id、rollback body | rolled back record | 迁入子模块，ledger target lookup 和 rollback event contract 不变 |

BE-001AF-03 允许随 handler 迁移的本叶私有 helper:

| helper | 迁移判断 | 约束 |
| --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | 移动 | 只服务 activation boundary 校验 |
| `resolve_runtime_parameter_mutation_boundary` | 移动 | 只服务 activation/rollback boundary resolution |
| `evaluate_runtime_parameter_mutation_safe_window` | 移动 | 只服务 activation/rollback safe window |
| `runtime_parameter_mutation_record_id` | 移动 | 只服务 create record id |
| `runtime_parameter_mutation_rollback_record_id` | 移动 | 只服务 rollback record id |
| `mutation_lifecycle_entry` | 移动 | 只服务 parameter mutation lifecycle |
| `persist_runtime_parameter_mutation_transition` | 移动 | 只服务 activation/rollback transition 写回 |
| `auto_snapshot_on_activation` | 移动 | 随 activation handler 移动，但不迁移 snapshot/config generation owner |

---

## 必须保留在父级的 shared helper

下列函数目前被 AI proposal 或相邻 mutation owner 复用，BE-001AF-03 不得私有化到 `runtime.mutation.parameter_mutation`:

| helper | 保留原因 |
| --- | --- |
| `canonical_runtime_parameter_version` | AI proposal 和 parameter mutation 都依赖 parameter version canonicalization |
| `validate_runtime_parameter_mutation_target` | AI proposal 复用 target validation |
| `runtime_parameter_mutation_governance` | governance copy 仍服务相邻 mutation owner |
| `governance_with_parameter_version` | AI proposal transition 仍复用 parameter version governance |
| `append_parameter_mutation_events_to_run` | AI proposal 也会 append parameter mutation event |
| `build_runtime_parameter_mutation_event` | 与 `append_parameter_mutation_events_to_run` 保持同一父级 event construction owner |
| `mutation_event_contract` | 与 event construction owner 一起保留 |
| `status_contract_value` | 与 event construction owner 一起保留 |
| `runtime_mode_from_events` | 与 event construction owner 一起保留 |

子模块可通过 `use super::*;` 调用父级 shared helper。父级不得反向调用子模块私有 helper；只通过 `pub(crate) use mutation_parameter_mutation::{...}` 暴露五个 public handler。

---

## 父子通信规则

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.mutation
  -> runtime facade in src/runtime/mod.rs
  -> runtime.mutation.parameter_mutation
  -> parent shared helper / AppState / runtime persistence / run evidence
```

硬规则:

1. `runtime.mutation.parameter_mutation` 只能通过父级 runtime facade 暴露 public handler。
2. `backend.runtime.routes.mutation` 只调用父级 re-export，不直接引用子文件路径。
3. `runtime.mutation.parameter_mutation` 不得横向接管 AI proposal、approval review、report、evidence、experiment、ops、executor 或 frontend caller。
4. 状态 owner、schema owner、persistence owner 和锁顺序不迁移。
5. 发布过渡前不得主动提出横向连接或性能旁路；即使下一批性能上能优化，也只能保持开发态父子通信。

---

## BE-001AF-03 操作顺序

1. 在 `src/runtime/mod.rs` 中声明 `mutation_parameter_mutation` 子模块，并 re-export 五个 public handler。
2. 新建 `src/runtime/mutation/parameter_mutation.rs`，先放 `use super::*;`。
3. 从 `src/runtime/mutation.rs` 迁移五个 public handler 和允许迁移的私有 helper。
4. 保留 shared helper 在 `src/runtime/mutation.rs`，必要时由子模块通过父级可见性调用。
5. 不修改 `src/backend/runtime/routes/mutation.rs` route facade。
6. 运行格式、编译、测试和治理门禁。
7. 若失败，按回退点恢复，不继续扩大到 AI proposal 或 approval。

---

## 回退点

若 BE-001AF-03 失败，回退必须只做以下动作:

1. 删除 `src/runtime/mutation/parameter_mutation.rs`。
2. 移除 `src/runtime/mod.rs` 中的 `#[path = "mutation/parameter_mutation.rs"]`、`mod mutation_parameter_mutation;` 和 `pub(crate) use mutation_parameter_mutation`。
3. 将五个 public handler 与已迁移私有 helper 原样放回 `src/runtime/mutation.rs`。
4. 不回退与本批无关的 route facade、AI proposal、approval、schema、AppState、frontend caller 或测试资产。

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 新子模块和父级声明格式稳定 |
| `cargo check -p quantpilot` | Rust 模块/visibility/type | re-export、parent shared helper、route facade 类型不漂移 |
| `cargo test --no-run` | 测试编译 | mutation/AI proposal/approval 邻接 handler 仍可编译 |
| `cargo test -p quantpilot --test api_mutation` | parameter mutation 生命周期 | create/list/detail/activate/rollback、safe window、manual pause、contract snapshot 不漂移 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接域 | shared helper 保留没有破坏 AI proposal |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence/report side effect | event append 与 evidence contract 不漂移 |
| `cargo test -p quantpilot --test api_run` | run record 邻接域 | run record append、replay、status 邻接行为不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增抽离方案保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 本方案、模块树、全量树、路线图索引完整 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新里程碑入口可定位 |
| `git diff --check` | whitespace | diff 没有空白错误 |

---

## 下一步

下一批进入 BE-001AF-03 `runtime.mutation.parameter_mutation` 实际抽离记录。该批只允许执行本文的第一轮物理抽离，不得顺手整理 AI proposal、approval review、shared persistence/governance helper、AppState、schema、frontend caller、report/evidence/experiment/ops route 或发布过渡连接。

BE-001AF-03 完成后，必须再进入 BE-001AF-04 单叶整理 / closeout，判断 `runtime.mutation.parameter_mutation` 是否继续细拆。

---

## 幻觉检查点

AI 声称 BE-001AF-02 完成时，必须说明本批只完成抽离方案，仍是 `no code movement`。不得宣称 parameter mutation handler 已迁移、目标文件已创建、AI proposal/approval 已迁移、AppState 或锁顺序已改变、snapshot/config generation owner 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `133-runtime.mutation.parameter_mutation抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树中 `runtime.mutation.parameter_mutation` 从 BE-001AF-01 基线状态推进到 BE-001AF-02 抽离方案状态。
3. 治理门禁能发现目标路径、父级 re-export、允许迁移/保留 helper、`RuntimeParameterMutationListQuery` 保留、`no code movement`、下一批 BE-001AF-03 和测试证据缺失。
4. 本批验证通过后，后续才能进入 BE-001AF-03 实际抽离。
