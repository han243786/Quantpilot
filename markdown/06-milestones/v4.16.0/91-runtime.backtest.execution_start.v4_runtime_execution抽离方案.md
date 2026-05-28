# v4.16.0 runtime.backtest.execution_start.v4_runtime_execution 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Q-02。  
> 基准: `90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`、`89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start.v4_runtime_execution` 抽离方案，`no code movement`；下一批若实施，只允许迁移 deterministic replay、v4 runtime execution 和 `V4BacktestArtifact` 输出 helper，不得混入 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Q 从 v4 runtime execution 基线进入抽离方案 | 推进 |
| 规范矩阵 | runtime execution 最小移动、父级私有调用、blocking/cancellation/error mapping 保持 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` | 抽离方案 |
| 模块树 | `runtime.backtest.execution_start.v4_runtime_execution` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_runtime_execution` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| 下一批计划目标 | future parent-private v4_runtime_execution module under `src/runtime/backtest` |
| 父级导入策略 | 在 `execution_start.rs` 中用 path module 接入 `v4_runtime_execution`，只由父模块调用 |
| 对外 API 策略 | 不新增 public API；候选 helper 使用 `pub(super)`，不得扩大到 `pub(crate)` 或更宽 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 适配性校验

当前 `src/runtime/backtest/execution_start.rs` 中，v4 runtime execution 已形成 request resolution 之后、projection/record write 之前的连续区域:

| 段落 | 当前职责 | 方案归属 |
| --- | --- | --- |
| `qrpc_runtime::expand_v4_graph_for_symbols` | 按 resolved symbols 扩展 v4 machine graph | 可迁移 |
| `qrpc_runtime::build_v4_deterministic_replay_bars` | 生成 deterministic bar replay 输入 | 可迁移 |
| `tick_replay` 判定与 `V4BacktestTickInput` 构造 | 根据 replay mode 派生 tick replay 输入 | 可迁移 |
| `tokio::task::spawn_blocking` | 将 v4 paper simulated runtime replay 放入 blocking task | 可迁移 |
| `V4PaperSimulatedRuntime::new_for_backtest` | 用 expanded graph、`runtime_simulated_v4_matrix("paper-local")` 和 `ExecutionCapabilityKind::Market` 初始化 runtime | 可迁移 |
| `run_backtest_ticks` / `run_backtest_bars` | 执行 tick 或 bar replay | 可迁移 |
| `V4BacktestArtifact` | 返回后交给 projection 与 record write | 可迁移为 helper 输出 |

该连续区域不拥有 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`，也不拥有 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact`、`v4_backtest_no_execution_data`、record write、artifact views、event envelope、state lock 或 persistence。

---

## 抽离目标

下一批实际抽离只允许做以下结构性移动:

1. 新建 future parent-private v4_runtime_execution module under `src/runtime/backtest`。
2. 在 `src/runtime/backtest/execution_start.rs` 内用 path module 接入 `v4_runtime_execution`。
3. 从 `execute_v4_backtest_request` 中移入 deterministic runtime execution 连续段。
4. 暴露一个父级私有 helper，例如 `run_v4_backtest_runtime_execution`，可见性为 `pub(super)`。
5. helper 输入只允许来自已完成 request resolution 的 `graph`、`symbols`、`event_type`，以及父级创建路径已有的 `now_ms` 和 `tick_replay` 判定。
6. helper 输出只允许是 `V4BacktestArtifact`，继续交还父级执行 projection、no data check、event envelope、record write 和 transient spill。
7. 保持 `execute_v4_backtest_request` 的 request resolution、projection、record write 和 state 写入顺序不变。

建议 helper 形态:

```rust
pub(super) async fn run_v4_backtest_runtime_execution(
    graph: &V4MachineGraphContract,
    symbols: &[String],
    event_type: &str,
    now_ms: u64,
    tick_replay: bool,
) -> Result<qrpc_runtime::V4BacktestArtifact, (StatusCode, String)> {
    // Move only deterministic replay/runtime execution here.
}
```

父级保留形态:

```rust
let graph = resolve_v4_backtest_graph(graph_json)?;
let symbols = resolve_v4_backtest_symbols(request, graph_json, &graph);
let event_type = resolve_v4_backtest_market_event_type(&expanded_graph)?;
let tick_replay = request.backtest_options.replay_mode...;
let v4_artifact = run_v4_backtest_runtime_execution(
    &graph,
    &symbols,
    &event_type,
    now_ms,
    tick_replay,
).await?;
```

实际实现时若 `event_type` 依赖 expanded graph，允许 helper 负责先 expand graph 再 resolve event type 的顺序优化讨论，但必须暂停复核，因为这会触碰 `resolve_v4_backtest_market_event_type` 的 owner。默认方案是保持 event type resolution owner 不动。

---

## 允许迁移清单

| 代码段 | 迁移原因 | 可见性策略 |
| --- | --- | --- |
| expanded graph 构造 | v4 runtime execution 的输入适配 | 子模块私有 |
| deterministic bars 构造 | v4 runtime execution 的 replay input 生成 | 子模块私有 |
| tick replay 输入构造 | v4 runtime execution 的 replay mode 分支 | 子模块私有 |
| blocking task runtime 初始化 | v4 runtime execution 的阻塞执行边界 | 子模块私有 |
| `run_backtest_ticks` / `run_backtest_bars` 调用 | v4 runtime execution 的核心执行 | 子模块私有 |
| `run_v4_backtest_runtime_execution` helper | 父模块调用入口 | `pub(super)` |

---

## 必须保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request`、request resolution 调用、projection 调用、record write | 创建路径父模块不迁移 |
| `src/runtime/backtest/v4_request_resolution.rs` | `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` | request resolution 子叶已 closeout，不回流、不混入 runtime execution |
| `src/runtime/backtest/v4_projection.rs` | `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` | projection 子叶已 closeout，不回流、不混入 runtime execution |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、transient spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` 与 response mapping | response schema owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API 类型 | schema owner 不迁移 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 错误与兼容桥保持

| 场景 | 当前错误/路径 | 约束 |
| --- | --- | --- |
| symbol expansion 失败 | `internal_error` | 不改为 bad request，不吞掉 runtime owner 错误 |
| runtime constructor 失败 | `internal_error` | 不改变 `runtime_simulated_v4_matrix("paper-local")` 或 `ExecutionCapabilityKind::Market` |
| tick replay runtime 失败 | `internal_error(anyhow!(error))` | 不改变 error mapping |
| bar replay runtime 失败 | `internal_error(anyhow!(error))` | 不改变 error mapping |
| blocking task cancelled | `internal_error("v4 backtest task cancelled: {error}")` | 不改变 cancellation 文案语义 |
| 空执行数据 | `v4_backtest_no_execution_data` | 不属于本叶；继续保留在 projection/父级后置检查 |

---

## 明确排除

- 不迁移 `execute_v4_backtest_request` 整体。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols` 或 `resolve_v4_backtest_market_event_type`。
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 config hash、governance snapshot、capability snapshot event、runtime event envelopes、record write、artifact view、transient spill 或 `state.backtests`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`V4MachineGraphContract`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不新增发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 中止条件

下一批实际抽离只要出现以下任一情况，必须中止并回到方案讨论:

1. 需要改变 request resolution helper、错误码或 owner。
2. 需要改变 projection helper、`v4_backtest_no_execution_data` 或 frontend event projection。
3. 需要改变 artifact schema、response schema、event envelope、state lock、persistence IO 或 frontend caller。
4. 需要把 helper 变为 `pub(crate)` 或更宽的 public API。
5. 需要让 request resolution、projection、record store、replay、experiment、compare 或 frontend caller 直接调用 runtime execution 子模块。
6. 需要改变 `spawn_blocking`、bar/tick replay 分支、`V4PaperSimulatedRuntime` 初始化参数或 `V4BacktestArtifact` 语义。
7. `api_backtest`、`api_evidence_contract`、`api_run` 或 `cargo test --no-run` 出现行为回归。

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

## 下一步

下一批进入 `BE-001Q-03 runtime.backtest.execution_start.v4_runtime_execution 抽离记录`。实施范围只能是:

1. 新建 v4 runtime execution 子模块。
2. 移入允许迁移清单中的 deterministic runtime execution helper。
3. 在父级 `execution_start.rs` 私有导入一个入口 helper。
4. 保持 request resolution、projection、record write、artifact、response、persistence、schema、state、frontend 和发布过渡边界不变。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_runtime_execution` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 deterministic replay、`V4PaperSimulatedRuntime`、`run_backtest_ticks`、`run_backtest_bars`、`V4BacktestArtifact` 或任何 helper 已迁移；不得宣称 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、整理、重构或发布过渡已经完成。

---

## 验收标准

1. `91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确下一批只允许迁移 deterministic runtime execution 最小 helper。
3. 方案明确父级只私有导入 `run_v4_backtest_runtime_execution` 入口 helper。
4. 方案明确 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生代码移动。
