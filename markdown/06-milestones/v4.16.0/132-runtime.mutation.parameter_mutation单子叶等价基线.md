# v4.16.0 runtime.mutation.parameter_mutation 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AF-01  
> 基准: `131-backend.runtime.routes.mutation单叶closeout.md`、`128-backend.runtime.routes.mutation单子叶等价基线.md`、`tests/api_mutation.rs`、`src/runtime/mutation.rs`  
> 判定: 建立 `runtime.mutation.parameter_mutation` 单子叶等价基线。当前只冻结 runtime parameter mutation handler 域的 create/list/detail/activate/rollback、safe window、activation/rollback event、parameter version、run record append、persistence bridge 和测试证据；本批 `no code movement`。下一步只能进入 BE-001AF-02 抽离方案。  
> 代码动作: `no code movement`

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AF-01 parameter mutation handler 单子叶等价基线 | 扩展 |
| 规范矩阵 | handler owner、参数版本、safe window、父子通信、状态 owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation` | 新增白箱节点 |
| 模块树 | `runtime.mutation.parameter_mutation` | 建立单子叶基线 |

---

## 选择理由

`runtime.mutation.parameter_mutation` 是 `src/runtime/mutation.rs` handler 域的第一条稳定主线:

1. 它覆盖 `/api/runtime/mutations` 的 create/list/detail/activate/rollback 全生命周期，输入输出和测试证据最完整。
2. 它有独立的 safe window、activation boundary、rollback target、parameter version canonicalization 和 mutation lifecycle event 语义。
3. `api_mutation` 已覆盖 proposal、noop rejection、canonical version、activation、manual pause、safe window denial、rollback 和 contract snapshot。
4. AI proposal、approval review 和 shared persistence/governance helper 仍与本叶相邻，但不在本批移动，能避免把三组状态流一次性揉成大改。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.parameter_mutation` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.parameter_mutation` |
| 父模块 | `backend.runtime` |
| route 入口 | `backend.runtime.routes.mutation` |
| handler owner | `src/runtime/mutation.rs` |
| handler facade | `src/runtime/mod.rs` |
| route facade | `src/backend/runtime/routes/mutation.rs` |
| state owner | `AppState` in `src/lib.rs` |
| schema owner | `src/frontend_api_types.rs` |
| persistence owner | `src/runtime_persistence.rs` |
| 测试证据 | `tests/api_mutation.rs` |
| 下一批次 | BE-001AF-02 抽离方案 |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `CreateRuntimeParameterMutationRequest` | frontend、API caller、`api_mutation` | 必须保留 source_kind=run、capability context、actor、reason、target、old/new value 和 activation boundary 校验 |
| 输入 | `ActivateRuntimeParameterMutationRequest` | frontend、API caller、`api_mutation` | 必须保留 capability guard、safe window、explicit boundary、manual pause 和 sequence cursor 语义 |
| 输入 | `RollbackRuntimeParameterMutationRequest` | frontend、API caller、`api_mutation` | 必须保留 activated-only guard、ledger-backed target lookup、safe window 和 rollback boundary |
| 输入 | `RuntimeParameterMutationListQuery` | frontend、API caller、`api_mutation` | 必须保留 source_kind/source_id filtering、created_at desc 排序和 pagination |
| 输入 | `AppState` | backend runtime state | 不迁移 `parameter_mutations`、`runs`、`mutation_store_dir`、`run_store_dir`、`evidence_metrics`、snapshot/config generation state |
| 输出 | `RuntimeParameterMutationRecord` | frontend、tests、runtime evidence | 不改变 record schema、status、governance、lifecycle、safe_window_state 或 activation_state |
| 输出 | `FrontendRuntimeEvent` | run record / replay / report | 不改变 event_type、reason_code、envelope、retention_class、sequence_no 或 parameter_version |
| 输出 | persisted mutation record | `src/runtime_persistence.rs` | 不改变 file name、load/list/persist behavior |

---

## handler / helper 边界基线

| 子域 | 当前函数 | 基线约束 |
| --- | --- | --- |
| target/version/boundary validation | `canonical_runtime_parameter_version`、`validate_runtime_parameter_mutation_target`、`validate_runtime_parameter_mutation_boundary`、`resolve_runtime_parameter_mutation_boundary` | 不改变 canonical digest、supported module gate、immediate activation denial、next_cycle/manual_pause/sequence_cursor 解析 |
| safe window | `evaluate_runtime_parameter_mutation_safe_window` | 不改变 runtime status、open order、risk violation、freshness、exposure、cooldown 的 denial reason |
| event projection | `status_contract_value`、`mutation_event_contract`、`build_runtime_parameter_mutation_event`、`append_parameter_mutation_events_to_run` | 不改变 ParameterMutation event type、reason_code、payload、envelope append、run governance parameter_version |
| governance/id | `runtime_parameter_mutation_governance`、`runtime_parameter_mutation_record_id`、`runtime_parameter_mutation_rollback_record_id` | 不改变 record id digest 输入、capability/deployment/strategy/permission boundary copy |
| proposal handlers | `create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail` | 不改变 proposal persistence、noop rejection、list sorting/filtering、memory-first detail lookup |
| lifecycle persistence | `mutation_lifecycle_entry`、`governance_with_parameter_version`、`persist_runtime_parameter_mutation_transition` | 不改变 lifecycle entry schema、governance copy 或 memory/disk write order |
| activation handler | `activate_runtime_parameter_mutation` | 不改变 proposed/safe_window_denied guard、safe window denial side effect、schedule/apply/fail transition、auto snapshot side effect |
| activation side effect | `auto_snapshot_on_activation` | 本批只登记未来决策点；不得迁移 snapshot/config generation owner |
| rollback handler | `rollback_runtime_parameter_mutation` | 不改变 activated-only guard、ledger target lookup、noop guard、schedule/apply/fail transition |

---

## route owner 基线

| route | method | route owner | handler |
| --- | --- | --- | --- |
| `/api/runtime/mutations` | GET | `src/backend/runtime/routes/mutation.rs` | `list_runtime_parameter_mutations` |
| `/api/runtime/mutations` | POST | `src/backend/runtime/routes/mutation.rs` | `create_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id` | GET | `src/backend/runtime/routes/mutation.rs` | `get_runtime_parameter_mutation_detail` |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `src/backend/runtime/routes/mutation.rs` | `activate_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `src/backend/runtime/routes/mutation.rs` | `rollback_runtime_parameter_mutation` |

---

## 关键 public 方法

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_parameter_mutation` | mutation request | mutation proposal record | `backend.runtime.routes.mutation` | 不得绕过 capability / target / boundary / actor / reason 校验 |
| `list_runtime_parameter_mutations` | list query | paginated records | `backend.runtime.routes.mutation` | 不得改变 filtering、排序或 pagination |
| `get_runtime_parameter_mutation_detail` | proposal id | mutation record | `backend.runtime.routes.mutation` | 不得绕过 scoped memory lookup 或 disk fallback |
| `activate_runtime_parameter_mutation` | proposal id + activation request | mutation record | `backend.runtime.routes.mutation` | 不得改变 safe window、boundary、event append、snapshot side effect |
| `rollback_runtime_parameter_mutation` | proposal id + rollback request | mutation record | `backend.runtime.routes.mutation` | 不得改变 activated-only、ledger target lookup、rollback event |

---

## 父子通信规则

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.mutation
  -> runtime.mutation.parameter_mutation
  -> AppState / runtime persistence / run evidence
```

`runtime.mutation.parameter_mutation` 只能经 `backend.runtime.routes.mutation` 暴露 HTTP route。它不得横向接管 AI proposal、approval review、report、evidence、experiment、ops、strategy_config、frontend caller 或 executor。状态 owner 仍是 `AppState`，schema owner 仍是 `src/frontend_api_types.rs`，persistence owner 仍是 `src/runtime_persistence.rs`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `src/runtime/mutation.rs` 中任何 handler/helper。
- 不新建 `src/runtime/parameter_mutation.rs`、`src/runtime/mutation/parameter_mutation.rs` 或其他代码文件。
- 不修改 `src/runtime/mod.rs` include facade。
- 不修改 `src/backend/runtime/routes/mutation.rs` route facade。
- 不迁移 `AppState`、`parameter_mutations`、`runs`、`mutation_store_dir`、`run_store_dir`、snapshot/config generation state 或锁顺序。
- 不修改 `src/frontend_api_types.rs` schema、frontend caller、fixture、测试资产或发布过渡协议。
- 不把 AI proposal、approval review、shared persistence/governance helper、report/evidence/experiment/ops 混入本子叶。

---

## 未来抽离决策点

| 决策点 | 当前默认 | 原因 |
| --- | --- | --- |
| 目标文件路径 | BE-001AF-02 决定 | `src/runtime/mod.rs` 当前通过 `include!("mutation.rs")` 暴露 handler，直接声明子模块的路径需谨慎 |
| `auto_snapshot_on_activation` 是否随 activation handler 移动 | BE-001AF-02 决定 | 该 helper 只被 activation 调用，但触达 snapshot/config generation state owner |
| `append_parameter_mutation_events_to_run` 是否作为本叶私有 helper | BE-001AF-02 决定 | 它服务 parameter mutation event append，但触达 run record persistence owner |
| shared persistence/governance helper | 暂不迁移 | `persist_runtime_parameter_mutation_record`、load/list、governance copy 仍被后续 AI proposal / approval 判断影响 |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 本批没有制造格式漂移 |
| `cargo check -p quantpilot` | Rust 模块与 route handler 类型 | handler owner 与 route facade 类型不漂移 |
| `cargo test --no-run` | 测试编译 | mutation / proposal / approval 邻接 handler 仍可编译 |
| `cargo test -p quantpilot --test api_mutation` | parameter mutation 全生命周期 | capability guard、proposal、noop rejection、canonical version、activation、manual pause、safe window denial、rollback、snapshot contract 不漂移 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接域 | baseline 不误伤 AI proposal handler |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effects | mutation evidence/report 健康指标不漂移 |
| `cargo test -p quantpilot --test api_run` | run record / report 邻接域 | event append、replay、report 邻接行为不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增基线保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 基线、模块树、全量树引用不缺失 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新基线和真实文件可定位 |
| `git diff --check` | diff whitespace | 本批没有空白错误 |

---

## 下一步

下一批进入 BE-001AF-02 `runtime.mutation.parameter_mutation` 抽离方案。该批仍应先保持 `no code movement`，只允许规划目标文件路径、允许迁移函数清单、保留父级 re-export / include facade、验证门禁和回退点。

不得直接移动 AI proposal、approval review、shared persistence/governance helper、AppState、锁顺序、schema、frontend caller、report/evidence/experiment/ops route 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001AF-01 完成时，必须说明本批只建立 `runtime.mutation.parameter_mutation` 单子叶等价基线，且为 `no code movement`。不得宣称 parameter mutation handler 已迁移、目标文件已创建、AI proposal/approval 已迁移、AppState 或锁顺序已改变、snapshot/config generation owner 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `132-runtime.mutation.parameter_mutation单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.parameter_mutation` 白箱节点，包含 handler、helper、route、状态 owner、schema owner、persistence owner 和排除边界。
3. 治理门禁能发现本文档、`no code movement`、下一批 BE-001AF-02、关键 handler/helper、未来决策点和测试证据缺失。
4. 本批验证通过后，后续才能进入 BE-001AF-02 抽离方案。
