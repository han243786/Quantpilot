# v4.16.0 runtime.backtest.execution_start.legacy_dispatch 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001R-01。  
> 父叶: `runtime.backtest.execution_start`。  
> 基准: `81-runtime.backtest.execution_start单叶closeout.md`、`93-runtime.backtest.execution_start.v4_runtime_execution单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 legacy non-v4 path 的等价基线，`no code movement`。下一步若继续，必须先形成 BE-001R-02 抽离方案，不能直接移动代码。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001R 从父叶候选队列进入单子叶等价基线 | 基线 |
| 规范矩阵 | 父级私有 helper、legacy compile/sandbox replay、blocking/error mapping、禁止横向连接 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` | 新候选 |
| 模块树 | `runtime.backtest.execution_start.legacy_dispatch` 白箱节点 | 登记 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.legacy_dispatch` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.legacy_dispatch` |
| 父模块 | `runtime.backtest.execution_start` |
| 真实文件 | `src/runtime/backtest/execution_start.rs` |
| sibling 文件 | `src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_runtime_execution.rs` |
| 保留 owner | `src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`AppState` |
| 关键 public 方法 | 无新增 public API；候选子叶只能由父级通过 `pub(super)` helper 调用 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树、UTF-8、diff check |

---

## 冻结范围

本基线只冻结 `execute_backtest_request` 中非 v4 分支的 legacy dispatch 行为:

1. capability guard 与 runtime config capability guard。
2. `graph_json` 必填校验与 v4 fallback bridge。
3. `compile_runtime_protocol_via_qs`。
4. `apply_backtest_execution_assumption_overrides`。
5. `compile_runtime_protocol_config`。
6. `resolved_backtest_execution_assumptions` 与 `resolved_execution_assumption_sources`。
7. `build_compile_artifact_bundle`。
8. `FrontendBacktestReplaySource::HistoricalReplay` / `FrontendBacktestReplaySource::DeterministicMock` 分支。
9. `FastBacktestSandbox::with_replay_from_core_ir`。
10. `FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode`。
11. `DeterministicTestMode::replay_defaults(now_ms, BACKTEST_DETERMINISTIC_SEED)`。
12. `tokio::task::spawn_blocking` legacy sandbox boundary。
13. latency override 到 `qrpc_runtime::slippage::ExecutionAssumptions`。
14. `sandbox.start()` 与 `sandbox.run_backtest()`。

本基线不移动代码，不新增 helper，不改变任何 route、schema、state lock、persistence 或 frontend caller。

---

## 输入输出基线

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `state` | 父级 `execute_backtest_request` | `AppState` | 本批不迁移 state owner，只读取 `graph_store_dir`、`transient_backtest_store_dir` 等父级上下文 |
| `user_id` | auth middleware / experiment bridge | `auth::UserId` | 不改变 scoped key 或 audit identity |
| `request` | `FrontendRunRequest` | request body | 不改变 `runtime_config`、`backtest_options`、`actor` 或 metadata 语义 |
| `graph_json` | `request.graph_json` | JSON graph | legacy path 继续走 QS compile，v4 path 已由父级 bridge 分流 |
| `id_suffix` | experiment sweep / direct run | optional suffix | 本候选不拥有 backtest id 生成 |
| `now_ms` | 父级 timestamp | u64 | legacy compile artifact、replay defaults 和 backtest id 继续共享同一时间锚点 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `compiled` | record/spec/artifact builder | compiled runtime protocol | 不改变 protocol name、config hash、compiled config 或 core IR |
| `artifacts` | `BacktestRecord.artifacts` | compile artifact bundle | 不改变 artifact source kind 或 graph/compile metadata |
| `backtest` | record/projection | sandbox backtest output | 不改变 portfolio、summary、events source 或 trade count |
| resolved assumptions | `build_backtest_spec` | execution assumption snapshot | 不改变 latency override 或 source attribution |

---

## 候选拆分价值

| 维度 | 判定 | 说明 |
| --- | --- | --- |
| 独立 owner | 值得评估 | legacy non-v4 path 的 QS compile、sandbox replay 和 assumption override 与 v4 deterministic runtime execution 已形成不同 owner |
| 独立 public 入口 | 不新增 | 只能是父级私有 helper，不可对外 public |
| 独立状态/锁 | 谨慎 | 本候选不能迁移 `AppState`、`state.backtests`、transient spill 或 persistence owner |
| 独立验证证据 | 可建立 | `api_backtest`、`api_evidence_contract`、`api_run` 可覆盖 legacy backtest 创建与 response/event envelope |
| 横向连接风险 | 中等 | compile artifact、record write 和 artifact views 易被误迁移，必须在方案阶段重新列排除项 |

初步结论: `legacy_dispatch` 值得进入抽离方案评估，但不允许在 BE-001R-01 直接实施。若 BE-001R-02 发现返回值需要携带过多 record/persistence 上下文，应暂停并重新拆分为更窄的 legacy compile/replay helper。

---

## 父子通信规则

```text
runtime.backtest.execution_start
  -> execute_backtest_request
      -> v4 bridge / v4 child modules
      -> legacy_dispatch candidate
          -> QS compile
          -> execution assumption override
          -> compile artifact bundle
          -> blocking FastBacktestSandbox replay
      -> parent record/event/persistence assembly
```

`runtime.backtest.execution_start.legacy_dispatch` 只能由父级 `runtime.backtest.execution_start` 调用。不得让 v4 request resolution、v4 projection、v4 runtime execution、record store、replay、experiment、compare、persistence、frontend caller 或其他 sibling 横向直连；发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `execute_backtest_request` 或 `start_backtest_run`。
- 不移动 legacy compile/sandbox 代码。
- 不改变 `is_v4_backtest_request` fallback bridge。
- 不迁移 `execute_v4_backtest_request`、`v4_request_resolution`、`v4_projection` 或 `v4_runtime_execution`。
- 不迁移 graph targets、runtime targets、event envelope、capability snapshot event、account summary、backtest spec、artifact views、record write、transient spill、`state.backtests` 或 audit log。
- 不迁移 record store、replay、experiment、compare、artifact schema、response mapping owner、schema owner、state owner、persistence owner 或 frontend caller。
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

AI 声称 BE-001R-01 完成时，必须说明: 这里只完成 `runtime.backtest.execution_start.legacy_dispatch` 的等价基线，且 `no code movement`。不得宣称 legacy helper 已抽离、`execute_backtest_request` 已整理、record write/persistence/state/frontend owner 已迁移、发布过渡已启动，或 `runtime.backtest.execution_start` 已整体停止细分。

---

## 验收标准

1. `94-runtime.backtest.execution_start.legacy_dispatch单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.backtest.execution_start.legacy_dispatch` 白箱节点。
3. 本基线明确 `no code movement`，并冻结 legacy compile/sandbox replay 的输入输出与排除项。
4. 下一步只能进入 BE-001R-02 抽离方案，不能直接移动代码。
5. 治理门禁能发现本基线文档、引导坐标、保留 owner、禁止迁移边界和回归证据缺失。
