# v4.16.0 runtime.backtest.execution_start 父叶残余判断

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001S-01。  
> 基准: `97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md`、`93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`、`89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.execution_start` 父叶残余判断完成。当前不再从本父叶内部继续拆新子叶，下一步回到 `runtime.backtest` 上层队列，优先为 `runtime.backtest.record_store` 建立单子叶等价基线。  
> 代码动作: `no code movement`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001R 之后回到父叶残余判断 | 队列分流 |
| 规范矩阵 | 父叶停止继续私拆 state/record/persistence 边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start` | 残余判断 |
| 模块树 | `runtime.backtest.execution_start` 白箱节点 | 转回上层队列 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start` |
| 父模块 | `runtime.backtest` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 已完成子叶 | `runtime.backtest.execution_start.v4_projection`、`runtime.backtest.execution_start.v4_request_resolution`、`runtime.backtest.execution_start.v4_runtime_execution`、`runtime.backtest.execution_start.legacy_dispatch` |
| 下一候选 | `runtime.backtest.record_store` |
| 下一批次 | BE-001T-01 |

---

## 保留 owner

| owner | 文件 | 本批次处理 |
| --- | --- | --- |
| artifact view owner | `src/backtest_artifacts.rs` | 保留原位 |
| response mapping owner | `src/runtime_response_mapping.rs` | 保留原位 |
| persistence owner | `src/runtime_persistence.rs` | 保留原位 |
| frontend schema owner | `src/frontend_api_types.rs` | 保留原位 |
| parent handler owner | `src/runtime/backtest.rs` | 下一候选来源，当前不移动 |
| app state owner | `AppState` | 保留原位 |
| projection child | `src/runtime/backtest/v4_projection.rs` | 已 closeout |
| request resolution child | `src/runtime/backtest/v4_request_resolution.rs` | 已 closeout |
| runtime execution child | `src/runtime/backtest/v4_runtime_execution.rs` | 已 closeout |
| legacy dispatch child | `src/runtime/backtest/legacy_dispatch.rs` | 已 closeout |

---

## 当前父叶结构

| 残余片段 | 真实职责 | 判定 |
| --- | --- | --- |
| `start_backtest_run` | route handler 外壳，调用 `execute_backtest_request` 并投影 `BacktestRunResponse` | 保留在父叶 |
| `execute_backtest_request` | legacy/v4 分流、capability guard、actor/collaboration、id、governance、event envelope、record assembly、artifact views、transient spill、state write、audit log | 保留为父级兼容桥 |
| `execute_v4_backtest_request` | v4 orchestration、graph expansion、runtime execution、projection、record assembly、artifact views、transient spill、state write、audit log | 保留为父级内部编排 |
| `v4_projection` | v4 artifact 到 `BacktestOutput` / `FrontendRuntimeEvent` 投影 | 已 closeout，`stop_split: true` |
| `v4_request_resolution` | v4 request detection、graph/symbol/event resolution | 已 closeout，`stop_split: true` |
| `v4_runtime_execution` | deterministic bars/ticks、blocking runtime replay 和 `V4BacktestArtifact` 输出 | 已 closeout，`stop_split: true` |
| `legacy_dispatch` | legacy compile/assumption/artifact/sandbox replay | 已 closeout，`stop_split: true` |

---

## 残余候选判断

| 候选 | 判定 | 原因 |
| --- | --- | --- |
| `runtime.backtest.execution_start.record_finalize` | 不在本父叶内继续拆 | 它会同时接触 `BacktestRecord`、`build_backtest_artifact_views`、`maybe_spill_transient_backtest_record`、`state.backtests`、audit log 和 artifact schema owner。若从 `execution_start` 私拆，会制造 state owner 与 record store owner 混淆 |
| `runtime.backtest.execution_start.v4_bridge` | 不继续拆 | 剩余 v4 path 主要是父级 orchestration，继续拆会把 graph expansion、runtime execution、projection、record finalize 重新缠在微文件中 |
| `runtime.backtest.execution_start.legacy_finalize` | 不继续拆 | legacy dispatch 已抽离，剩余部分是 record assembly 与 state write，不应成为 legacy 子叶私有 owner |
| `runtime.backtest.record_store` | 值得进入上层队列 | list/detail/save/discard 与 transient/persistent record 边界在 `src/runtime/backtest.rs` 中仍是清晰 handler 群，适合另起单子叶等价基线 |
| `runtime.backtest.experiment_sweep` | 后续候选 | 当前通过 `execute_backtest_request` 复用桥联动，不应在本父叶残余判断中迁移 |
| `runtime.backtest.replay` | 后续候选 | replay handler 与 response mapping/metrics 需要独立基线 |

---

## 父子通信规则

`runtime.backtest.execution_start` 继续只能通过父级 `runtime.backtest` 与 route facade `backend.runtime.routes.backtest` 暴露创建路径。其内部子模块只允许父级私有调用，不得让 `record_store`、`replay`、`experiment_sweep`、`compare`、`persistence`、`frontend caller` 或其他 sibling 横向直连。发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `src/runtime/backtest/execution_start.rs` 代码。
- 不新增 `src/runtime/backtest/record_finalize.rs`。
- 不迁移 `BacktestRecord`、`BacktestRunResponse`、artifact schema、response schema、state owner、persistence owner 或 frontend caller。
- 不迁移 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay`、`start_backtest_experiment` 或 experiment persistence。
- 不启动发布过渡，不提出 sibling 横向直连。
- 不进入整理或重构阶段。

---

## 下一步

1. 回到 `runtime.backtest` 上层队列。
2. 默认下一批建立 `runtime.backtest.record_store` 单子叶等价基线，即 BE-001T-01。
3. BE-001T-01 只能冻结 list/detail/save/discard、transient/persistent record、audit 和排除边界；不能直接移动代码。

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
git diff --check
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 幻觉检查点

AI 声称 BE-001S-01 完成时，必须说明: 本批次只是 `runtime.backtest.execution_start` 父叶残余判断，`no code movement`，并未迁移 record store、replay、experiment、artifact schema、state owner、persistence owner、frontend caller 或发布过渡。当前结论是 `v4_projection`、`v4_request_resolution`、`v4_runtime_execution`、`legacy_dispatch` 四个子叶已完成 closeout 并设置 `stop_split: true`；父叶残余不继续私拆，下一步回到 `runtime.backtest.record_store` 上层队列。

---

## 验收标准

1. `98-runtime.backtest.execution_start父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `runtime.backtest.execution_start` 父叶残余判断完成，但不宣称 `runtime.backtest` 顶层完成。
3. 下一候选固定为 `runtime.backtest.record_store` / BE-001T-01。
4. 治理门禁能发现本残余判断文档、`no code movement`、下一候选、禁止迁移边界和回归证据缺失。
