# v4.16.0 runtime.backtest.execution_start.v4_runtime_execution 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Q-01。  
> 基准: `81-runtime.backtest.execution_start单叶closeout.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start.v4_runtime_execution` 等价基线，`no code movement`；不迁移代码、不拆 request resolution、不改 projection、不改 record write、不改 artifact schema、不改 response schema、不改 state/persistence/frontend caller。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Q 从 `v4_request_resolution` closeout 回到父叶下一候选基线 | 推进 |
| 规范矩阵 | v4 deterministic replay、runtime execution、artifact output、父子通信、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` | 新增单子叶基线 |
| 模块树 | `runtime.backtest.execution_start.v4_runtime_execution` 白箱候选 | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_runtime_execution` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_request_resolution.rs` |
| 保留父级文件 | `src/runtime/mod.rs`、`src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| 当前 helper/调用群 | `expand_v4_graph_for_symbols`、`build_v4_deterministic_replay_bars`、`V4BacktestTickInput`、`V4PaperSimulatedRuntime::new_for_backtest`、`run_backtest_ticks`、`run_backtest_bars` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 当前真实边界

`runtime.backtest.execution_start.v4_runtime_execution` 只覆盖 v4 backtest 创建路径中 request resolution 之后、projection/record write 之前的 deterministic runtime execution:

1. 父级 `execute_v4_backtest_request` 从 `v4_request_resolution` 取得 `graph`、`symbols` 和 `event_type`。
2. 本候选段调用 `qrpc_runtime::expand_v4_graph_for_symbols`，得到按 symbols 扩展后的 v4 graph。
3. 本候选段调用 `qrpc_runtime::build_v4_deterministic_replay_bars`，以 `symbols`、`now_ms` 和 `event_type` 生成 deterministic bar replay 输入。
4. 若 `request.backtest_options.replay_mode = tick_replay`，本候选段从 bars 派生 `qrpc_runtime::V4BacktestTickInput` 列表；否则 ticks 保持空数组。
5. 本候选段使用 `tokio::task::spawn_blocking` 包住 v4 backtest runtime，避免阻塞 async runtime。
6. `V4PaperSimulatedRuntime::new_for_backtest` 使用 expanded graph、`runtime_simulated_v4_matrix("paper-local")` 和 `ExecutionCapabilityKind::Market` 初始化 runtime。
7. tick replay 分支调用 `run_backtest_ticks`，bar replay 分支调用 `run_backtest_bars`，最终输出 `V4BacktestArtifact`。

本子叶不拥有 v4 request detection、graph/symbol/event resolution、artifact projection、governance event envelope、record write、artifact views、response mapping、state lock、persistence 或 frontend caller。

---

## 输入输出白箱

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `graph` | `resolve_v4_backtest_graph` | `V4MachineGraphContract` | 不改变 request resolution owner 或 static validation |
| `symbols` | `resolve_v4_backtest_symbols` | `Vec<String>` | 不改变 symbol normalize 和 fallback 语义 |
| `event_type` | `resolve_v4_backtest_market_event_type` | `String` | 不改变 market event selection 语义 |
| `now_ms` | parent execution path | `u64` timestamp | 不改变 deterministic replay 时间锚点 |
| `FrontendRunRequest.backtest_options.replay_mode` | request body | optional string | 只识别 `tick_replay`，大小写不敏感 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| expanded graph | v4 runtime constructor | `V4MachineGraphContract` | 不改变 symbol expansion failure mapping |
| deterministic bars | v4 runtime replay | bar replay inputs | 不改变 symbols、event type 或 timestamp 语义 |
| deterministic ticks | v4 runtime replay | `Vec<V4BacktestTickInput>` | tick replay 时按 bars 顺序生成，sequence 从 0 开始 |
| v4 artifact | projection / artifact views | `V4BacktestArtifact` | 不改变 machine trajectory、risk decisions、execution capability source 或 final snapshot 语义 |

---

## 关键调用冻结

| 调用 | 当前职责 | 基线约束 |
| --- | --- | --- |
| `qrpc_runtime::expand_v4_graph_for_symbols` | 按 symbols 扩展 v4 machine graph | 不改变 error mapping，不私有化 qrpc runtime owner |
| `qrpc_runtime::build_v4_deterministic_replay_bars` | 生成 deterministic bar replay 输入 | 不改变 `symbols`、`now_ms`、`event_type` 输入顺序 |
| `qrpc_runtime::V4BacktestTickInput` | tick replay 输入结构 | 不改变 venue、symbol、price、size、ts、sequence、event type 映射 |
| `tokio::task::spawn_blocking` | 承载阻塞 v4 runtime replay | 不改变 async 边界和 cancellation 错误 |
| `qrpc_runtime::V4PaperSimulatedRuntime::new_for_backtest` | 初始化 v4 paper simulated backtest runtime | 不改变 `runtime_simulated_v4_matrix("paper-local")` 或 `ExecutionCapabilityKind::Market` |
| `run_backtest_ticks` | tick replay 执行 | 只在 `tick_replay` 模式调用 |
| `run_backtest_bars` | bar replay 执行 | 非 tick replay 默认路径 |

---

## 错误与兼容桥冻结

| 场景 | 当前错误/路径 | 约束 |
| --- | --- | --- |
| symbol expansion 失败 | `internal_error` | 不改为 bad request，不改变 runtime owner |
| runtime constructor 失败 | `internal_error` | 不吞掉初始化错误 |
| tick replay runtime 失败 | `internal_error(anyhow!(error))` | 不改变 error mapping |
| bar replay runtime 失败 | `internal_error(anyhow!(error))` | 不改变 error mapping |
| blocking task cancelled | `internal_error("v4 backtest task cancelled: {error}")` | 不改变 cancellation 文案语义 |
| 空执行数据 | `v4_backtest_no_execution_data` | 不属于本叶；保留在 projection/父级后置检查 |

---

## 明确排除

- 不迁移 `execute_v4_backtest_request` 整体。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols` 或 `resolve_v4_backtest_market_event_type`。
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 config hash、governance snapshot、capability snapshot event、runtime event envelopes、record write、artifact view、transient spill 或 `state.backtests`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`V4MachineGraphContract`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不引入发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。
- 不删除旧实现，不进入整理/重构阶段。

---

## 第一轮抽离候选方案

下一批如果进入抽离方案，应只允许规划新建 v4 runtime execution 子模块，例如:

```text
src/runtime/backtest/v4_runtime_execution.rs
```

允许迁移候选:

- `qrpc_runtime::expand_v4_graph_for_symbols` 调用段。
- deterministic bars/ticks 构建段。
- `tokio::task::spawn_blocking` 内的 `V4PaperSimulatedRuntime::new_for_backtest`、`run_backtest_ticks`、`run_backtest_bars` 调用段。
- 返回 `V4BacktestArtifact` 的最小 helper。

父级 `runtime.backtest.execution_start` 只能通过父模块私有导入调用该 helper；未来 helper 可见性必须保持 `pub(super)` 或更窄，不得让 projection、request resolution、record store、replay、experiment、frontend caller 或 persistence owner 横向接入。

---

## 暂停点

- 如果抽离需要改变 request resolution helper 或错误 code，则暂停，回到 `v4_request_resolution` closeout 排除边界。
- 如果抽离需要改变 projection helper、`v4_backtest_no_execution_data` 或 frontend event projection，则暂停，回到 `v4_projection` closeout 排除边界。
- 如果抽离需要改变 artifact schema、response schema、event envelope、state lock、persistence 或 frontend caller，则暂停。
- 如果抽离需要把 helper 变为对外 public API，则暂停。
- 如果抽离需要改变 `spawn_blocking`、bar/tick replay 分支或 `V4BacktestArtifact` 语义，则暂停。
- 如果 `api_backtest`、`api_evidence_contract`、`api_run` 或 `cargo test --no-run` 发现回归，则先修复等价缺口。

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

AI 声称 `runtime.backtest.execution_start.v4_runtime_execution` 已建立基线时，必须说明本批 `no code movement`。不得宣称 deterministic replay、`V4PaperSimulatedRuntime`、`run_backtest_ticks`、`run_backtest_bars`、`V4BacktestArtifact` 或任何 helper 已迁移；不得宣称 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 v4 runtime execution 的输入、输出、调用群、错误 mapping、兼容边界和 API 回归证据。
3. 基线明确下一批只能规划 v4 runtime execution 最小 helper 抽离，不能直接移动代码。
4. 基线明确 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生代码移动。
