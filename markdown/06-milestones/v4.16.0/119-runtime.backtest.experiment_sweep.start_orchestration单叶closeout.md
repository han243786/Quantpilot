# v4.16.0 runtime.backtest.experiment_sweep.start_orchestration 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Y-04。  
> 基准: `118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md`、`117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`、`116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`、`115-runtime.backtest.experiment_sweep父叶残余判断.md`。  
> 判定: `runtime.backtest.experiment_sweep.start_orchestration` 已完成单叶整理 / closeout；本叶设置 `stop_split: true`。下一步应回到 `runtime.backtest.experiment_sweep` 父叶残余判断，默认进入 BE-001Z-01，重新判断 `record_lifecycle` 是否值得另起基线。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Y start_orchestration 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级私有子模块、受控 re-export、父子通信、细分停止条件 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` | 单叶 closeout |
| 模块树 | `runtime.backtest.experiment_sweep.start_orchestration` | 设置停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.start_orchestration` |
| 真实文件 | `src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` |
| public 方法 | `start_backtest_experiment` |
| 父级 re-export | `pub(crate) use start_orchestration::start_backtest_experiment;` |
| 父级模块声明 | `mod start_orchestration;` |
| 私有调用 | `build_experiment_overrides`、`execute_backtest_request` |
| 保留 sibling | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 保留输出类型 | `ExperimentRecord`、`ExperimentVariantSummary`、`ExperimentDetailResponse`、`FrontendRunRequest` |
| closeout 判定 | `stop_split: true` |
| 下一递归点 | BE-001Z-01 `runtime.backtest.experiment_sweep` 父叶残余判断 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 等价 closeout 结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `/api/runtime/experiments` 仍经 `backend.runtime.routes` 调用 `crate::runtime::start_backtest_experiment` |
| 父级出口 | 等价 | `experiment_sweep.rs` 通过 `pub(crate) use start_orchestration::start_backtest_experiment` 保持兼容出口 |
| handler 文件 | 等价 | `start_backtest_experiment` 已独立到 `src/runtime/backtest/start_orchestration.rs` |
| guard 顺序 | 等价 | capability guard、runtime config guard、execution assumption guard、`graph_json` 必填顺序不变 |
| QS compile | 等价 | 仍在参数网格前执行 `compile_runtime_protocol_via_qs(graph_json)` |
| parameter grid | 等价 | 仍只调用 `runtime.backtest.experiment_sweep.parameter_grid::build_experiment_overrides` |
| variant 执行 | 等价 | 每个 variant 仍组装完整 `FrontendRunRequest` 并调用 `execute_backtest_request` |
| summary/tag | 等价 | 仍优先 artifact metrics，缺失时回退 record summary |
| preview persistence | 等价 | 仍先 `persist_experiment_record`，再写 `state.experiments` scoped cache |
| response mapping | 等价 | 仍通过 `experiment_detail_response_from_record(record)` 输出 |
| 外部 owner | 保留 | route、record lifecycle、schema、state、persistence、response mapping、audit、frontend caller 和发布过渡均未迁移 |

---

## 细分价值判断

**最终判定**: `runtime.backtest.experiment_sweep.start_orchestration` 当前不继续细拆，设置 `stop_split: true`。

理由: 本叶只有一个 public handler。它的内部步骤虽然多，但这些步骤都是同一条创建编排流水线的顺序约束: guard、QS compile、参数网格、variant request、execution bridge、record assembly、preview persistence 和 response mapping。继续拆成 guard、variant assembly、preview persistence 或 summary projection 微叶，会增加父级导入面和中间类型，却不会形成新的稳定 owner。真正的 owner 已在外部: `parameter_grid`、`execution_start`、persistence、response mapping、schema、state、audit 和 frontend caller。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `start_orchestration.guard_pipeline` | 不拆 | guard 顺序依赖 handler 错误语义，拆出只会制造无状态微叶 |
| `start_orchestration.protocol_resolution` | 不拆 | `graph_json` 必填、QS compile 和 base assumptions 是创建编排的前置步骤，不形成独立 owner |
| `start_orchestration.variant_request_assembly` | 不拆 | 依赖 request 全量字段和 override 输出，单独成叶会增加中间结构 |
| `start_orchestration.variant_execution_bridge` | 不拆 | 只能调用父级授权的 `execute_backtest_request`，不得横向直连 execution_start 内部 helper |
| `start_orchestration.preview_record_assembly` | 不拆 | 与 `ExperimentRecord` schema、persistence 和 response mapping 强绑定，owner 不在本叶 |
| `start_orchestration.summary_projection` | 不拆 | 只是在 record/artifact 之间选择 summary/tag，不形成 response owner |
| `start_orchestration.persistence_adapter` | 不拆 | `persist_experiment_record` 与 store owner 保持在 persistence 边界 |

---

## 父子通信收口

```text
backend.runtime.routes
  -> crate::runtime::start_backtest_experiment
  -> runtime.backtest.experiment_sweep
  -> runtime.backtest.experiment_sweep.start_orchestration
  -> parameter_grid::build_experiment_overrides
  -> execute_backtest_request
  -> runtime_persistence / runtime_response_mapping / AppState
```

本叶只能被父级 `runtime.backtest.experiment_sweep` 私有承载，并只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 experiment 创建 API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep` 父叶残余判断 | 默认下一步 | BE-001Z-01 应确认 `parameter_grid` 与 `start_orchestration` 均已关闭后，父叶是否还存在值得拆分的 `record_lifecycle` |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 待父叶判断 | 需要重新确认 list/detail/save/discard 与 persistence/audit/state owner 的边界，不能直接移动 |
| `runtime.backtest.experiment_sweep.start_orchestration` | 停止 | 本叶已 closeout 并设置 `stop_split: true` |
| `runtime.backtest.experiment_sweep.parameter_grid` | 停止 | 已在 BE-001W-04 closeout 并设置 `stop_split: true` |
| `runtime.backtest` 父叶残余 | 暂缓 | 需先完成 `experiment_sweep` 父叶残余判断 |

---

## 本批次不做

- 不继续细拆 guard pipeline、protocol resolution、variant request assembly、variant execution bridge、summary projection 或 preview persistence adapter。
- 不迁移 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`。
- 不迁移 `record_lifecycle`；该候选必须等 BE-001Z-01 父叶残余判断后再决定。
- 不迁移 experiment route registration。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay` 或 `backtest_compare`。
- 不私有化 schema、persistence、response mapping、AppState、audit、frontend caller 或测试资产。
- 不删除 drained parent include。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.start_orchestration` 已完成时，必须说明只完成 `start_backtest_experiment` 的物理抽离与单叶 closeout，并设置 `stop_split: true`；record lifecycle、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布版本过渡、整理和重构均未完成。不得宣称 `runtime.backtest.experiment_sweep` 父叶最终完成；下一步只能先进入 BE-001Z-01 父叶残余判断。

---

## 验收标准

1. `119-runtime.backtest.experiment_sweep.start_orchestration单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.backtest.experiment_sweep.start_orchestration` closeout 完成并设置 `stop_split: true`。
3. 全量树覆盖本 closeout 文档与 `src/runtime/backtest/start_orchestration.rs`。
4. 治理门禁能发现本 closeout 文档、`stop_split: true`、BE-001Z-01 父叶残余下一步和禁止迁移边界。
5. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
