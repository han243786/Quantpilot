# v4.16.0 runtime.backtest.experiment_sweep 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001V-04。  
> 基准: `109-runtime.backtest.experiment_sweep抽离记录.md`、`108-runtime.backtest.experiment_sweep抽离方案.md`、`107-runtime.backtest.experiment_sweep单子叶等价基线.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: `runtime.backtest.experiment_sweep` 第一轮抽离等价成立；本叶暂不设置 `stop_split: true`。下一轮先为 `runtime.backtest.experiment_sweep.parameter_grid` 建立单子叶等价基线。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001V experiment_sweep 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、route aggregate 保留、shared owner 保留、细分价值判断 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` | 单叶 closeout |
| 模块树 | `runtime.backtest.experiment_sweep` | 标记等价并登记下一内部子叶候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep` |
| 真实文件 | `src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public handler | `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 私有 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` |
| 保留 shared helper | `execute_backtest_request`、`persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state`、`persist_backtest_record`、`delete_transient_backtest_record`、`experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 保留 public 类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`FrontendExecutionAssumptionOverrides`、`FrontendBacktestReplaySource`、`ExperimentRecord`、`ExperimentDefinitionSummary`、`ExperimentVariantSummary`、`ExperimentListItem`、`ExperimentDetailResponse`、`DiscardRuntimeArtifactResponse` |
| closeout 判定 | `stop_split: false`，下一候选 `runtime.backtest.experiment_sweep.parameter_grid` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `/api/runtime/experiments/*` 仍经 `src/backend/runtime/routes.rs -> crate::runtime::*` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `backtest_experiment_sweep` 私有子模块和 `pub(crate)` re-export |
| handler 文件 | 已抽离 | 5 个 experiment handler 已迁入 `src/runtime/backtest/experiment_sweep.rs` |
| 参数网格 | 等价 | 3 个 helper 同迁，保持空网格拒绝、负数校验、去重顺序、base fallback 和 `MAX_EXPERIMENT_VARIANTS` 上限 |
| 创建 sweep | 等价 | capability/config guard、execution assumption guard、compile via QS、variant backtest 复用桥和 preview persistence 均保持 |
| list/detail | 等价 | 仍经 persistence owner 和 response mapping owner 输出列表、分页和详情 |
| save/discard lifecycle | 等价 | variant backtest 持久化、transient 清理、saved conflict、state cleanup 和 graph audit 保持 |
| drained parent include | 保留 | `src/runtime/backtest.rs` 只保留 drained parent include 注释；是否删除归后续父叶残余判断 |
| shared owner | 保留 | route、execution_start、persistence、response mapping、schema、state、audit、frontend caller 和发布过渡均未迁移 |

---

## 细分价值判断

**最终判定**: 本叶暂不设置 `stop_split: true`。下一轮应先进入 `runtime.backtest.experiment_sweep.parameter_grid` 单子叶等价基线。

理由: `experiment_sweep` 抽离后仍包含两类不同责任。一类是 experiment API lifecycle handler，另一类是参数网格的校验、轴归一化、base fallback、去重和 variant 展开。后者输入输出清晰、状态依赖少、风险集中在 `api_experiments`，适合作为下一轮小子叶。相比之下，route aggregate、execution_start 复用桥、persistence、response mapping、schema、state 和 audit 都是外部 owner，不能被本叶私有化。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep.parameter_grid` | 值得继续 | 3 个 helper 共同负责参数网格校验与展开，边界清晰，可用 `api_experiments` 保护 |
| `runtime.backtest.experiment_sweep.start_orchestration` | 暂缓 | `start_backtest_experiment` 仍是 handler 编排层，需先拆出 parameter_grid 后再判断是否值得拆 |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 暂缓 | list/detail/save/discard 与 persistence/audit/state owner 强耦合，先不拆成独立子叶 |
| `runtime.backtest.experiment_sweep.variant_execution` | 不直接拆 | variant 执行依赖 `execute_backtest_request` 父级复用桥，不能形成 sibling 横向连接 |
| `runtime.backtest.experiment_sweep.response_projection` | 不拆 | `experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes` 属于 response mapping owner |
| `runtime.backtest.experiment_sweep.persistence` | 不拆 | experiment/backtest persistence helper 继续归 `src/runtime_persistence.rs` |
| `runtime.backtest.experiment_sweep.audit` | 不拆 | `persist_graph_audit_entry` 与 `build_graph_audit_entry` 属于 graph audit owner |

---

## 父子通信收口

```text
backend.runtime.routes
  -> crate::runtime::{start_backtest_experiment,list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}
  -> runtime::backtest_experiment_sweep
  -> runtime::backtest_execution_start::execute_backtest_request
  -> runtime_persistence / runtime_response_mapping / graph audit / AppState
```

本叶只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 experiment API。不得让 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep.parameter_grid` | 默认下一候选 | 必须先建 BE-001W-01 单子叶等价基线，冻结 empty grid、负数校验、去重、base fallback、variant count 和展开顺序 |
| `runtime.backtest.experiment_sweep.start_orchestration` | 暂缓 | parameter_grid closeout 后再判断 |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 暂缓 | 需要重新确认 persistence/audit/state owner，不得直接移动 |
| `runtime.backtest.experiment_sweep` 父叶残余 | 暂缓 | 需在内部子叶 closeout 后判断 |
| `runtime.backtest` 父叶残余 | 暂缓 | 负责 drained parent include 是否删除，不能混入本 closeout |

---

## 本批次不做

- 不移动 `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides`。
- 不删除 `src/runtime/backtest.rs` drained parent include。
- 不迁移 experiment route registration。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay` 或 `backtest_compare`。
- 不私有化 persistence、response mapping、schema、AppState、audit、frontend caller 或测试资产。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep` closeout 已完成时，必须说明本叶等价成立但 `stop_split: false`，下一候选是 `runtime.backtest.experiment_sweep.parameter_grid`。不得宣称 parameter_grid 已抽离、`src/runtime/backtest.rs` drained parent include 已删除、route aggregate 已迁移、execution_start/persistence/mapping/schema/state/audit/frontend owner 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `110-runtime.backtest.experiment_sweep单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.backtest.experiment_sweep` closeout 完成但 `stop_split: false`。
3. 全量树覆盖本 closeout 文档与下一候选 `runtime.backtest.experiment_sweep.parameter_grid`。
4. 治理门禁能发现本 closeout 文档、`stop_split: false`、下一候选和禁止迁移边界。
5. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
