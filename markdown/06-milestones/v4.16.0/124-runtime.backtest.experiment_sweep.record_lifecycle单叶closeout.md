# v4.16.0 runtime.backtest.experiment_sweep.record_lifecycle 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001AA-04。  
> 基准: `123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md`、`122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md`、`121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`、`120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`。  
> 判定: `runtime.backtest.experiment_sweep.record_lifecycle` 已完成单叶整理 / closeout；四个 lifecycle handler 等价成立，本叶设置 `stop_split: true`。下一步应回到 `runtime.backtest.experiment_sweep` 父叶残余判断，默认进入 BE-001AB-01。

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AA record_lifecycle 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级私有子模块、受控 re-export、父子通信、细分停止条件 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` | 单叶 closeout |
| 模块树 | `runtime.backtest.experiment_sweep.record_lifecycle` | 设置停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.record_lifecycle` |
| 真实文件 | `src/runtime/backtest/record_lifecycle.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/backtest_artifacts.rs` |
| public 方法 | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 父级 re-export | `pub(crate) use record_lifecycle::{discard_experiment_record,get_experiment_detail,list_experiments,save_experiment_record};` |
| 父级模块声明 | `mod record_lifecycle;` |
| 子模块导入 | `use super::*;` |
| closeout 判定 | `stop_split: true` |
| 下一递归点 | BE-001AB-01 `runtime.backtest.experiment_sweep` 第三轮父叶残余判断 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 等价 closeout 结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `/api/runtime/experiments`、detail、save、discard 仍经 `backend.runtime.routes` / backtest route facade 调用 `crate::runtime::*` |
| 父级出口 | 等价 | `experiment_sweep.rs` 通过 `pub(crate) use record_lifecycle::{...};` 保持 `crate::runtime::*` 兼容出口 |
| handler 文件 | 已抽离 | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 已进入 `src/runtime/backtest/record_lifecycle.rs` |
| list read | 等价 | 仍调用 `list_experiment_records(state.experiment_store_dir.as_ref())` |
| list projection | 等价 | 仍 `.map(experiment_list_item_from_record)` 并按 `created_at_ms` 倒序后 `paginate(items, pagination)` |
| detail lookup | 等价 | 仍调用 `load_experiment_record_from_state(&state, &user_id, &experiment_id)` |
| detail response | 等价 | 仍调用 `experiment_detail_response_from_record(record)` |
| save persistence | 等价 | 每个 variant 仍加载 backtest record，调用 `persist_backtest_record`，再清理 transient backtest |
| save state cache | 等价 | 仍写入 `state.experiments` scoped cache，并在 actor 存在时写 `GraphAuditAction::ExperimentCreated` |
| discard conflict | 等价 | saved experiment 仍返回 `StatusCode::CONFLICT` |
| discard cleanup | 等价 | 仍通过 `sanitize_storage_path_segment` 清理路径、删除 preview experiment、移除 transient backtest cache 并调用 `delete_transient_backtest_record` |
| 外部 owner | 保留 | route、schema、state、persistence、response mapping、audit、frontend caller 和发布过渡均未迁移 |

---

## 细分价值判断

**最终判定**: `runtime.backtest.experiment_sweep.record_lifecycle` 当前不继续细拆，设置 `stop_split: true`。

理由: 本叶已经是围绕 experiment record list/detail/save/discard 的完整 lifecycle handler 子模块。继续拆成 read/write 或单 handler 文件，会增加父级 `mod` 与 re-export 面，但不会形成新的稳定 owner。真正复杂且需要共享的边界仍在外部: `runtime_persistence`、`runtime_response_mapping`、AppState cache、graph audit、path sanitize、backtest artifact/schema 和 frontend caller。把这些 owner 私有化到本叶，反而会破坏 backtest record store、experiment sweep、report/evidence 与后续复用边界。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `record_lifecycle.list_query` | 不拆 | `list_experiments` 只是读取、映射、排序和分页；真实 owner 是 persistence 与 response mapping helper |
| `record_lifecycle.detail_projection` | 不拆 | `get_experiment_detail` 是薄 handler，projection owner 在 response mapping helper |
| `record_lifecycle.save_transition` | 不拆 | save 事务必须同时处理 variant persistence、transient cleanup、state cache 和 audit，拆出只会制造桥接层 |
| `record_lifecycle.discard_transition` | 不拆 | discard 的 saved conflict、preview file cleanup、transient variant cleanup 和 response 属于同一生命周期事务 |
| `record_lifecycle.audit_adapter` | 不拆 | audit helper owner 仍在 graph audit/collaboration 边界，本叶只做条件调用 |
| `record_lifecycle.persistence_adapter` | 不拆 | `list_experiment_records`、`persist_experiment_record`、`persist_backtest_record`、`delete_transient_backtest_record` 仍是共享 persistence owner |

ASCII markers: `save transition`、`persistence adapter`。

---

## 父子通信收口

```text
backend.runtime.routes
  -> crate::runtime::{list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}
  -> runtime.backtest.experiment_sweep
  -> runtime.backtest.experiment_sweep.record_lifecycle
  -> runtime_persistence / runtime_response_mapping / AppState / graph audit helpers
```

本叶只能被父级 `runtime.backtest.experiment_sweep` 私有承载，并只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 experiment lifecycle API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep` 第三轮父叶残余判断 | 默认下一步 | BE-001AB-01 应确认 `parameter_grid`、`start_orchestration`、`record_lifecycle` 均已关闭后，父叶是否还能设置 `stop_split: true` |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 停止 | 本叶已 closeout 并设置 `stop_split: true` |
| `runtime.backtest.experiment_sweep.parameter_grid` | 停止 | 已在 BE-001W-04 closeout 并设置 `stop_split: true` |
| `runtime.backtest.experiment_sweep.start_orchestration` | 停止 | 已在 BE-001Y-04 closeout 并设置 `stop_split: true` |
| `runtime.backtest` 父叶残余 | 暂缓 | 需先完成 `experiment_sweep` 第三轮父叶残余判断 |

---

## 本批次不做

- 不继续细拆 list/detail/save/discard、read/write、save/discard transition 或 audit adapter。
- 不迁移 route registration。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay` 或 `backtest_compare`。
- 不私有化 `list_experiment_records`、`load_experiment_record_from_state`、`persist_experiment_record`、`persist_backtest_record`、`delete_transient_backtest_record`、`sanitize_storage_path_segment`。
- 不私有化 schema、persistence、response mapping、state、audit、frontend caller 或测试资产。
- 不删除 drained parent include。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.record_lifecycle` 已完成时，必须说明只完成四个 experiment record lifecycle handler 的抽离与单叶 closeout，并设置 `stop_split: true`；route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理和重构均未完成。不得宣称 `runtime.backtest.experiment_sweep` 父叶最终完成；下一步只能先进入 BE-001AB-01 父叶残余判断。

---

## 验收标准

1. `124-runtime.backtest.experiment_sweep.record_lifecycle单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.backtest.experiment_sweep.record_lifecycle` closeout 完成并设置 `stop_split: true`。
3. 全量树覆盖本 closeout 文档与 `src/runtime/backtest/record_lifecycle.rs`。
4. 治理门禁能发现本 closeout 文档、`stop_split: true`、BE-001AB-01 父叶残余下一步和禁止迁移边界。
5. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
