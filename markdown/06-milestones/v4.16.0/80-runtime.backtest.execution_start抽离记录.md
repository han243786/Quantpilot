# v4.16.0 runtime.backtest.execution_start 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001N-03。  
> 基准: `79-runtime.backtest.execution_start抽离方案.md`、`78-runtime.backtest.execution_start单子叶等价基线.md`。  
> 判定: 本批完成 `runtime.backtest.execution_start` 第一轮物理抽离；只移动 backtest 创建路径 handler/helper，并保留父级兼容出口与 experiment 复用桥。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001N-03 `runtime.backtest.execution_start` 从抽离方案进入抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、`pub(super)` 内部桥、共享 owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start` | 实际抽离 |
| 模块树 | `runtime.backtest.execution_start` 白箱节点 | 补真实文件 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start` |
| 父模块 | `runtime.backtest` |
| 新真实文件 | `src/runtime/backtest/execution_start.rs` |
| 父级兼容文件 | `src/runtime/mod.rs` |
| 保留文件 | `src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs`、`src/backtest_artifacts.rs`、`src/runtime_response_mapping.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs` |
| 关键 public 方法 | `start_backtest_run` 继续由父级 `runtime` re-export 给 route facade |
| 内部复用桥 | `execute_backtest_request` 改为 `pub(super)`，由父级 `runtime` 导入供 `start_backtest_experiment` 继续复用 |
| 测试/门禁 | `cargo check -p quantpilot`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 实际变更

1. 新建 `src/runtime/backtest/execution_start.rs`，并以 `use super::*;` 接入父级 runtime 白箱上下文。
2. 从 `src/runtime/backtest.rs` 迁出 backtest 创建路径 handler/helper。
3. 在 `src/runtime/mod.rs` 增加:

```rust
#[path = "backtest/execution_start.rs"]
mod backtest_execution_start;

use backtest_execution_start::execute_backtest_request;
pub(crate) use backtest_execution_start::start_backtest_run;
```

4. `start_backtest_run` 继续保持 `pub(crate)`，route facade 调用语义不变。
5. `execute_backtest_request` 改为 `pub(super)`，只开放给父级 `runtime` 内部桥接，不形成对外 public API。
6. `src/runtime/backtest.rs` 当前从 experiment helper 开始，record store、replay、experiment、detail/save/discard 仍保留原 owner。

---

## 已迁移清单

| 函数 | 新位置 | 可见性 |
| --- | --- | --- |
| `start_backtest_run` | `src/runtime/backtest/execution_start.rs` | `pub(crate)`，由父级 re-export |
| `execute_backtest_request` | `src/runtime/backtest/execution_start.rs` | `pub(super)`，父级内部复用桥 |
| `execute_v4_backtest_request` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `is_v4_backtest_request` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `resolve_v4_backtest_graph` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `resolve_v4_backtest_symbols` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `resolve_v4_backtest_market_event_type` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `build_v4_backtest_output` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `v4_win_rate_from_equity_curve` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `v4_equity_curve_from_artifact` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `v4_portfolio_from_artifact` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `frontend_events_from_v4_backtest_artifact` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |
| `v4_frontend_event` | `src/runtime/backtest/execution_start.rs` | 子模块私有 |

---

## 保持原位

- `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 仍在 `src/runtime/backtest.rs`。
- `get_backtest_replay` 仍在 `src/runtime/backtest.rs`。
- `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 仍在 `src/runtime/backtest.rs`。
- `compare_backtests` 仍在 `src/backtest_compare.rs`，route facade 不变。
- `build_backtest_artifact_views`、`maybe_spill_transient_backtest_record` 仍在 `src/backtest_artifacts.rs`。
- `backtest_run_response` 与 detail/replay/list response mapping 仍在 `src/runtime_response_mapping.rs`。
- `runtime_persistence`、`frontend_api_types`、`AppState`、state locks、spill threshold、scoped key 语义均不迁移。
- 不主动提出发布版本过渡，不新增横向连接。ASCII guard: `release transition guard`。

---

## 等价保护

| 保护点 | 结果 |
| --- | --- |
| route path/method | `/api/runtime/backtest` 未改变 |
| route facade | `src/backend/runtime/routes/backtest.rs` 未改变 |
| response schema | `BacktestRunResponse` mapping 未改变 |
| experiment 复用 | `start_backtest_experiment` 继续经父级导入调用 `execute_backtest_request` |
| shared owner | artifact、persistence、schema、state、frontend owner 未私有化 |
| 发布过渡 | 未启动 |

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

## 下一步

下一批进入 `BE-001N-04 runtime.backtest.execution_start 单叶 closeout`。closeout 必须判断:

1. `runtime.backtest.execution_start` 是否已经具备独立白箱边界。
2. 本叶是否继续细拆 v4 helper、artifact projection 或 event projection。
3. 是否应回到 `runtime.backtest` sibling 队列，继续 `runtime.backtest.record_store` 或 `runtime.backtest.replay_status`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start` 已抽离时，必须说明只移动了创建路径 handler/helper 到 `src/runtime/backtest/execution_start.rs`，并且 `execute_backtest_request` 只是父级内部 `pub(super)` 复用桥。不得宣称 record store、replay、experiment、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller 或发布过渡已经迁移。

---

## 验收标准

1. `80-runtime.backtest.execution_start抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/execution_start.rs` 进入模块树与全量树真实文件。
3. `src/runtime/mod.rs` 保留 `start_backtest_run` re-export 和 `execute_backtest_request` 内部桥。
4. `src/runtime/backtest.rs` 保留 record store、replay、experiment、compare 调用边界和共享 owner。
5. 本批不改变 route、response schema、artifact schema、persistence、state、frontend 或发布过渡。
