# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AH-01  
> 基准: `139-runtime.mutation.parameter_mutation.transition_lifecycle单叶closeout.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 单子叶等价基线。当前只冻结 boundary validation、boundary resolution、safe window evaluation 和相关回归证据；本批 `no code movement`。下一步只能进入 BE-001AH-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AH-01 boundary_safety 单子叶等价基线 | 扩展 |
| 规范矩阵 | 父子通信、safe window reason code、boundary request 语义、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 新增白箱节点 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` | 建立单子叶基线 |

---

## 白箱边界

`runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 是 `transition_lifecycle` 下的纯策略子叶，只冻结三组 helper:

- `validate_runtime_parameter_mutation_boundary`
- `resolve_runtime_parameter_mutation_boundary`
- `evaluate_runtime_parameter_mutation_safe_window`

本节点不拥有 activation / rollback 事务编排，不拥有 mutation record persistence，不拥有 run event append，不拥有 snapshot/config generation side effect，也不拥有 proposal create/list/detail、AI proposal、approval review、schema、frontend caller、AppState owner 或发布过渡连接。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` |
| 父模块 | `runtime.mutation.parameter_mutation.transition_lifecycle` |
| 上层模块 | `runtime.mutation.parameter_mutation` |
| 路由入口 | `backend.runtime.routes.mutation` |
| 当前 owner 文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 父级文件 | `src/runtime/mutation/parameter_mutation.rs` |
| 目标文件路径 | BE-001AH-02 决定，候选为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` |
| route facade | `src/backend/runtime/routes/mutation.rs` |
| runtime facade | `src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` |
| 测试证据 | `tests/api_mutation.rs` |
| 下一批次 | BE-001AH-02 抽离方案 |

---

## 输入输出冻结

| Helper | 输入 | 输出 | 必须保持 |
| --- | --- | --- | --- |
| `validate_runtime_parameter_mutation_boundary` | `RuntimeParameterMutationBoundary` | `Ok(())` 或 `(StatusCode, String)` | 空 requested、`immediate`、非法 boundary 均按既有 error code 拒绝 |
| `resolve_runtime_parameter_mutation_boundary` | boundary、current sequence no | resolved `RuntimeParameterMutationBoundary` | `next_cycle_start` 解析为 `current_sequence_no + 2`；`manual_pause` 不写 resolved sequence；`sequence_cursor` 必须有 sequence no |
| `evaluate_runtime_parameter_mutation_safe_window` | optional `RuntimeParameterMutationSafeWindowSnapshot` | `RuntimeParameterMutationSafeWindowState` | allowed/denied、reason code、retryable、retry_after_ms、snapshot 回填语义不变 |

`validate_runtime_parameter_mutation_boundary` 仍必须被父级 `create_runtime_parameter_mutation` 复用；后续若创建子模块，只允许通过 `transition_lifecycle` 的受控 re-export 暴露给 `parameter_mutation` 父级。

---

## Boundary 语义冻结

| requested | 现有语义 |
| --- | --- |
| empty string | `bad_request`，message 指出 `activation_boundary.requested` 必填 |
| `immediate` | `parameter_mutation_boundary_violation`，不支持立即激活 |
| `next_cycle_start` | validation 通过；resolution 写入 `current_sequence_no + 2` |
| `manual_pause` | validation 通过；resolution 不写入 resolved sequence |
| `sequence_cursor` + `resolved_sequence_no` | validation 通过；resolution 规范化为 requested=`sequence_cursor` |
| `sequence_cursor:<u64>` | validation 通过；resolution 解析 `<u64>` 并规范化为 requested=`sequence_cursor` |
| `sequence_cursor` without sequence | `parameter_mutation_boundary_violation`，要求 resolved sequence |
| other requested | `parameter_mutation_boundary_violation` |

---

## Safe Window 语义冻结

| 条件 | reason_code | retryable | retry_after_ms |
| --- | --- | --- | --- |
| runtime status 是 `paused` / `idle` / `stopped` / `ready` 且其余条件安全 | `SAFE_WINDOW_OPEN` | `false` | `None` |
| runtime status 不在允许集合 | `SAFE_WINDOW_RUNTIME_ACTIVE` | `true` | `None` |
| `open_order_count > 0` | `SAFE_WINDOW_OPEN_ORDERS` | `true` | `None` |
| `outstanding_risk_violation == true` | `SAFE_WINDOW_RISK_VIOLATION` | `true` | `None` |
| `data_freshness_ms > 60000` | `SAFE_WINDOW_STALE_DATA` | `true` | `None` |
| `portfolio_exposure_bps.abs() > 10000` | `SAFE_WINDOW_EXPOSURE_LIMIT` | `true` | `None` |
| `cooldown_remaining_ms > 0` | `SAFE_WINDOW_COOLDOWN` | `true` | cooldown ms |

优先级按当前 `else if` 顺序冻结；后续抽离不得改变同一 snapshot 命中多个风险条件时返回的第一个 reason code。

---

## 回归证据冻结

| 测试 | 覆盖范围 |
| --- | --- |
| `runtime_parameter_mutation_safe_window_denial_is_audited_without_activation` | safe window denied reason、audit event、mutation record persistence |
| `runtime_parameter_mutation_activation_uses_explicit_boundary_versions_and_reports` | explicit boundary、resolved sequence、activation report |
| `runtime_parameter_mutation_manual_pause_stays_pending` | manual pause boundary 不立即 activated |
| `runtime_parameter_mutation_rejects_noop_with_rejection_event` | create path 仍复用 boundary / target validation |
| `runtime_parameter_mutation_rolls_back_to_ledger_backed_prior_version` | rollback path 仍通过 boundary + safe window gate |
| `runtime_parameter_mutation_contract_snapshot_matches_fixture` | safe window state / snapshot 字段契约 |

---

## 父子通信规则

```text
backend.runtime.routes.mutation
  -> crate::runtime::{create/activate/rollback}
  -> runtime.mutation.parameter_mutation
  -> runtime.mutation.parameter_mutation.transition_lifecycle
  -> boundary_safety candidate
```

BE-001AH-01 只建立基线，不能创建 `boundary_safety.rs`。BE-001AH-02 才能决定目标路径、`mod boundary_safety` 声明、visibility、re-export 面和回退点。

后续若创建 `boundary_safety`，它只能向父级 `transition_lifecycle` 暴露 helper；`parameter_mutation.rs` 对 `validate_runtime_parameter_mutation_boundary` 的复用必须继续经 `transition_lifecycle` 受控出口，不得让 route facade、AI proposal、approval review、frontend caller 或发布过渡连接直接依赖该子叶。ASCII guard: `release transition guard`。

---

## 排除边界

- 不移动 Rust 代码。
- 不创建 `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`。
- 不拆 activation / rollback handler。
- 不迁移 `mutation_lifecycle_entry`、`persist_runtime_parameter_mutation_transition`、`runtime_parameter_mutation_rollback_record_id` 或 `auto_snapshot_on_activation`。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
- 不主动提出发布版本过渡或横向性能连接。

---

## 回归保护

| 证据 | 覆盖范围 |
| --- | --- |
| `cargo fmt --check` | Rust 格式不漂移 |
| `cargo check -p quantpilot` | type / visibility 不漂移 |
| `cargo test --no-run` | 测试编译不漂移 |
| `cargo test -p quantpilot --test api_mutation` | boundary / safe window / activation / rollback 主证据 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接 shared helper 不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effect 不漂移 |
| `cargo test -p quantpilot --test api_run` | run record append 不漂移 |
| `tools\check-utf8.ps1` | UTF-8 |
| `tools\check-matrix-governance.ps1` | 三矩阵登记 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 |
| `git diff --check` | whitespace |

---

## 下一步

下一批进入 BE-001AH-02 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 抽离方案。该方案必须先决定:

| 决策点 | 候选 | 暂定约束 |
| --- | --- | --- |
| 目标文件路径 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` | 由 `transition_lifecycle.rs` 私有 `mod boundary_safety` 承接 |
| child helper visibility | `pub(super)` | 只允许 `transition_lifecycle` 父级调用 |
| parent re-export | `pub(super) use boundary_safety::validate_runtime_parameter_mutation_boundary;` | 继续让 `parameter_mutation.rs` 复用 validation |
| activation / rollback 调用 | 父级内部调用 `resolve_*` / `evaluate_*` | 不改变事务编排 |
| shared owner | 保留原位 | 不迁移 AppState、snapshot、run event append 或 persistence owner |

---

## 幻觉检查点

AI 声称 BE-001AH-01 完成时，必须说明本批只建立 `boundary_safety` 单子叶等价基线，仍为 `no code movement`。不得宣称 `boundary_safety.rs` 已创建、boundary helper 已迁移、activation/rollback 已继续拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `140-runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` 白箱候选节点。
3. 基线冻结 boundary validation、boundary resolution、safe window evaluation、reason code priority、父级复用方式和排除边界。
4. 治理门禁能发现 `no code movement`、BE-001AH-02 下一步、关键 helper、route、state owner、发布过渡保护和测试证据。
5. 本批验证通过后，后续才能进入 BE-001AH-02 抽离方案。
