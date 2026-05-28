# v4.16.0 runtime.backtest.execution_start 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001N-02。  
> 基准: `78-runtime.backtest.execution_start单子叶等价基线.md`、`77-runtime.backtest单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start` 抽离方案，`no code movement`；下一批若实施，只允许迁移 backtest 创建路径 handler/helper，不得混入 record store、replay、experiment、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001N `runtime.backtest.execution_start` 从等价基线进入抽离方案 | 推进 |
| 规范矩阵 | execution_start 最小移动、父级兼容 re-export、experiment 复用桥、共享 owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start` | 抽离方案 |
| 模块树 | `runtime.backtest.execution_start` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start` |
| 父模块 | `runtime.backtest` |
| 当前真实文件 | `src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/backtest.rs` |
| 下一批计划目标 | future `execution_start` handler 子模块，优先采用 `src/runtime/backtest/execution_start.rs` |
| 关键 public 方法 | `start_backtest_run` 作为 route handler public 兼容出口；`execute_backtest_request` 作为父级内部复用桥，不对外扩展 public API |
| 兼容出口 | `src/runtime/mod.rs` re-export `start_backtest_run`，并保留父级内部 `execute_backtest_request` 复用桥 |
| 测试/门禁 | `api_backtest`、`api_evidence_contract`、`api_run`、`cargo check -p quantpilot`、三矩阵门禁、全量树 |

---

## 抽离目标

下一批实际抽离只允许做以下结构性移动:

1. 新建 execution_start handler 子模块。
2. 从 `src/runtime/backtest.rs` 移入 backtest 创建路径函数和直接 helper。
3. 在 `src/runtime/mod.rs` 注册该子模块。
4. 通过父级兼容出口继续暴露 `start_backtest_run` 给 `src/backend/runtime/routes/backtest.rs`。
5. 通过父级内部兼容桥继续让 `start_backtest_experiment` 复用 `execute_backtest_request`，不迁移 experiment routes。

建议形态:

```rust
#[path = "backtest/execution_start.rs"]
mod backtest_execution_start;

pub(crate) use backtest_execution_start::start_backtest_run;
use backtest_execution_start::execute_backtest_request;
```

`execution_start` 子模块内部保持:

```rust
use super::*;
```

---

## 允许迁移清单

| 函数 | 迁移原因 | 可见性策略 |
| --- | --- | --- |
| `start_backtest_run` | route handler，属于创建路径入口 | `pub(crate)`，由父级 re-export 给 route facade |
| `execute_backtest_request` | legacy/v4 分流与 legacy backtest 创建主入口，同时被 experiment sweep 复用 | `pub(super)` 或父级内部可见，父级保留内部兼容桥 |
| `execute_v4_backtest_request` | v4 deterministic MachineGraph replay 创建入口 | 子模块内部私有 |
| `is_v4_backtest_request` | legacy/v4 path 判断 | 子模块内部私有 |
| `resolve_v4_backtest_graph` | v4 machine graph 解析 | 子模块内部私有 |
| `resolve_v4_backtest_symbols` | v4 symbols 解析 | 子模块内部私有 |
| `resolve_v4_backtest_market_event_type` | v4 replay event type 解析 | 子模块内部私有 |
| `build_v4_backtest_output` | v4 artifact 到 backtest output | 子模块内部私有 |
| `v4_win_rate_from_equity_curve` | v4 metrics helper | 子模块内部私有 |
| `v4_equity_curve_from_artifact` | v4 equity curve helper | 子模块内部私有 |
| `v4_portfolio_from_artifact` | v4 portfolio helper | 子模块内部私有 |
| `frontend_events_from_v4_backtest_artifact` | v4 artifact 到 frontend runtime events | 子模块内部私有 |
| `v4_frontend_event` | v4 frontend event helper | 子模块内部私有 |

---

## 必须保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest.rs` | record store、replay、experiment sweep 相关 handler/helper | 后续另起 sibling 基线 |
| `src/backend/runtime/routes/backtest.rs` | route registration | route facade 已 closeout，不再改 path/method |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、`maybe_spill_transient_backtest_record`、artifact schema、manifest digest | artifact owner 不私有化 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response`、detail/replay/list response mapping | response schema owner 不迁移 |
| `src/frontend_api_types.rs` | API 类型 | schema owner 不迁移 |
| `src/backtest_compare.rs` | `compare_backtests` 与 compare core/narrative | compare owner 不迁移 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 明确排除

- 不迁移 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`。
- 不迁移 `get_backtest_replay`。
- 不迁移 `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`。
- 不迁移 `compare_backtests`。
- 不改变 `/api/runtime/backtest` route path、method、handler 调用或 response schema。
- 不改变 event envelope、governance snapshot、artifact manifest、transient spill threshold、state scoped key 或 AppState lock order。
- 不主动提出发布版本过渡，不新增子模块横向连接。ASCII guard: `release transition guard`。

---

## 中止条件

下一批实际抽离只要出现以下任一情况，必须中止并回到方案讨论:

1. 需要修改 route facade、route path、route method 或 response schema。
2. 需要迁移 artifact schema、manifest digest、persistence IO、response mapping、frontend API types 或 AppState owner。
3. 需要把 record store、replay、experiment 或 compare 混入 execution_start。
4. 需要改变 `execute_backtest_request` 被 experiment sweep 复用的能力。
5. 需要扩大 v4 provider 支持、改变 v4 graph/symbol/event resolution 或发布版本横向连接。
6. `cargo check -p quantpilot` 暴露的可见性问题不能通过父级兼容 re-export / internal bridge 解决。
7. `api_backtest`、`api_evidence_contract` 或 `api_run` 出现行为回归。

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

下一批进入 `BE-001N-03 runtime.backtest.execution_start 抽离记录`。实施范围只能是:

1. 新建 execution_start handler 子模块。
2. 移入允许迁移清单中的函数。
3. 在 `src/runtime/mod.rs` 注册子模块并保留 `start_backtest_run` / `execute_backtest_request` 兼容出口。
4. 保持 route facade、record store、replay、experiment、artifact、compare、persistence、schema、state 和 frontend owner 不变。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 `start_backtest_run`、`execute_backtest_request`、v4 helper、record store、replay、experiment、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller 或发布过渡已经迁移。

---

## 验收标准

1. `79-runtime.backtest.execution_start抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确下一批计划目标、父级兼容出口、允许迁移清单和必须原位保留的 owner。
3. 方案明确 `execute_backtest_request` 被 experiment sweep 复用，实际抽离不得破坏该调用。
4. 方案明确下一批仍不得迁移 record store、replay、experiment、artifact、compare、persistence、schema、state、frontend 或发布过渡。
5. 本批不发生代码移动。
