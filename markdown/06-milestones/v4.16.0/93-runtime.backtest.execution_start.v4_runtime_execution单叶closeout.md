# v4.16.0 runtime.backtest.execution_start.v4_runtime_execution 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Q-04。  
> 基准: `90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`、`91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md`、`92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md`、`89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.execution_start.v4_runtime_execution` 已完成等价 closeout，并设置 `stop_split: true`。本叶不继续细拆；后续若继续，应回到父叶 `runtime.backtest.execution_start` 另起候选基线，不能从 runtime execution 子叶继续外扩。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Q 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级私有 helper、blocking/cancellation/error mapping 等价、`stop_split: true`、禁止横向连接 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` | closeout |
| 模块树 | `runtime.backtest.execution_start.v4_runtime_execution` 白箱节点 | 更新状态与下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_runtime_execution` |
| 父模块 | `runtime.backtest.execution_start` |
| 真实文件 | `src/runtime/backtest/v4_runtime_execution.rs`、`src/runtime/backtest/execution_start.rs` |
| sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs` |
| 保留 owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| 关键 public 方法 | 无新增 public API；父级只通过 `pub(super)` helper 调用 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树、UTF-8、diff check |

---

## 等价 closeout 结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| 父级调用 | 等价 | `execution_start.rs` 只私有导入 `run_v4_backtest_runtime_execution`，并在 `execute_v4_backtest_request` 内调用 |
| deterministic bars | 等价 | `build_v4_deterministic_replay_bars(symbols, now_ms, event_type)` 输入不变 |
| tick replay | 等价 | `tick_replay` 判定仍由父级从 request body 解析，ticks 仍按 bars 顺序生成，`sequence` 从 0 开始 |
| blocking 边界 | 等价 | `tokio::task::spawn_blocking` 仍包住 `V4PaperSimulatedRuntime` replay |
| runtime 初始化 | 等价 | `V4PaperSimulatedRuntime::new_for_backtest` 仍使用 expanded graph、`runtime_simulated_v4_matrix("paper-local")` 和 `ExecutionCapabilityKind::Market` |
| replay 分支 | 等价 | `tick_replay` 调 `run_backtest_ticks`，默认路径调 `run_backtest_bars` |
| cancellation/error mapping | 等价 | runtime error 仍经 `internal_error(anyhow!(error))`，task cancelled 文案仍为 `v4 backtest task cancelled: {error}` |
| artifact 输出 | 等价 | `V4BacktestArtifact` 继续交还父级 projection、`v4_backtest_no_execution_data` 后置检查、event envelope 和 record write |
| owner 边界 | 保留 | expanded graph、request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 均未迁移 |
| 发布过渡 | 未启动 | `release transition guard` 生效；未新增横向直连、缓存旁路或性能优化提案 |

---

## 当前白箱结构

| helper / 代码段 | 当前 owner | 细分判断 |
| --- | --- | --- |
| `run_v4_backtest_runtime_execution` | `src/runtime/backtest/v4_runtime_execution.rs` | 保留在本叶。它是唯一父级私有入口，继续拆会增加微文件和导入面 |
| deterministic bars 构造 | `src/runtime/backtest/v4_runtime_execution.rs` | 保留在 helper 内。只服务本 helper 的 bar/tick replay |
| deterministic ticks 构造 | `src/runtime/backtest/v4_runtime_execution.rs` | 保留在 helper 内。只服务 tick replay branch |
| blocking runtime replay | `src/runtime/backtest/v4_runtime_execution.rs` | 保留在 helper 内。它与 bars/ticks 输入共享同一 artifact 输出语义 |
| `expand_v4_graph_for_symbols` | `src/runtime/backtest/execution_start.rs` | 暂留父级。`event_type` 当前依赖 expanded graph，迁移它需要另起边界复核 |

---

## 冻结符号表

| 类别 | 符号 | closeout 约束 |
| --- | --- | --- |
| 父级 helper | `run_v4_backtest_runtime_execution` | 只能是 `pub(super)` |
| graph 输入 | `V4MachineGraphContract` | expanded graph 语义不变 |
| replay 输入 | `build_v4_deterministic_replay_bars` | symbols、now_ms、event_type 顺序不变 |
| replay 输入 | `V4BacktestTickInput` | venue、symbol、price、size、ts、sequence、event type 映射不变 |
| async 边界 | `tokio::task::spawn_blocking` | blocking replay 边界不变 |
| runtime | `V4PaperSimulatedRuntime::new_for_backtest` | runtime matrix 和 capability source 不变 |
| runtime matrix | `runtime_simulated_v4_matrix("paper-local")` | 不改本地 paper runtime matrix |
| capability | `ExecutionCapabilityKind::Market` | 不改 execution capability 来源 |
| replay branch | `run_backtest_ticks` | 只在 `tick_replay` 模式调用 |
| replay branch | `run_backtest_bars` | 非 tick replay 默认路径 |
| artifact | `V4BacktestArtifact` | schema 与 projection 输入不变 |
| post execution check | `v4_backtest_no_execution_data` | 保留在父级/projection 后置检查 |
| schema owner | `BacktestOutput` | 不在本叶迁移或改 schema |
| schema owner | `BacktestRunResponse` | 不在本叶迁移或改 schema |
| schema owner | `BacktestRecord` | 不在本叶迁移或改 schema |
| frontend event | `FrontendRuntimeEvent` | 不在本叶迁移或改事件 schema |
| runtime event | `RuntimeEventEnvelope` | 不在本叶迁移或改 envelope schema |

---

## 细分价值判断

**最终判定**: `stop_split: true`。

理由:

1. 本叶没有 state、IO、锁、route、persistence、schema owner 或外部 API。
2. `run_v4_backtest_runtime_execution` 已经是唯一父级私有入口；继续拆成 replay input、blocking execution、artifact output 会制造微文件和更宽导入面。
3. deterministic bars/ticks 与 blocking runtime replay 共同维护一个 `V4BacktestArtifact` 输出语义，拆开不能带来真实解耦收益。
4. `expand_v4_graph_for_symbols` 保留在父级是当前正确边界；若未来要将 expanded graph 也并入 runtime execution，必须另起基线，因为它与 `resolve_v4_backtest_market_event_type` 的 expanded graph 输入顺序绑定。
5. 本叶不允许被 request resolution、projection、record store、replay、experiment、compare、persistence 或 frontend caller 横向直连。

---

## 后续递归队列

| 顺序 | 候选 | 进入条件 |
| --- | --- | --- |
| 1 | `runtime.backtest.execution_start.legacy_dispatch` | 若继续父叶内部递归，可另起基线评估 legacy non-v4 path 的 compile/sandbox dispatch；不得混入 v4 子叶或 record write |
| 2 | `runtime.backtest.execution_start.record_write_bridge` | 当前不进入；涉及 artifact views、transient spill、state owner 和 persistence owner，必须另起决策暂停 |
| 3 | `runtime.backtest.record_store` | 只有当 execution_start 内部值得拆的候选完成或暂停后，才回到 `runtime.backtest` sibling 队列 |
| 4 | `backend.runtime` 父叶 closeout | 只有 runtime backtest/run/event stream 当前队列完成或暂停后，才允许评估父叶阶段性 closeout |

---

## 本批次不做

- 不迁移 `execute_v4_backtest_request` 整体。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `expand_v4_graph_for_symbols`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols` 或 `resolve_v4_backtest_market_event_type`。
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 config hash、governance snapshot、capability snapshot event、runtime event envelopes、record write、artifact view、transient spill 或 `state.backtests`。
- 不继续拆 `run_v4_backtest_runtime_execution`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不进入整理、重构、发布版本过渡或性能连接优化。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_runtime_execution` 已 closeout 时，必须说明: 本叶只完成 deterministic bars/ticks、blocking runtime replay 和 `run_v4_backtest_runtime_execution` helper 的等价 closeout，并设置 `stop_split: true`。`expand_v4_graph_for_symbols` 仍保留在父级 `execute_v4_backtest_request` 内；不得宣称 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.backtest.execution_start.v4_runtime_execution` 已 closeout，并设置 `stop_split: true`。
3. `run_v4_backtest_runtime_execution` 的父级私有调用、blocking/error mapping、artifact 输出和排除边界均有白箱登记。
4. 下一候选回到父叶 `runtime.backtest.execution_start`，后续必须另起等价基线。
5. 治理门禁能发现 closeout 文档、`stop_split: true`、禁止迁移边界、下一候选和回归证据缺失。
