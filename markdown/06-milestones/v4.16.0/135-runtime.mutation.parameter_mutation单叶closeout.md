# v4.16.0 runtime.mutation.parameter_mutation 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AF-04  
> 基准: `132-runtime.mutation.parameter_mutation单子叶等价基线.md`、`133-runtime.mutation.parameter_mutation抽离方案.md`、`134-runtime.mutation.parameter_mutation抽离记录.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: `runtime.mutation.parameter_mutation` 第一轮抽离等价成立，但本叶暂不停止细拆，设置 `stop_split: false`。下一步进入 BE-001AG-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线，只冻结 activation / rollback lifecycle，不直接移动代码。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AF 从实际抽离进入单叶 closeout，下一轮进入 transition lifecycle 基线 | 收束 |
| 规范矩阵 | 父级 re-export、父子通信、细分价值判断、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation` | 单叶 closeout |
| 模块树 | `runtime.mutation.parameter_mutation` | 设置 `stop_split: false` 并登记下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.parameter_mutation` |
| 父模块 | `backend.runtime` |
| 路由入口 | `backend.runtime.routes.mutation` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/runtime/mutation/parameter_mutation.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.parameter_mutation` |
| 真实文件 | `src/runtime/mutation/parameter_mutation.rs`、`src/runtime/mutation.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/mutation.rs` |
| public 方法 | `create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail`、`activate_runtime_parameter_mutation`、`rollback_runtime_parameter_mutation` |
| 父级 re-export | `pub(crate) use mutation_parameter_mutation::{activate_runtime_parameter_mutation,create_runtime_parameter_mutation,get_runtime_parameter_mutation_detail,list_runtime_parameter_mutations,rollback_runtime_parameter_mutation};` |
| 子模块声明 | `#[path = "mutation/parameter_mutation.rs"] mod mutation_parameter_mutation;` |
| closeout 判定 | `stop_split: false` |
| 下一递归点 | BE-001AG-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_mutation`、`cargo test -p quantpilot --test api_ai_proposal`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 等价 closeout 结论

| 维度 | 结论 |
| --- | --- |
| route 入口 | 等价。`src/backend/runtime/routes/mutation.rs` 未变更，仍调用 `runtime_handlers::*` |
| 父级出口 | 等价。`src/runtime/mod.rs` 通过 `pub(crate) use mutation_parameter_mutation` 暴露五个 handler |
| handler 文件 | 已抽离。五个 parameter mutation public handler 位于 `src/runtime/mutation/parameter_mutation.rs` |
| create/list/detail | 等价。mutation ledger id、list filtering/order、detail scoped lookup 未变 |
| activation | 等价。safe window、boundary resolution、schedule/activated event、auto snapshot side effect 未变 |
| rollback | 等价。ledger target lookup、rollback schedule/rolled_back event、safe window denial 未变 |
| shared helper | 等价。`canonical_runtime_parameter_version`、`validate_runtime_parameter_mutation_target`、`runtime_parameter_mutation_governance`、`governance_with_parameter_version`、`append_parameter_mutation_events_to_run`、`build_runtime_parameter_mutation_event` 仍留父级 |
| AppState / 锁顺序 | 未变更 |
| schema / frontend caller | 未变更 |
| 发布过渡 | 未启动，不新增横向连接或性能旁路 |

---

## 细分价值判断

**最终判定**: `runtime.mutation.parameter_mutation` 当前不停止细拆，设置 `stop_split: false`。

理由: 本叶已完成第一轮物理抽离，但新文件仍是 864 行，内部存在明显的事务子域。继续细拆有助于把 create/list/detail 的 proposal record 流、activation/rollback 的 transition lifecycle 流、safe window/boundary helper 和 auto snapshot side effect 分层管理。当前最值得先建基线的是 `runtime.mutation.parameter_mutation.transition_lifecycle`，因为它同时覆盖 activation、rollback、lifecycle entry、transition persistence、safe window/boundary 和 run event append，是文件中复杂度最高、回归风险最大的区域。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.mutation.parameter_mutation.transition_lifecycle` | 值得拆，下一候选 | `activate_runtime_parameter_mutation` 与 `rollback_runtime_parameter_mutation` 共享 boundary、safe window、lifecycle entry、transition persistence 和 event append，事务边界清晰 |
| `runtime.mutation.parameter_mutation.proposal_record` | 后续候选 | create/list/detail 与 record id 相关，但当前复杂度低于 transition lifecycle |
| `runtime.mutation.parameter_mutation.safe_window_boundary` | 暂缓 | helper 只服务 transition lifecycle，宜先纳入 transition 基线，避免过早制造微文件 |
| `runtime.mutation.parameter_mutation.auto_snapshot_side_effect` | 暂缓 | 触达 snapshot/config generation owner，必须在 transition lifecycle 基线中冻结后再判断 |
| `runtime.mutation.parameter_mutation.shared_event_governance` | 暂缓 | 父级 shared helper 同时服务 AI proposal，不能从 parameter leaf 私有化 |

ASCII markers: `transition lifecycle`、`proposal record`、`safe window boundary`、`auto snapshot side effect`。

transition lifecycle 下一基线必须冻结的关键 helper:

- `validate_runtime_parameter_mutation_boundary`
- `resolve_runtime_parameter_mutation_boundary`
- `evaluate_runtime_parameter_mutation_safe_window`
- `persist_runtime_parameter_mutation_transition`
- `auto_snapshot_on_activation`
- `append_parameter_mutation_events_to_run`

---

## 父子通信收口

```text
backend.runtime.routes.mutation
  -> crate::runtime::{create/list/detail/activate/rollback}
  -> runtime.mutation.parameter_mutation
  -> parent shared helper / AppState / runtime persistence / run evidence
```

本叶只能经父级 `src/runtime/mod.rs` re-export 暴露，不得让 `backend.runtime.routes.mutation` 直接引用 `src/runtime/mutation/parameter_mutation.rs`。不得横向接管 AI proposal、approval review、report、evidence、experiment、ops、strategy_config、executor、schema、frontend caller 或 persistence owner。发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.mutation.parameter_mutation.transition_lifecycle` | 默认下一步 | BE-001AG-01 先建立单子叶等价基线，不移动代码 |
| `runtime.mutation.parameter_mutation.proposal_record` | 后续候选 | transition lifecycle 完成后再判断 |
| `runtime.mutation.ai_proposal` | 后续 sibling | 不得在 parameter mutation 内顺手处理 |
| `runtime.mutation.approval_review` | 后续 sibling | 不得在 parameter mutation 内顺手处理 |
| `runtime.mutation.shared_persistence_governance` | 暂缓 | 等 parameter / AI proposal / approval 三条主线稳定后再判断 |

---

## 本批次不做

- 不移动任何 Rust 代码。
- 不继续拆 activate/rollback、safe window、boundary 或 auto snapshot。
- 不迁移 AI proposal、approval review、AppState、schema、frontend caller、runtime persistence、run record persistence 或 shared governance helper。
- 不修改 `src/backend/runtime/routes/mutation.rs`。
- 不主动提出发布版本过渡或横向连接。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001AF-04 完成时，必须说明本批只完成 `runtime.mutation.parameter_mutation` 单叶 closeout 和细分价值判断，`stop_split: false`，下一步只能进入 BE-001AG-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线。不得宣称 transition lifecycle 已抽离、AI proposal/approval 已迁移、AppState 或锁顺序已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `135-runtime.mutation.parameter_mutation单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.mutation.parameter_mutation` closeout 完成并设置 `stop_split: false`。
3. closeout 明确下一候选为 BE-001AG-01 `runtime.mutation.parameter_mutation.transition_lifecycle` 单子叶等价基线。
4. closeout 明确本批 `no code movement`，不得继续移动 activate/rollback、safe window、auto snapshot、AI proposal、approval、schema、state、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AG-01。
