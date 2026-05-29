# v4.16.0 runtime.backtest.execution_start.legacy_dispatch 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001R-03。  
> 基准: `95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md`、`94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`、`93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批完成 `runtime.backtest.execution_start.legacy_dispatch` 第一轮物理抽离；只迁移 legacy non-v4 path 的 QS compile、execution assumption override、compile artifact bundle、blocking `FastBacktestSandbox` replay 和轻量输出结构，不迁移 record assembly、artifact views、transient spill、state write、audit log、schema owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001R 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | legacy dispatch helper 父级私有导入、两段式父级编排、record write 保持原位 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` | 物理抽离 |
| 模块树 | `runtime.backtest.execution_start.legacy_dispatch` 白箱节点 | 补真实文件 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.legacy_dispatch` |
| 父模块 | `runtime.backtest.execution_start` |
| 新真实文件 | `src/runtime/backtest/legacy_dispatch.rs` |
| 父级文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_runtime_execution.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| public API | 无新增 public API；新增入口均为父模块可见 `pub(super)` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 实际改动

| 文件 | 改动 | 边界 |
| --- | --- | --- |
| `src/runtime/backtest/legacy_dispatch.rs` | 新建 legacy dispatch 子模块 | 只承载 legacy compile/assumption/artifact/sandbox replay 和轻量输出 |
| `src/runtime/backtest/execution_start.rs` | 新增 path module 与父级私有导入 | 保留 validation、v4 bridge、actor/collaboration、id、governance、event envelope、record assembly、artifact views、spill、state write 和 audit log |

父级私有导入形态:

```rust
#[path = "legacy_dispatch.rs"]
mod legacy_dispatch;

use legacy_dispatch::{
    prepare_legacy_backtest_dispatch, run_legacy_backtest_dispatch, LegacyBacktestDispatchOutput,
};
```

子模块入口形态:

```rust
pub(super) fn prepare_legacy_backtest_dispatch(
    graph_json: &Value,
    request: &FrontendRunRequest,
) -> Result<LegacyBacktestDispatchPlan, (StatusCode, String)>

pub(super) async fn run_legacy_backtest_dispatch(
    plan: LegacyBacktestDispatchPlan,
    request: &FrontendRunRequest,
    now_ms: u64,
) -> Result<LegacyBacktestDispatchOutput, (StatusCode, String)>
```

---

## 已迁移清单

| 代码段 | 新位置 | 可见性 |
| --- | --- | --- |
| `compile_runtime_protocol_via_qs` legacy QS compile 调用 | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `apply_backtest_execution_assumption_overrides` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `compile_runtime_protocol_config` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `resolved_backtest_execution_assumptions` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `resolved_execution_assumption_sources` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `build_compile_artifact_bundle` legacy artifact bundle 调用 | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `FrontendBacktestReplaySource` replay source branch | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `FastBacktestSandbox::with_replay_from_core_ir` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `DeterministicTestMode::replay_defaults` / `BACKTEST_DETERMINISTIC_SEED` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `tokio::task::spawn_blocking` legacy replay closure | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `sandbox.start` / `sandbox.run_backtest` | `src/runtime/backtest/legacy_dispatch.rs` | 子模块内部 |
| `LegacyBacktestDispatchPlan` / `LegacyBacktestDispatchOutput` | `src/runtime/backtest/legacy_dispatch.rs` | `pub(super)` |

---

## 保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request`、capability guard、runtime config guard、`graph_json` 必填校验、v4 fallback bridge | 创建路径父模块不迁移 |
| `src/runtime/backtest/execution_start.rs` | `now_ms`、actor/collaboration、graph targets、runtime targets、backtest id、governance、capability snapshot event、runtime event envelopes、account summary、backtest spec、record construction、artifact views、transient spill、`state.backtests`、audit log | record assembly 与 state owner 保持父级 |
| `src/runtime/backtest/v4_request_resolution.rs` | v4 detection、graph/symbol/event resolution | sibling 已 closeout，不回流 |
| `src/runtime/backtest/v4_projection.rs` | v4 artifact projection 与 frontend events | sibling 已 closeout，不回流 |
| `src/runtime/backtest/v4_runtime_execution.rs` | v4 deterministic runtime execution | sibling 已 closeout，不回流 |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` | response mapping owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API schema | schema owner 不迁移 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 等价约束

- `execute_backtest_request` 的 guard、`graph_json` 必填校验和 v4 fallback bridge 顺序不变。
- legacy path 使用两段式父级编排: `prepare_legacy_backtest_dispatch` 先完成 compile/assumption resolution，父级随后执行 actor/collaboration，再调用 `run_legacy_backtest_dispatch` 完成 artifact bundle 与 sandbox replay。
- `protocol_name`、`config_hash` 仍在父级从 compiled protocol 克隆，并继续用于 governance snapshot 与 record assembly。
- `now_ms` 仍由父级 `execution_start.rs` 产生，并传入 legacy dispatch 子模块作为 replay/default seed 时间锚点。
- `compile_runtime_protocol_via_qs`、`apply_backtest_execution_assumption_overrides`、`compile_runtime_protocol_config`、`resolved_backtest_execution_assumptions`、`resolved_execution_assumption_sources` 的输入输出语义不变。
- `build_compile_artifact_bundle` 的 source kind 仍为 `StrategyArtifactSourceKind::FrontendGraph`，metadata、graph_id、compile_id、name、mode 和 source_ref 语义不变。
- `FrontendBacktestReplaySource::HistoricalReplay` 与 `FrontendBacktestReplaySource::DeterministicMock` 分支语义不变。
- HistoricalReplay 本地市场数据缺失提示不被吞掉，仍建议 `backtest_options.replay_source = "deterministic_mock"`。
- `FastBacktestSandbox` constructor、latency override、`sandbox.start`、`sandbox.run_backtest` 和 blocking task cancellation 映射语义不变。
- `BacktestRecord`、`build_backtest_artifact_views`、transient spill、`state.backtests` 和 audit log 不进入子模块。
- 不新增 public API，不新增 sibling 横向连接，不新增发布版本过渡、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 明确排除

- 不迁移 `execute_backtest_request` 整体。
- 不迁移 `start_backtest_run` 或 `execute_v4_backtest_request`。
- 不迁移 capability guard、runtime config capability guard、`graph_json` 必填校验或 v4 fallback bridge。
- 不迁移 graph targets、runtime targets、backtest id、governance snapshot、capability snapshot event、runtime event envelopes、account summary、backtest spec、record construction、artifact views、transient spill、`state.backtests` 或 audit log。
- 不迁移 record store、replay、experiment、compare、artifact schema、response mapping owner、schema owner、state owner、persistence owner 或 frontend caller。
- 不改变 `BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
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

下一批进入 `BE-001R-04 runtime.backtest.execution_start.legacy_dispatch 单叶 closeout`。closeout 必须判断:

1. `prepare_legacy_backtest_dispatch` / `run_legacy_backtest_dispatch` 两段式 helper 的父级私有子模块等价是否成立。
2. `legacy_dispatch` 是否值得继续细拆为 compile preparation、artifact bundle 或 sandbox replay 微叶。
3. 若继续细拆，是否需要先另起更低层等价基线，而不是直接拆动 record write、artifact schema、state、persistence 或 frontend。
4. 若不继续细拆，是否回到 `runtime.backtest.execution_start` 父叶候选队列。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.legacy_dispatch` 已抽离时，必须说明只迁移了 legacy compile/assumption/artifact/sandbox replay 到 `src/runtime/backtest/legacy_dispatch.rs`，且父级 `src/runtime/backtest/execution_start.rs` 仍保留 validation、v4 bridge、actor/collaboration、id、governance、event envelope、record assembly、artifact views、transient spill、state write 和 audit log。不得宣称 record write/persistence/state/frontend owner 已迁移、发布过渡已启动、`runtime.backtest.execution_start` 已整体停止细分，或 `legacy_dispatch` 已完成 closeout。

---

## 验收标准

1. `96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/legacy_dispatch.rs` 存在并承载 `prepare_legacy_backtest_dispatch`、`run_legacy_backtest_dispatch`、`LegacyBacktestDispatchPlan` 和 `LegacyBacktestDispatchOutput`。
3. `src/runtime/backtest/execution_start.rs` 只通过父级私有导入调用 legacy dispatch helper。
4. record assembly、artifact views、transient spill、state write、audit log、schema owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生发布版本过渡连接。
