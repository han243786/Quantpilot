# v4.16.0 runtime.backtest.execution_start.legacy_dispatch 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001R-04。  
> 基准: `96-runtime.backtest.execution_start.legacy_dispatch抽离记录.md`、`95-runtime.backtest.execution_start.legacy_dispatch抽离方案.md`、`94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md`、`93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `runtime.backtest.execution_start.legacy_dispatch` 已完成等价 closeout，并设置 `stop_split: true`。本叶不继续细拆；后续若继续，应回到父叶 `runtime.backtest.execution_start` 或上层 `runtime.backtest` 另起候选基线，不能从 legacy dispatch 子叶继续外扩。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001R 从抽离记录进入单叶 closeout | 收口 |
| 规范矩阵 | 父级私有 helper、两段式等价、`stop_split: true`、禁止横向连接 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` | 单叶 closeout |
| 模块树 | `runtime.backtest.execution_start.legacy_dispatch` 白箱节点 | 设置停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.legacy_dispatch` |
| 父模块 | `runtime.backtest.execution_start` |
| 真实文件 | `src/runtime/backtest/legacy_dispatch.rs` |
| 父级文件 | `src/runtime/backtest/execution_start.rs` |
| 保留 sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_runtime_execution.rs` |
| 保留 shared owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| closeout 判定 | `stop_split: true` |
| public API | 无新增 public API；现有入口均为父模块可见 `pub(super)` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 等价确认

| 检查项 | 结论 | 证据 |
| --- | --- | --- |
| 父级私有调用 | 通过 | `src/runtime/backtest/execution_start.rs` 只通过 path module 导入 `legacy_dispatch` |
| compile preparation 顺序 | 通过 | `prepare_legacy_backtest_dispatch` 保留 QS compile、assumption override、compiled config 与 resolved assumptions |
| actor/collaboration 顺序 | 通过 | 父级仍在 dispatch plan 后、sandbox replay 前执行 `collaboration_with_run_actor` |
| sandbox replay 语义 | 通过 | `run_legacy_backtest_dispatch` 保留 HistoricalReplay / DeterministicMock 分支、latency override、`spawn_blocking`、`sandbox.start` 和 `sandbox.run_backtest` |
| record assembly 保留 | 通过 | `BacktestRecord` 构造、event envelope、artifact views、spill、`state.backtests` 和 audit log 仍在父级 |
| schema/owner 边界 | 通过 | artifact schema、response mapping、persistence、frontend schema 和 state owner 均未迁移 |
| 发布过渡保护 | 通过 | 未新增横向连接、缓存旁路、发布版本过渡或性能优化提案 |

**精确保留的关键调用**: `compile_runtime_protocol_via_qs`、`apply_backtest_execution_assumption_overrides`、`compile_runtime_protocol_config`、`resolved_backtest_execution_assumptions`、`resolved_execution_assumption_sources`、`build_compile_artifact_bundle`、`FrontendBacktestReplaySource`、`FastBacktestSandbox`、`tokio::task::spawn_blocking` 均只在 legacy dispatch 子叶内维持原语义，不外扩到 record assembly。

---

## 细拆价值判断

| 候选微叶 | 是否继续 | 理由 |
| --- | --- | --- |
| compile preparation | 否 | 只包含 QS compile、assumption override、compiled config 和 resolved assumptions，当前与 dispatch plan 一起更清晰 |
| artifact bundle | 否 | 只是 legacy dispatch 输出的一部分；拆出会扩大 helper 输出/导入面，不能减少 owner 耦合 |
| sandbox replay | 否 | 与 replay source、core IR、latency override、blocking cancellation 映射强相关；单独拆分会制造过窄微叶 |
| dispatch output types | 否 | `LegacyBacktestDispatchPlan` 与 `LegacyBacktestDispatchOutput` 只服务父级，单独建类型文件没有收益 |

**最终判定**: `stop_split: true`。

`legacy_dispatch` 当前没有 state、IO 持久化、锁、route、schema owner、frontend caller 或外部 API。继续细拆只能降低局部可读性并增加父级导入面，不会降低真实耦合。因此本叶停止内部递归。

---

## 保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | validation、v4 bridge、actor/collaboration、id、governance、event envelope、record assembly、artifact views、transient spill、state write、audit log | 创建路径父模块与 record assembly owner |
| `src/runtime/backtest/v4_request_resolution.rs` | v4 detection、graph/symbol/event resolution | sibling 已 closeout，不回流 |
| `src/runtime/backtest/v4_projection.rs` | v4 artifact projection 与 frontend events | sibling 已 closeout，不回流 |
| `src/runtime/backtest/v4_runtime_execution.rs` | v4 deterministic runtime execution | sibling 已 closeout，不回流 |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` | response mapping owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API schema | schema owner 不迁移 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 父子通信规则

`runtime.backtest.execution_start.legacy_dispatch` 只能由父级 `runtime.backtest.execution_start` 调用。不得让 v4 request resolution、v4 projection、v4 runtime execution、record store、replay、experiment、compare、persistence、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

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

BE-001R closeout 后，递归队列回到父级:

1. 优先对 `runtime.backtest.execution_start` 做父叶残余判断，确认是否还有值得独立抽离的非 record-write 候选。
2. 若剩余职责主要是 record assembly、artifact views、spill、state write 和 audit log，则不得在本叶内继续硬拆，应回到 `runtime.backtest` 上层队列另起基线。
3. 若要动 record write、artifact schema、state owner、persistence owner 或 frontend caller，必须另起提案并回到适配性校验。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.legacy_dispatch` 已 closeout 时，必须说明: 本叶只完成 legacy compile/assumption/artifact/sandbox replay helper 的等价 closeout，并设置 `stop_split: true`。父级 `src/runtime/backtest/execution_start.rs` 仍保留 validation、v4 bridge、actor/collaboration、id、governance、event envelope、record assembly、artifact views、transient spill、state write 和 audit log；不得宣称 record write/persistence/state/frontend owner、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `97-runtime.backtest.execution_start.legacy_dispatch单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.backtest.execution_start.legacy_dispatch` 已 closeout，并设置 `stop_split: true`。
3. `src/runtime/backtest/legacy_dispatch.rs` 只保留父级私有 helper 与轻量结构，不新增 public API。
4. record assembly、artifact views、transient spill、state write、audit log、schema owner、persistence owner 和 frontend caller 不迁移。
5. 治理门禁能发现 closeout 文档、`stop_split: true`、禁止迁移边界、下一候选和回归证据缺失。
