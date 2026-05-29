# v4.16.0 runtime.backtest.execution_start.legacy_dispatch 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001R-02。  
> 基准: `94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`、`93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立抽离方案，`no code movement`。下一批若实施，只允许迁移 legacy non-v4 compile/sandbox dispatch 最小 helper，不得迁移 record write、artifact views、state、persistence、schema、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001R 从等价基线进入抽离方案 | 推进 |
| 规范矩阵 | legacy dispatch 最小移动、父级私有 helper、record write 排除、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` | 抽离方案 |
| 模块树 | `runtime.backtest.execution_start.legacy_dispatch` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.legacy_dispatch` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_runtime_execution.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| 下一批计划目标 | future parent-private legacy dispatch module under `src/runtime/backtest` |
| 父级导入策略 | 在 `execution_start.rs` 中用 path module 接入 `legacy_dispatch`，只由父模块调用 |
| 对外 API 策略 | 不新增 public API；候选 helper 使用 `pub(super)`，不得扩大到 `pub(crate)` 或更宽 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树、UTF-8、diff check |

---

## 适配性校验

当前 `execute_backtest_request` 的 legacy non-v4 分支可以分成两段:

| 段落 | 当前职责 | 方案归属 |
| --- | --- | --- |
| pre-dispatch guard | capability guard、runtime config capability guard、execution assumption override validation、`graph_json` 必填校验、v4 fallback bridge | 继续留父级 |
| legacy dispatch | QS compile、execution assumption override、compiled config、resolved assumptions、compile artifact bundle、replay source、core IR、latency override、blocking sandbox replay | 下一批允许迁移 |
| parent record assembly | graph/runtime targets、backtest id、governance、events、capability snapshot、event envelopes、account summary、backtest spec、record、artifact views、spill、state write、audit log | 必须留父级 |

适配性结论: `legacy_dispatch` 值得抽离，但边界必须停在 sandbox replay 输出之后、record assembly 之前。它不能拥有 `BacktestRecord` 写入、`build_backtest_artifact_views`、transient spill、`state.backtests` 或 audit log。

---

## 抽离目标

下一批 BE-001R-03 只允许做以下结构移动:

1. 新建 future parent-private legacy dispatch module under `src/runtime/backtest`。
2. 在 `src/runtime/backtest/execution_start.rs` 中用 path module 接入 `legacy_dispatch`。
3. 从 `execute_backtest_request` 中迁移 QS compile 到 blocking sandbox replay 前后的连续 legacy dispatch 段。
4. 暴露一个父级私有 helper，建议名为 `run_legacy_backtest_dispatch`，可见性为 `pub(super)`。
5. helper 输入只允许是 `graph_json`、`request`、`now_ms` 和必要 metadata borrowed values。
6. helper 输出必须是轻量结构，建议名为 `LegacyBacktestDispatchOutput`，只携带 parent record assembly 需要的结果。
7. 父级继续负责 id 生成、governance、event envelope、account summary、backtest spec、record construction、artifact views、transient spill、state write 和 audit log。

建议 helper 形态:

```rust
pub(super) struct LegacyBacktestDispatchOutput {
    pub compiled: CompiledRuntimeProtocol,
    pub artifacts: StrategyArtifactBundle,
    pub replay_source: FrontendBacktestReplaySource,
    pub resolved_execution_assumptions: ResolvedBacktestExecutionAssumptions,
    pub resolved_execution_assumption_sources: ResolvedExecutionAssumptionSources,
    pub backtest: FastBacktestOutput,
}

pub(super) async fn run_legacy_backtest_dispatch(
    graph_json: &Value,
    request: &FrontendRunRequest,
    now_ms: u64,
) -> Result<LegacyBacktestDispatchOutput, (StatusCode, String)> {
    // Move only legacy compile/sandbox dispatch here.
}
```

实际类型名称以代码现有类型为准。若现有类型不可直接命名或会扩散 import 面，下一批应改用局部结构或暂停复核；不得为了强行抽离而把 schema owner 或 persistence owner 拖进子叶。

---

## 允许迁移清单

| 代码段 | 迁移原因 | 可见性策略 |
| --- | --- | --- |
| `compile_runtime_protocol_via_qs(graph_json)` | legacy path 的 QS compile 真源 | 子模块私有 |
| `apply_backtest_execution_assumption_overrides` | legacy execution assumptions 适配 | 子模块私有 |
| `compile_runtime_protocol_config` | compiled config 与 config hash 生成 | 子模块私有 |
| `resolved_backtest_execution_assumptions` | backtest spec 所需 resolved assumption | 输出结构字段 |
| `resolved_execution_assumption_sources` | backtest spec 所需 source attribution | 输出结构字段 |
| `build_compile_artifact_bundle` | legacy compile artifact bundle | 输出结构字段 |
| `request.backtest_replay_source` | sandbox replay 分支选择 | 输出结构字段 |
| `compiled.core_ir.clone` | sandbox input | 子模块私有 |
| `latency_override` | sandbox execution assumption override | 子模块私有 |
| `tokio::task::spawn_blocking` legacy closure | blocking sandbox replay 边界 | 子模块私有 |
| `FastBacktestSandbox::with_replay_from_core_ir` | HistoricalReplay sandbox | 子模块私有 |
| `FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode` | DeterministicMock sandbox | 子模块私有 |
| `DeterministicTestMode::replay_defaults` / `BACKTEST_DETERMINISTIC_SEED` | deterministic mock input | 子模块私有 |
| `sandbox.set_execution_assumptions` | latency override 应用 | 子模块私有 |
| `sandbox.start` / `sandbox.run_backtest` | legacy backtest execution | 子模块私有 |

---

## 必须保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `start_backtest_run`、`execute_backtest_request`、v4 fallback bridge、v4 parent orchestration、record assembly、state write | 创建路径父模块不迁移 |
| `src/runtime/backtest/v4_request_resolution.rs` | v4 detection、graph/symbol/event resolution | sibling 已 closeout，不回流 |
| `src/runtime/backtest/v4_projection.rs` | v4 artifact projection 与 frontend events | sibling 已 closeout，不回流 |
| `src/runtime/backtest/v4_runtime_execution.rs` | v4 deterministic runtime execution | sibling 已 closeout，不回流 |
| `src/backtest_artifacts.rs` | artifact views、manifest digest、spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` | response mapping owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API schema | schema owner 不迁移 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 错误与兼容桥保持

| 场景 | 当前错误/路径 | 约束 |
| --- | --- | --- |
| graph_json 缺失 | `json_bad_request("bad_request", ...)` | 保持父级，不进入本子叶 |
| v4 request | `is_v4_backtest_request` -> `execute_v4_backtest_request` | 保持父级 bridge，不进入 legacy helper |
| QS compile 失败 | `compile_runtime_protocol_via_qs` 原错误映射 | 不改变 error code 或 message |
| HistoricalReplay 缺数据 | 增补本地市场数据缺失提示与 deterministic_mock fallback 文案 | 不吞掉错误，不改 fallback 文案语义 |
| sandbox constructor 失败 | `internal_error(anyhow!(e))` | 不改变映射 |
| sandbox start/run 失败 | `internal_error(anyhow!(e))` | 不改变映射 |
| blocking task cancelled | `internal_error("回测任务被取消: {e}")` 当前语义 | 不改变 cancellation 映射 |

---

## 明确排除

- 不迁移 `execute_backtest_request` 整体。
- 不迁移 `start_backtest_run`。
- 不迁移 `execute_v4_backtest_request` 或任何 v4 child module。
- 不迁移 capability guard、runtime config capability guard、`graph_json` 必填校验或 v4 fallback bridge。
- 不迁移 graph targets、runtime targets、backtest id、governance snapshot、capability snapshot event、runtime event envelopes、account summary、backtest spec、record construction、artifact views、transient spill、`state.backtests` 或 audit log。
- 不迁移 record store、replay、experiment、compare、artifact schema、response mapping owner、schema owner、state owner、persistence owner 或 frontend caller。
- 不改 `BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不新增发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 中止条件

下一批实施时，遇到以下任一情况必须暂停并回到方案评估:

1. helper 输出必须携带完整 `BacktestRecord` 才能保持编译。
2. helper 需要写 `state.backtests` 或 transient spill。
3. helper 需要调用 `build_backtest_artifact_views`。
4. helper 需要改变 `BacktestSpec` 构建位置。
5. helper 需要迁移 event envelope 或 governance snapshot。
6. helper 需要扩大为 `pub(crate)` 或被 sibling 横向调用。
7. helper 需要更改 legacy HistoricalReplay 错误文案或 deterministic seed。

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

AI 声称 BE-001R-02 完成时，必须说明: 这里只完成 `runtime.backtest.execution_start.legacy_dispatch` 抽离方案，且 `no code movement`。不得宣称 legacy helper 已抽离、record write/persistence/state/frontend owner 已迁移、发布过渡已启动，或 `runtime.backtest.execution_start` 已整体停止细分。

---

## 验收标准

1. `95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 抽离方案明确下一批只允许迁移 legacy compile/sandbox dispatch 最小 helper。
3. 方案明确父级保留 record assembly、artifact views、spill、state write 和 audit log。
4. 方案明确 BE-001R-03 实施前的中止条件。
5. 治理门禁能发现本方案文档、引导坐标、允许迁移清单、禁止迁移边界、`no code movement` 和回归证据缺失。
