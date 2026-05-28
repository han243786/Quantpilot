# v4.16.0 runtime.backtest.execution_start.v4_projection 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001O-02。  
> 基准: `82-runtime.backtest.execution_start.v4_projection单子叶等价基线.md`、`81-runtime.backtest.execution_start单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.backtest.execution_start.v4_projection` 抽离方案，`no code movement`；下一批若实施，只允许迁移 v4 projection helper 与现有两个单元测试，不得混入 request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001O 从 v4 projection 基线进入抽离方案 | 推进 |
| 规范矩阵 | projection helper 最小移动、父级私有调用、schema/owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` | 抽离方案 |
| 模块树 | `runtime.backtest.execution_start.v4_projection` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.execution_start.v4_projection` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.execution_start.v4_projection` |
| 父模块 | `runtime.backtest.execution_start` |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs` |
| 下一批计划目标 | future `src/runtime/backtest/v4_projection.rs` |
| 父级导入策略 | 在 `execution_start.rs` 中用 path module 接入 `v4_projection`，只由父模块调用 |
| 对外 API 策略 | 不新增 public API；父模块可见 helper 使用 `pub(super)`，其余 helper 保持子模块私有 |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树 |

---

## 适配性校验

当前 `src/runtime/backtest/execution_start.rs` 中，v4 projection helper 已形成清晰连续区域:

| helper | 当前调用关系 | 方案可见性 |
| --- | --- | --- |
| `build_v4_backtest_output` | 被 `execute_v4_backtest_request` 调用 | `pub(super)` |
| `v4_equity_curve_from_artifact` | 被 `execute_v4_backtest_request` 调用 | `pub(super)` |
| `frontend_events_from_v4_backtest_artifact` | 被 `execute_v4_backtest_request` 调用 | `pub(super)` |
| `v4_win_rate_from_equity_curve` | 只被 `build_v4_backtest_output` 和单元测试调用 | 子模块私有 |
| `v4_portfolio_from_artifact` | 只被 `build_v4_backtest_output` 调用 | 子模块私有 |
| `v4_frontend_event` | 只被 `frontend_events_from_v4_backtest_artifact` 调用 | 子模块私有 |

该批 helper 只依赖 `V4BacktestArtifact`、`BacktestOutput`、`BacktestEquityPoint`、`PortfolioState`、`FrontendRuntimeEvent`、`RuntimeEventEnvelope`、`Value` 和 `json!`。实际迁移时优先保留 `use super::*;`，不新建跨 sibling 依赖，不改 schema owner。

---

## 抽离目标

下一批实际抽离只允许做以下结构性移动:

1. 新建 `src/runtime/backtest/v4_projection.rs`。
2. 在 `src/runtime/backtest/execution_start.rs` 内用 path module 接入 `v4_projection`。
3. 从 `execution_start.rs` 移入 v4 projection helper 群。
4. 只把父级真实调用的 helper 暴露为 `pub(super)`。
5. 把现有两个 v4 projection 单元测试迁入新子模块。
6. 保持 `execute_v4_backtest_request` 的调用顺序、错误处理、governance envelope 校验、record write 和 artifact view 构建不变。

建议形态:

```rust
#[path = "v4_projection.rs"]
mod v4_projection;

use v4_projection::{
    build_v4_backtest_output,
    frontend_events_from_v4_backtest_artifact,
    v4_equity_curve_from_artifact,
};
```

`v4_projection` 子模块内部保持:

```rust
use super::*;
```

---

## 允许迁移清单

| 函数/测试 | 迁移原因 | 可见性策略 |
| --- | --- | --- |
| `build_v4_backtest_output` | v4 artifact 到 `BacktestOutput` 的纯 projection | `pub(super)` |
| `v4_equity_curve_from_artifact` | v4 artifact 到 equity curve 的纯 projection | `pub(super)` |
| `frontend_events_from_v4_backtest_artifact` | v4 artifact 到 frontend runtime events 的纯 projection | `pub(super)` |
| `v4_win_rate_from_equity_curve` | output summary helper | 子模块私有 |
| `v4_portfolio_from_artifact` | final portfolio helper | 子模块私有 |
| `v4_frontend_event` | frontend event construction helper | 子模块私有 |
| `v4_win_rate_counts_up_steps_over_directional_steps` | win rate 等价证据 | 子模块 test |
| `v4_equity_curve_empty_artifact_does_not_fabricate_zero_point` | 空 artifact 等价证据 | 子模块 test |

---

## 必须保持原位

| owner | 保留内容 | 原因 |
| --- | --- | --- |
| `src/runtime/backtest/execution_start.rs` | `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request` | 创建路径和 record write owner 不迁移 |
| `src/runtime/backtest/execution_start.rs` | `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type` | v4 request resolution 另起候选，不混入 projection |
| `src/backtest_artifacts.rs` | `build_backtest_artifact_views`、artifact views、manifest digest、transient spill helper | artifact schema owner 不迁移 |
| `src/runtime_response_mapping.rs` | `backtest_run_response` 与 response mapping | response schema owner 不迁移 |
| `src/runtime_persistence.rs` | saved/transient record IO | persistence owner 不迁移 |
| `src/frontend_api_types.rs` | API 类型 | schema owner 不迁移 |
| `src/runtime/backtest.rs` | record store、replay、experiment sibling | 后续另起基线 |
| `AppState` | `state.backtests`、store dirs、transient dirs、locks | state owner 与锁顺序不迁移 |

---

## 明确排除

- 不迁移 `execute_v4_backtest_request`。
- 不迁移 `is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`。
- 不迁移 `execute_backtest_request` 或 `start_backtest_run`。
- 不迁移 `build_backtest_artifact_views`、`maybe_spill_transient_backtest_record` 或 `backtest_run_response`。
- 不迁移 record store、replay、experiment、compare、artifact schema、persistence owner、response mapping owner、schema owner、state owner 或 frontend caller。
- 不改变 `V4BacktestArtifact`、`BacktestOutput`、`BacktestRunResponse`、`BacktestRecord`、`FrontendRuntimeEvent` 或 `RuntimeEventEnvelope` schema。
- 不新增发布版本过渡、横向直连、缓存旁路或性能优化提案。ASCII guard: `release transition guard`。

---

## 中止条件

下一批实际抽离只要出现以下任一情况，必须中止并回到方案讨论:

1. 需要改变 v4 request resolution、graph/symbol/event type 解析或错误码。
2. 需要改变 artifact schema、response schema、event envelope、state lock、persistence IO 或 frontend caller。
3. 需要把 projection helper 变为 `pub(crate)` 或更宽的 public API。
4. 需要让 record store、replay、experiment、compare 或 frontend caller 直接调用 projection 子模块。
5. 需要移动 `execute_v4_backtest_request` 或改变 record write / transient spill 顺序。
6. `cargo check -p quantpilot` 暴露的可见性问题不能通过父级私有导入解决。
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

下一批进入 `BE-001O-03 runtime.backtest.execution_start.v4_projection 抽离记录`。实施范围只能是:

1. 新建 v4 projection 子模块。
2. 移入允许迁移清单中的 helper 与测试。
3. 在父级 `execution_start.rs` 私有导入三个入口 helper。
4. 保持 request resolution、record write、artifact、response、persistence、schema、state、frontend 和发布过渡边界不变。

---

## 幻觉检查点

AI 声称 `runtime.backtest.execution_start.v4_projection` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact` 或任何 helper 已迁移；不得宣称 request resolution、record write、artifact schema、response schema、state owner、persistence owner、frontend caller、整理、重构或发布过渡已经完成。

---

## 验收标准

1. `83-runtime.backtest.execution_start.v4_projection抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确下一批只允许迁移六个 projection helper 和两个单元测试。
3. 方案明确父级只私有导入 `build_v4_backtest_output`、`v4_equity_curve_from_artifact`、`frontend_events_from_v4_backtest_artifact`。
4. 方案明确 request resolution、record write、artifact schema、response schema、state owner、persistence owner 和 frontend caller 不迁移。
5. 本批不发生代码移动。
