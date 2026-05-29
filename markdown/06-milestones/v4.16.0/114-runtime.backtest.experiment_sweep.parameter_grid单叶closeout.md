# v4.16.0 runtime.backtest.experiment_sweep.parameter_grid 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001W-04。  
> 基准: `113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md`、`112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md`、`111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`、`110-runtime.backtest.experiment_sweep单叶closeout.md`。  
> 判定: `runtime.backtest.experiment_sweep.parameter_grid` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆，`stop_split: true`。后续应回到 `runtime.backtest.experiment_sweep` 父叶残余判断，默认下一步 BE-001X-01。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001W parameter_grid 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级私有调用、`pub(super)` 可见性、helper cohesive boundary、细分停止条件 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` | 单叶 closeout |
| 模块树 | `runtime.backtest.experiment_sweep.parameter_grid` | 设置停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.parameter_grid` |
| 真实文件 | `src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest.rs`、`src/frontend_api_types.rs` |
| public 方法 | 本节点不新增 public 方法；只保留父级私有 `pub(super)` helper |
| 私有 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis` |
| 父级私有 helper | `build_experiment_overrides` |
| 保留父级方法 | `start_backtest_experiment`、`resolved_backtest_execution_assumptions`、`execute_backtest_request` |
| 保留常量 | `MAX_EXPERIMENT_VARIANTS` |
| 保留类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`RuntimeProtocolCoreConfig`、`FrontendExecutionAssumptionOverrides` |
| closeout 判定 | `stop_split: true` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| 父级调用 | 等价 | `src/runtime/backtest/experiment_sweep.rs` 仅通过 `use parameter_grid::build_experiment_overrides` 调用 |
| 可见性 | 等价 | `build_experiment_overrides` 只以 `pub(super)` 暴露给父级，未新增 public API 或 crate re-export |
| 文件位置 | 等价 | `src/runtime/backtest/parameter_grid.rs` 是 Rust 对父模块片段 `mod parameter_grid;` 的真实解析路径 |
| empty grid | 等价 | 仍通过 provided value count 拒绝空参数网格 |
| 负数校验 | 等价 | `fee_bps` / `slippage_bps` 仍返回 `parameter_grid.{field} 必须 >= 0` |
| base fallback | 等价 | 空 fee/slippage/latency 轴继续回退到 base execution assumptions |
| dedupe | 等价 | 继续使用 `Vec::contains`，保持原输入顺序去重 |
| variant count | 等价 | 继续使用 fee × slippage × latency，并受 `MAX_EXPERIMENT_VARIANTS` 限制 |
| expansion order | 等价 | 保持 fee 外层、slippage 中层、latency 内层 |
| 输出结构 | 等价 | 每个 variant 继续输出 `Some(fee_bps)`、`Some(slippage_bps)`、`Some(latency_ms)` |
| 外部 owner | 保留 | schema、route、handler orchestration、execution_start、persistence、response mapping、state、audit、frontend caller 均未迁移 |

---

## 细分价值判断

**最终判定**: `runtime.backtest.experiment_sweep.parameter_grid` 当前不继续细拆，设置 `stop_split: true`。

理由: 本叶只有 3 个 helper，且共同完成同一条参数网格流水线: 轴归一化、base fallback、去重、variant 上限和展开输出。继续拆成 axis/expansion/error 子叶会增加文件与父级导入面，却不会形成新的 owner。真正值得保护的边界已经由本叶白箱输入输出和 `api_experiments` 覆盖；schema、常量、handler、route、state/persistence 和 frontend caller 都是外部 owner，不能被本叶私有化。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `parameter_grid.float_axis` | 不拆 | fee/slippage 共享同一个 helper，拆开会复制负数校验与 base fallback 语义 |
| `parameter_grid.latency_axis` | 不拆 | 单 helper，逻辑极小，独立成叶只会增加文件噪音 |
| `parameter_grid.variant_expansion` | 不拆 | 与 axis 输出、variant count 和 `FrontendExecutionAssumptionOverrides` 强绑定，保留在同一白箱更清楚 |
| `parameter_grid.error_adapter` | 不拆 | 只调用父级 `json_bad_request`，没有独立 owner |
| `parameter_grid.limit_policy` | 不拆 | `MAX_EXPERIMENT_VARIANTS` 保持父级常量 owner，不私有化 |
| `parameter_grid.schema` | 不拆 | `FrontendExecutionAssumptionSweepGrid` 与 `FrontendExecutionAssumptionOverrides` 属于 `src/frontend_api_types.rs` |

---

## 父子通信收口

```text
runtime.backtest.experiment_sweep
  -> parameter_grid::build_experiment_overrides
  -> normalize_experiment_float_axis / normalize_experiment_latency_axis
  -> FrontendExecutionAssumptionOverrides
```

本叶只能被父级 `runtime.backtest.experiment_sweep` 私有调用。不得让 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep` 父叶残余判断 | 默认下一步 | BE-001X-01 应确认 parameter_grid 关闭后父叶是否还存在值得拆分的内部子叶 |
| `runtime.backtest.experiment_sweep.start_orchestration` | 待父叶判断 | 只有父叶残余判断确认值得拆，才能另起基线 |
| `runtime.backtest.experiment_sweep.record_lifecycle` | 待父叶判断 | 需重新确认 persistence/audit/state owner，不能直接移动 |
| `runtime.backtest.experiment_sweep.parameter_grid` | 停止 | 本叶已 closeout 并设置 `stop_split: true` |
| `runtime.backtest` 父叶残余 | 暂缓 | 需先完成 `experiment_sweep` 父叶残余判断 |

---

## 本批次不做

- 不继续细拆 axis normalization、variant expansion、error adapter 或 limit policy。
- 不迁移 `start_backtest_experiment` 或其他 experiment handler。
- 不迁移 experiment route registration。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay` 或 `backtest_compare`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不私有化 schema、persistence、response mapping、state、audit、frontend caller 或测试资产。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.parameter_grid` 已完成时，必须说明只完成 3 个 helper 的抽离与 closeout，并设置 `stop_split: true`；`start_backtest_experiment`、route aggregate、execution_start、schema、state、persistence、response mapping、audit、frontend caller、发布版本过渡、整理和重构均未完成。不得宣称 experiment_sweep 父叶已经完成最终残余判断。

---

## 验收标准

1. `114-runtime.backtest.experiment_sweep.parameter_grid单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.backtest.experiment_sweep.parameter_grid` closeout 完成并设置 `stop_split: true`。
3. 全量树覆盖本 closeout 文档与 `src/runtime/backtest/parameter_grid.rs`。
4. 治理门禁能发现本 closeout 文档、`stop_split: true`、父级残余下一步和禁止迁移边界。
5. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
