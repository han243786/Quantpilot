# v4.16.0 runtime.backtest.experiment_sweep.parameter_grid 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001W-03。  
> 基准: `112-runtime.backtest.experiment_sweep.parameter_grid抽离方案.md`、`111-runtime.backtest.experiment_sweep.parameter_grid单子叶等价基线.md`、`110-runtime.backtest.experiment_sweep单叶closeout.md`。  
> 判定: 按方案完成 `runtime.backtest.experiment_sweep.parameter_grid` 第一轮实际抽离；只迁移 3 个参数网格 helper，不迁移 route、handler orchestration、execution_start、persistence、response mapping、schema、state、audit、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001W parameter_grid 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 私有子模块、父级调用、`pub(super)` 可见性、禁止横向连接 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` | 物理抽离 |
| 模块树 | `runtime.backtest.experiment_sweep.parameter_grid` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.parameter_grid` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.parameter_grid` |
| 新真实文件 | `src/runtime/backtest/parameter_grid.rs` |
| 父级真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 保留真实文件 | `src/runtime/backtest.rs`、`src/frontend_api_types.rs`、`tests/api_experiments.rs`、`tests/api_backtest.rs`、`tests/api_evidence_contract.rs` |
| public 方法 | 本节点不新增 public 方法；只保留父级私有 `pub(super)` helper |
| 已迁移 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` |
| 保留父级方法 | `start_backtest_experiment`、`resolved_backtest_execution_assumptions`、`execute_backtest_request` |
| 保留常量 | `MAX_EXPERIMENT_VARIANTS` |
| 保留类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`RuntimeProtocolCoreConfig`、`FrontendExecutionAssumptionOverrides` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 parameter_grid 子模块 | `src/runtime/backtest/parameter_grid.rs` | 承载 3 个参数网格 helper |
| 父级声明私有模块 | `src/runtime/backtest/experiment_sweep.rs` | 增加 `mod parameter_grid;` 与 `use parameter_grid::build_experiment_overrides;` |
| 保留 handler 编排 | `src/runtime/backtest/experiment_sweep.rs` | `start_backtest_experiment` 继续负责 capability/config guard、variant loop、preview persistence 和 lifecycle |
| 保留常量 owner | `src/runtime/backtest.rs` | `MAX_EXPERIMENT_VARIANTS` 未移动 |

实际路径适配:

`src/runtime/backtest/experiment_sweep.rs` 当前是由父模块 `src/runtime/backtest.rs` include 进来的源片段；Rust 对 `mod parameter_grid;` 的解析点是 `src/runtime/backtest/parameter_grid.rs`。因此本批把实际文件落在编译器解析的父模块路径，未改变模块树层级和父子通信规则。

父级调用形态:

```rust
use super::*;

mod parameter_grid;

use parameter_grid::build_experiment_overrides;
```

子模块暴露形态:

```rust
use super::*;

pub(super) fn build_experiment_overrides(
    request: &FrontendExperimentRequest,
    qs_protocol: &RuntimeProtocolCoreConfig,
) -> Result<Vec<FrontendExecutionAssumptionOverrides>, (StatusCode, String)>
```

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| empty grid | 继续通过 provided value count 拒绝空参数网格 |
| 负数校验 | `fee_bps` / `slippage_bps` 仍返回 `parameter_grid.{field} 必须 >= 0` |
| base fallback | 空 fee/slippage/latency 轴继续回退到 base execution assumptions |
| dedupe | 继续使用 `Vec::contains` 保持原输入顺序去重 |
| variant count | 继续用 fee × slippage × latency，并受 `MAX_EXPERIMENT_VARIANTS` 限制 |
| expansion order | 保持 fee 外层、slippage 中层、latency 内层 |
| override 输出 | 每个 variant 继续输出 `Some(fee_bps)`、`Some(slippage_bps)`、`Some(latency_ms)` |
| handler 编排 | `start_backtest_experiment` 调用位置、variant suffix、preview persistence 和 lifecycle 不变 |
| execution_start 复用桥 | `execute_backtest_request` 仍由父级 handler 使用，不归 parameter_grid |

---

## 明确未迁移

- 不迁移 `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`。
- 不迁移 experiment route registration；route owner 仍是 `src/backend/runtime/routes.rs`。
- 不迁移 `runtime.backtest.execution_start` 或 `execute_backtest_request`。
- 不迁移 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare` 或 compare route。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`。
- 不迁移 `FrontendExecutionAssumptionSweepGrid`、`FrontendExecutionAssumptionOverrides`、`FrontendExperimentRequest` 或 schema owner。
- 不迁移 persistence、response mapping、state、audit、frontend caller、整理、重构或发布过渡连接。ASCII guard: `release transition guard`。

---

## 回退点

若后续发现行为回归，可将 `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` 从 `src/runtime/backtest/parameter_grid.rs` 放回 `src/runtime/backtest/experiment_sweep.rs` 顶部，并移除父级的 `mod parameter_grid;` 与 `use parameter_grid::build_experiment_overrides;`。不需要回退 route、schema、state 或 persistence 文件，因为本批未修改这些 owner。

---

## 验证计划

本批收口必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 BE-001W-04 `runtime.backtest.experiment_sweep.parameter_grid` 单叶 closeout，确认 3 个 helper 抽离后与原功能等价，并判断 parameter_grid 是否设置 `stop_split: true`。当前不能直接继续细拆 axis normalization、variant expansion、error mapping、schema、route、state/persistence、frontend caller 或发布过渡连接。ASCII marker: `next closeout marker`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.parameter_grid` 已抽离时，必须说明只迁移了 3 个参数网格 helper 到 `src/runtime/backtest/parameter_grid.rs`，且 `build_experiment_overrides` 只以 `pub(super)` 暴露给父级 `runtime.backtest.experiment_sweep`。不得宣称 parameter_grid 已 closeout、`stop_split: true` 已设置、schema/constant/route/shared owner 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `113-runtime.backtest.experiment_sweep.parameter_grid抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/backtest/parameter_grid.rs` 进入全量树和模块树。
3. `src/runtime/backtest/experiment_sweep.rs` 只保留父级 handler 编排，并通过 `pub(super)` helper 调用 parameter_grid。
4. `MAX_EXPERIMENT_VARIANTS`、schema、route、state、persistence、response mapping、audit 和 frontend caller owner 不变。
5. 治理门禁能发现本抽离记录、实际文件、3 个 helper、禁止迁移边界和回归证据。
6. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
