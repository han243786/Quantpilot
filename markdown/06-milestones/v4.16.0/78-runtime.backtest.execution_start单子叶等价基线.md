# v4.16.0 runtime.backtest.execution_start 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001N-01。  
> 基准: `77-runtime.backtest单叶closeout.md`、`74-runtime.backtest单子叶等价基线.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start` 等价基线，`no code movement`；不迁移 handler、artifact schema、compare owner、record store、replay、experiment、persistence owner、state owner、schema owner、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001N `runtime.backtest.execution_start` 从 `runtime.backtest` closeout 进入 handler 域子叶基线 | 推进 |
| 规范矩阵 | backtest execution start 边界冻结、record/replay/experiment 排除、artifact/persistence/schema owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start` | 新增单子叶基线 |
| 模块树 | `runtime.backtest.execution_start` 白箱候选 | 新增子叶坐标 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start` |
| 父模块 | `runtime.backtest` |
| 父级 route facade | `src/backend/runtime/routes/backtest.rs` |
| 当前真实文件 | `src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| 当前 public route handler | `start_backtest_run` |
| 当前内部执行入口 | `execute_backtest_request`、`execute_v4_backtest_request` |
| 当前 helper 群 | `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`、`build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`v4_portfolio_from_artifact`、`frontend_events_from_v4_backtest_artifact`、`v4_frontend_event` |
| 当前共享 owner | `build_backtest_artifact_views`、`maybe_spill_transient_backtest_record`、`runtime_governance_snapshot`、`validate_runtime_event_envelopes`、`backtest_run_response` |
| 测试/门禁 | `cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`cargo check -p quantpilot` |

---

## 当前真实边界

`runtime.backtest.execution_start` 只覆盖 backtest 创建路径:

1. `POST /api/runtime/backtest` 经 `src/backend/runtime/routes/backtest.rs` 调用 `runtime_handlers::start_backtest_run`。
2. `start_backtest_run` 调用 `execute_backtest_request`，最终返回 `BacktestRunResponse`。
3. legacy path 由 `execute_backtest_request` 执行 QS compile、runtime protocol compile、deterministic/historical sandbox backtest、event envelope、artifact views 和 transient/in-memory record 写入。
4. v4 path 由 `execute_v4_backtest_request` 执行 v4 graph resolution、symbol expansion、deterministic bars/ticks replay、v4 artifact、event projection、artifact views 和 transient/in-memory record 写入。
5. 两条路径都必须生成 governance snapshot、validate runtime event envelopes、构建 `BacktestRecord`、调用 `build_backtest_artifact_views`，再通过 `maybe_spill_transient_backtest_record` 或 `state.backtests` 保存 transient record。

---

## 输入输出白箱

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| backtest HTTP request | frontend、tests、local API caller | `POST /api/runtime/backtest` JSON | route path/method 不变 |
| `FrontendRunRequest` | route JSON body | runtime config、graph_json、backtest_options、capability_context | 必须保留 capability guard、runtime config capability guard、execution assumption override 校验 |
| `graph_json` | `FrontendRunRequest` | graph JSON / v4 machine graph / formal QS artifact | legacy path 必须能 QS compile；v4 path 必须能 resolve machine graph |
| `AppState` | backend app state | stores、dirs、locks | 不迁移 AppState owner、store dir 或锁顺序 |
| `UserId` | auth middleware | scoped user | transient/in-memory record 必须按 user scoped key 存储 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `BacktestRunResponse` | frontend、tests | JSON response | 保留 backtest id、graph/compile/config、event count、account、artifact views |
| transient/in-memory `BacktestRecord` | `state.backtests` 或 transient spill file | governed record | 不改变 spill threshold、scoped key 或 persistence owner |
| artifact views | frontend artifact viewer、tests | event log、metrics、ledger、equity curve、manifest | 不改变 artifact schema、digest、governance rebuild 语义 |
| audit log | stderr safe log | sanitized audit line | 不引入 secret 或未清洗数据 |

---

## 关键方法冻结

| 方法 | 当前职责 | 本基线约束 |
| --- | --- | --- |
| `start_backtest_run` | route handler，调用执行入口并组装 `BacktestRunResponse` | 后续抽离时必须保留 route signature 与 response schema |
| `execute_backtest_request` | legacy deterministic/historical backtest 执行入口 | 不得改变 compile、sandbox、event envelope、artifact、spill 和 state 写入顺序 |
| `execute_v4_backtest_request` | v4 deterministic MachineGraph replay 执行入口 | 不得改变 graph/symbol/event resolution、bar/tick replay、artifact、event projection 和 state 写入顺序 |
| `is_v4_backtest_request` | 判断是否走 v4 path | 不得扩大 provider 支持或改变 v4 判定条件 |
| `resolve_v4_backtest_graph` | 解析 v4 machine graph | 不得绕过 static handoff / formal QS artifact rejection |
| `resolve_v4_backtest_symbols` | 解析回测 symbols | 不得改变 explicit symbols、metadata symbols 和 fallback 规则 |
| `resolve_v4_backtest_market_event_type` | 解析 v4 replay event type | 不得在缺少 replayable event 时静默 fallback |
| `build_v4_backtest_output` | v4 artifact 到 backtest output | 不得改变 metrics/account/equity curve 语义 |
| `frontend_events_from_v4_backtest_artifact` | v4 artifact 到 frontend events | 不得改变 event stage/severity/module_key 语义 |

---

## 明确排除

- 不迁移 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`，这些属于后续 `runtime.backtest.record_store` 候选。
- 不迁移 `get_backtest_replay`，它属于后续 `runtime.backtest.replay_status` 候选。
- 不迁移 `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`，这些属于后续 `runtime.backtest.experiment_sweep` 或 sibling 候选。
- 不迁移 `src/backtest_compare.rs`、`compare_backtests` 或 compare core/narrative owner。
- 不迁移 `src/backtest_artifacts.rs`、artifact schema、manifest digest、governance rebuild 或 transient spill implementation owner。
- 不迁移 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`。
- 不改变 `src/backend/runtime/routes/backtest.rs` 的 route registration。
- 不主动提出发布版本过渡，不新增子模块横向连接。ASCII guard: `release transition guard`。

---

## 第一轮抽离候选方案

下一批如果进入实际抽离，建议只把 execution start 相关代码迁入新的 handler 子模块，例如 `src/runtime/backtest/execution_start.rs`，并由 `src/runtime/mod.rs` 或父级兼容出口继续 re-export `start_backtest_run`。

允许迁移候选:

- `start_backtest_run`
- `execute_backtest_request`
- `execute_v4_backtest_request`
- v4 execution start 直接 helper: `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`、`build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`v4_portfolio_from_artifact`、`frontend_events_from_v4_backtest_artifact`、`v4_frontend_event`

暂停项:

- 如果迁移需要私有化 artifact/persistence/response/schema owner，则中止。
- 如果迁移需要改变 route、response schema、event envelope、governance snapshot、spilled transient record 或 AppState lock 顺序，则中止。
- 如果 `api_backtest` 或 `api_evidence_contract` 出现行为回归，则先修复等价缺口，不继续 closeout。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
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

AI 声称 `runtime.backtest.execution_start` 已建立基线时，必须说明本批 `no code movement`。不得宣称 `start_backtest_run` 或 `execute_backtest_request` 已迁移，不得宣称 record store、replay、experiment、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller 或发布过渡已经迁移。

---

## 验收标准

1. `78-runtime.backtest.execution_start单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 execution_start 的真实输入、输出、关键方法、共享 owner 和排除边界。
3. 基线明确第一轮实际抽离只能移动 execution start handler/helper，不得混入 record store、replay、experiment、artifact、compare、persistence、schema、state 或 frontend owner。
4. 本批不发生代码移动。
