# v4.16.0 runtime.backtest.execution_start.v4_runtime_execution 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Q-03。  
> 基准: `91-runtime.backtest.execution_start.v4_runtime_execution抽离方案.md`、`90-runtime.backtest.execution_start.v4_runtime_execution单子叶等价基线.md`、`89-runtime.backtest.execution_start.v4_request_resolution单叶closeout.md`、`85-runtime.backtest.execution_start.v4_projection单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批完成 `runtime.backtest.execution_start.v4_runtime_execution` 第一轮物理抽离；只迁移 deterministic bars/ticks、`tokio::task::spawn_blocking`、`V4PaperSimulatedRuntime::new_for_backtest`、`run_backtest_ticks`、`run_backtest_bars` 和 `V4BacktestArtifact` 输出 helper，不迁移 request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Q 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | runtime execution helper 父级私有导入、blocking/cancellation/error mapping 保持 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` | 物理抽离 |
| 模块树 | `runtime.backtest.execution_start.v4_runtime_execution` 白箱节点 | 补真实文件 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_runtime_execution` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_runtime_execution` |
| 父模块 | `runtime.backtest.execution_start` |
| 新真实文件 | `src/runtime/backtest/v4_runtime_execution.rs` |
| 父级文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| public API | 无新增 public API；新 helper 为 `pub(super)` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 实际改动

| 文件 | 改动 | 边界 |
| --- | --- | --- |
| `src/runtime/backtest/v4_runtime_execution.rs` | 新建 v4 runtime execution 子模块 | 只承载 deterministic bars/ticks、blocking runtime replay 和 `V4BacktestArtifact` 输出 |
| `src/runtime/backtest/execution_start.rs` | 新增 path module 与父级私有导入 `run_v4_backtest_runtime_execution` | 保留 `execute_v4_backtest_request`、request resolution、projection、record write 和 state 写入 |

父级私有导入形态:

```rust
#[path = "v4_runtime_execution.rs"]
mod v4_runtime_execution;

use v4_runtime_execution::run_v4_backtest_runtime_execution;
```

子模块入口形态:

```rust
pub(super) async fn run_v4_backtest_runtime_execution(
    expanded_graph: qrpc_core_ir::v4::V4MachineGraphContract,
    symbols: &[String],
    event_type: &str,
    now_ms: u64,
    tick_replay: bool,
) -> Result<qrpc_core_ir::v4::V4BacktestArtifact, (StatusCode, String)>
```

---

## 已迁移清单

| 代码段 | 新位置 | 可见性 |
| --- | --- | --- |
| `qrpc_runtime::build_v4_deterministic_replay_bars` 调用段 | `src/runtime/backtest/v4_runtime_execution.rs` | 子模块内部 |
| `qrpc_runtime::V4BacktestTickInput` tick replay 构造段 | `src/runtime/backtest/v4_runtime_execution.rs` | 子模块内部 |
| `tokio::task::spawn_blocking` runtime replay 段 | `src/runtime/backtest/v4_runtime_execution.rs` | 子模块内部 |
| `V4PaperSimulatedRuntime::new_for_backtest` 初始化段 | `src/runtime/backtest/v4_runtime_execution.rs` | 子模块内部 |
| `run_backtest_ticks` / `run_backtest_bars` 分支 | `src/runtime/backtest/v4_runtime_execution.rs` | 子模块内部 |
| `run_v4_backtest_runtime_execution` | `src/runtime/backtest/v4_runtime_execution.rs` | `pub(super)` |

---

## 保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `execute_v4_backtest_request`、backtest id、actor/collaboration、config hash、governance、projection、record write、artifact view、transient spill、state write | 创建路径父模块不迁移 |
| `src/runtime/backtest/execution_start.rs` | `qrpc_runtime::expand_v4_graph_for_symbols` 调用和 `resolve_v4_backtest_market_event_type` 调用顺序 | `event_type` 当前依赖 expanded graph；本批避免让 runtime execution 子模块横向调用 request resolution sibling |
| `src/runtime/backtest/v4_request_resolution.rs` | `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` | request resolution 子叶已 closeout，不回流、不混入 runtime execution |
| `src/runtime/backtest/v4_projection.rs` | `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` | projection 子叶已 closeout，不回流、不混入 runtime execution |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、transient spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` 与 response mapping | response schema owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API 类型 | schema owner 不迁移 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 等价约束

- `execute_v4_backtest_request` 的 request resolution 顺序不变。
- `expand_v4_graph_for_symbols` 的调用位置暂留父级，错误仍通过 `internal_error` 映射。
- `resolve_v4_backtest_market_event_type` 仍由 request resolution 子叶提供，且仍在 expanded graph 上解析。
- `v4_backtest_no_execution_data` 仍保留在父级/projection 后置检查，不进入本子叶。
- `tick_replay` 判定、大小写不敏感规则和默认 false 语义不变。
- deterministic bars 仍使用 `symbols`、`now_ms`、`event_type` 构造。
- deterministic ticks 仍按 bars 顺序生成，`sequence` 从 0 开始，`price = bar.close`，`size = 1.0`，`event_type` 逐 tick 保持一致。
- `tokio::task::spawn_blocking`、`runtime_simulated_v4_matrix("paper-local")`、`ExecutionCapabilityKind::Market`、`run_backtest_ticks` / `run_backtest_bars` 分支语义不变。
- `v4 backtest task cancelled: {error}` cancellation 文案语义不变。
- `V4BacktestArtifact` schema 不变，并继续交还父级进入 projection、no data check、event envelope 和 record write。
- 不新增 public API，不新增 sibling 横向连接，不新增发布版本过渡、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 明确排除

- 不迁移 `execute_v4_backtest_request` 整体。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols` 或 `resolve_v4_backtest_market_event_type`。
- 不迁移 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或 `src/runtime/backtest/v4_projection.rs`。
- 不迁移 config hash、governance snapshot、capability snapshot event、runtime event envelopes、record write、artifact view、transient spill 或 `state.backtests`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`V4MachineGraphContract`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
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

## 下一步

下一批进入 `BE-001Q-04 runtime.backtest.execution_start.v4_runtime_execution 单叶 closeout`。closeout 必须判断:

1. 新 `run_v4_backtest_runtime_execution` helper 的父级私有子模块等价是否成立。
2. `expand_v4_graph_for_symbols` 保留在父级是否是当前正确边界，还是需要另起更低层基线讨论。
3. 本叶是否值得继续细拆成 replay input、blocking execution 或 artifact output 微叶。
4. 若不继续细拆，是否回到 `runtime.backtest.execution_start` 父叶选择下一候选，而不是直接迁移 record/replay/experiment/schema/state/persistence/frontend。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_runtime_execution` 已抽离时，必须说明只迁移了 deterministic bars/ticks、blocking runtime replay 和 `run_v4_backtest_runtime_execution` helper；`expand_v4_graph_for_symbols` 仍保留在父级 `execute_v4_backtest_request` 内，request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、整理、重构和发布过渡均未完成。

---

## 验收标准

1. `92-runtime.backtest.execution_start.v4_runtime_execution抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/v4_runtime_execution.rs` 存在并承载 `run_v4_backtest_runtime_execution`。
3. `src/runtime/backtest/execution_start.rs` 只通过父级私有导入调用该 helper。
4. request resolution、projection、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生发布版本过渡连接。
