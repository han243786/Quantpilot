# v4.16.0 runtime.mutation.parameter_mutation 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AF-03  
> 基准: `133-runtime.mutation.parameter_mutation抽离方案.md`、`132-runtime.mutation.parameter_mutation单子叶等价基线.md`、`src/runtime/mutation.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/mutation.rs`、`tests/api_mutation.rs`  
> 判定: 完成 `runtime.mutation.parameter_mutation` 第一轮实际抽离。五个 parameter mutation public handler 和本叶私有 helper 已迁入 `src/runtime/mutation/parameter_mutation.rs`；父级 `src/runtime/mod.rs` 通过 re-export 维持 route facade 调用面；AI proposal、approval review、AppState、schema、frontend caller、锁顺序、shared persistence/governance owner 和发布过渡连接保持不变。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AF-03 parameter mutation handler 实际抽离 | 扩展 |
| 规范矩阵 | 父子通信、shared helper 保留、发布过渡保护、等价门禁 | 执行 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation` | 从方案推进到真实文件 |
| 模块树 | `runtime.mutation.parameter_mutation` | 完成第一轮物理抽离 |

---

## 变更文件

| 文件 | 变更 | 等价约束 |
| --- | --- | --- |
| `src/runtime/mutation/parameter_mutation.rs` | 新增 parameter mutation 子模块 | 承接 create/list/detail/activate/rollback handler 和本叶私有 helper |
| `src/runtime/mod.rs` | 新增 `#[path = "mutation/parameter_mutation.rs"] mod mutation_parameter_mutation;` 与 `pub(crate) use mutation_parameter_mutation` | route facade 仍调用父级 handler 名，不改 HTTP route |
| `src/runtime/mutation.rs` | 删除已迁移 handler/helper，保留 shared helper、AI proposal 和 approval review | 不改变 AI proposal、approval、event append 或 governance owner |
| `src/backend/runtime/routes/mutation.rs` | 未修改 | route 注册顺序、路径和 handler 调用名不变 |

---

## 实际抽离清单

已迁入 `src/runtime/mutation/parameter_mutation.rs` 的 public handler:

- `create_runtime_parameter_mutation`
- `list_runtime_parameter_mutations`
- `get_runtime_parameter_mutation_detail`
- `activate_runtime_parameter_mutation`
- `rollback_runtime_parameter_mutation`

已随 handler 迁移的本叶私有 helper:

- `validate_runtime_parameter_mutation_boundary`
- `resolve_runtime_parameter_mutation_boundary`
- `evaluate_runtime_parameter_mutation_safe_window`
- `runtime_parameter_mutation_record_id`
- `runtime_parameter_mutation_rollback_record_id`
- `mutation_lifecycle_entry`
- `persist_runtime_parameter_mutation_transition`
- `auto_snapshot_on_activation`

继续保留在 `src/runtime/mutation.rs` 的 shared helper:

- `canonical_runtime_parameter_version`
- `validate_runtime_parameter_mutation_target`
- `runtime_parameter_mutation_governance`
- `governance_with_parameter_version`
- `append_parameter_mutation_events_to_run`
- `build_runtime_parameter_mutation_event`
- `mutation_event_contract`
- `status_contract_value`
- `runtime_mode_from_events`

---

## 父子通信结果

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.mutation
  -> src/runtime/mod.rs runtime facade
  -> runtime.mutation.parameter_mutation
  -> parent shared helper / AppState / runtime persistence / run evidence
```

执行结果:

1. `runtime.mutation.parameter_mutation` 只通过父级 runtime facade 暴露 public handler。
2. `src/backend/runtime/routes/mutation.rs` 未直接引用子模块路径，仍经 `crate::runtime as runtime_handlers` 调用。
3. 子模块通过 `use super::*;` 复用父级 shared helper，未反向暴露私有 helper。
4. `RuntimeParameterMutationListQuery` 继续留在 `src/runtime/mod.rs`，schema/query owner 未迁移。
5. 发布过渡未启动；本批没有主动提出横向连接、缓存旁路或性能连接。ASCII guard: `release transition guard`。

---

## 非目标

- 不移动 AI proposal handler: `create_runtime_ai_proposal`、`list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail`。
- 不移动 approval review handler: `list_runtime_approvals`、`get_runtime_approval_detail`、`approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review`。
- 不迁移 `AppState`、`parameter_mutations`、`ai_proposals`、`approval_records`、snapshot/config generation state 或锁顺序。
- 不修改 `src/frontend_api_types.rs`、`src/runtime_persistence.rs`、frontend caller、route path、response schema 或测试资产。
- 不启动发布过渡，不做横向连接或性能旁路。

---

## 等价证据

| 证据 | 覆盖范围 | 本批要求 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 新子模块和父级声明格式稳定 |
| `cargo check -p quantpilot` | Rust 模块/visibility/type | re-export、parent shared helper、route facade 类型不漂移 |
| `cargo test --no-run` | 测试编译 | mutation/AI proposal/approval 邻接 handler 仍可编译 |
| `cargo test -p quantpilot --test api_mutation` | parameter mutation 生命周期 | create/list/detail/activate/rollback、safe window、manual pause、contract snapshot 不漂移 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接域 | shared helper 保留没有破坏 AI proposal |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence/report side effect | event append 与 evidence contract 不漂移 |
| `cargo test -p quantpilot --test api_run` | run record 邻接域 | run record append、replay、status 邻接行为不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增抽离记录保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 本记录、模块树、全量树、索引完整 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新源文件和新里程碑入口可定位 |
| `git diff --check` | whitespace | diff 没有空白错误 |

本批已执行并通过:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test --no-run`
- `cargo test -p quantpilot --test api_mutation`
- `cargo test -p quantpilot --test api_ai_proposal`
- `cargo test -p quantpilot --test api_evidence_contract`
- `cargo test -p quantpilot --test api_run`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`

---

## 下一步

下一批进入 BE-001AF-04 `runtime.mutation.parameter_mutation` 单叶整理 / closeout，必须检查本叶是否值得继续细拆。初步判断点:

| 维度 | 观察 |
| --- | --- |
| create/list/detail | 已形成提案记录 CRUD 子流，但代码量相对集中 |
| activate/rollback | 生命周期和 event append 更重，可能是进一步细拆候选 |
| safe window / boundary | 当前是本叶私有 helper，是否需要独立白箱由 BE-001AF-04 决定 |
| auto snapshot side effect | 仍触达 snapshot/config generation state owner，若继续拆分必须先建等价基线 |

BE-001AF-04 不得直接移动代码；如果判定值得继续细拆，必须回到单子叶等价基线，再走抽离方案。

---

## 幻觉检查点

AI 声称 BE-001AF-03 完成时，必须说明 parameter mutation handler 已迁入 `src/runtime/mutation/parameter_mutation.rs`，route facade 未修改，AI proposal / approval 未迁移，AppState / schema / frontend caller / 锁顺序未改变，发布过渡未启动，下一步只能进入 BE-001AF-04 单叶 closeout。不得宣称 `runtime.mutation` 已全部完成、AI proposal/approval 已拆分、整理或重构已经完成。

---

## 验收标准

1. `134-runtime.mutation.parameter_mutation抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/parameter_mutation.rs` 被全量树和模块树登记为真实文件。
3. `src/runtime/mod.rs` 暴露 `pub(crate) use mutation_parameter_mutation`，route facade 无需改动即可保持调用面。
4. `src/runtime/mutation.rs` 不再持有五个 parameter mutation public handler，但仍持有 shared helper、AI proposal 和 approval review。
5. 本批验证通过后，后续才能进入 BE-001AF-04 单叶 closeout。
