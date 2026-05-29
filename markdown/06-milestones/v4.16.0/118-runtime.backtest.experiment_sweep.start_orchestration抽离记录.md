# v4.16.0 runtime.backtest.experiment_sweep.start_orchestration 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001Y-03。  
> 基准: `117-runtime.backtest.experiment_sweep.start_orchestration抽离方案.md`、`116-runtime.backtest.experiment_sweep.start_orchestration单子叶等价基线.md`、`115-runtime.backtest.experiment_sweep父叶残余判断.md`。  
> 判定: 按方案完成 `runtime.backtest.experiment_sweep.start_orchestration` 第一轮实际抽离；只迁移 `start_backtest_experiment`，不迁移 record lifecycle、route registration、schema、state、persistence、response mapping、audit、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001Y start_orchestration 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级私有子模块、受控 re-export、父子通信、禁止横向连接 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` | 物理抽离 |
| 模块树 | `runtime.backtest.experiment_sweep.start_orchestration` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.start_orchestration` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.start_orchestration` |
| 新真实文件 | `src/runtime/backtest/start_orchestration.rs` |
| 父级真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 保留真实文件 | `src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` |
| 保留子节点 | `runtime.backtest.experiment_sweep.parameter_grid` |
| public 方法 | `start_backtest_experiment` |
| 已迁移方法 | `start_backtest_experiment` |
| 父级 re-export | `pub(crate) use start_orchestration::start_backtest_experiment;` |
| 子模块导入 | `use super::*;`、`use super::parameter_grid::build_experiment_overrides;` |
| 保留 sibling | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 保留复用桥 | `execute_backtest_request` |
| 保留输出类型 | `ExperimentRecord`、`ExperimentVariantSummary`、`ExperimentDetailResponse`、`FrontendRunRequest` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 start_orchestration 子模块 | `src/runtime/backtest/start_orchestration.rs` | 承载 `start_backtest_experiment` 创建编排 handler |
| 父级声明私有模块 | `src/runtime/backtest/experiment_sweep.rs` | 增加 `mod start_orchestration;` |
| 父级受控出口 | `src/runtime/backtest/experiment_sweep.rs` | 增加 `pub(crate) use start_orchestration::start_backtest_experiment;` |
| 保留参数网格子模块 | `src/runtime/backtest/parameter_grid.rs` | `build_experiment_overrides` 仍归 parameter_grid |
| 保留 record lifecycle | `src/runtime/backtest/experiment_sweep.rs` | `list/get/save/discard` 四个 handler 不迁移 |

父级形态:

```rust
use super::*;

mod parameter_grid;
mod start_orchestration;

pub(crate) use start_orchestration::start_backtest_experiment;
```

子模块形态:

```rust
use super::parameter_grid::build_experiment_overrides;
use super::*;

pub(crate) async fn start_backtest_experiment(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<FrontendExperimentRequest>,
) -> Result<Json<ExperimentDetailResponse>, (StatusCode, String)>
```

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| capability guard | `validate_runtime_capability_guard` 仍先执行，失败 code 为 `capability_boundary_violation` |
| runtime config guard | `validate_runtime_config_capabilities` 仍映射 `capability_gated` |
| execution assumption guard | `validate_backtest_execution_assumption_overrides` 仍映射 `bad_request` |
| graph requirement | `graph_json` 缺失仍返回 `bad_request` 和原 message |
| QS compile | `compile_runtime_protocol_via_qs(graph_json)` 仍在 grid 前执行 |
| base assumptions | `resolved_backtest_execution_assumptions` 仍生成 base fee/slippage/latency |
| parameter grid | 仍调用 `build_experiment_overrides(&request, &qs_protocol)`，不复制实现 |
| replay source | 缺失时仍回退 `FrontendBacktestReplaySource::HistoricalReplay` |
| identity | `experiment_{current_time_ms()}` 与 `created_at_ms` 语义不变 |
| actor/name | `normalize_actor_identity` 和空白 experiment name 转 `None` 不变 |
| variant request | 每个 override 仍组装完整 `FrontendRunRequest` |
| execution bridge | 每个 variant 仍调用 `execute_backtest_request`，suffix 为 `{experiment_id}_v{n}` |
| summary/tag | summary fallback 和 execution assumptions tag 来源不变 |
| preview persistence | 仍先 `persist_experiment_record`，再写 `state.experiments` scoped cache |
| response mapping | 仍通过 `experiment_detail_response_from_record(record)` 返回 |

---

## 明确未迁移

- 不迁移 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`。
- 不迁移 `record_lifecycle`；该候选必须等 BE-001Y-04 closeout 后再判断。
- 不迁移 route registration；route owner 仍是 `backend.runtime.routes` / backtest route facade。
- 不迁移 `execute_backtest_request` 或 `runtime.backtest.execution_start`。
- 不迁移 `parameter_grid` helper、`MAX_EXPERIMENT_VARIANTS` 或 schema owner。
- 不迁移 persistence、response mapping、state、audit、frontend caller、整理、重构或发布过渡连接。ASCII guard: `release transition guard`。

---

## 回退点

若后续发现行为回归，可将 `start_backtest_experiment` 从 `src/runtime/backtest/start_orchestration.rs` 放回 `src/runtime/backtest/experiment_sweep.rs` 顶部，并移除父级的 `mod start_orchestration;` 与 `pub(crate) use start_orchestration::start_backtest_experiment;`。不需要回退 route、schema、state、persistence、response mapping 或 audit 文件，因为本批未修改这些 owner。

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

下一批应进入 BE-001Y-04 `runtime.backtest.experiment_sweep.start_orchestration` 单叶 closeout，确认 `start_backtest_experiment` 抽离后与原功能等价，并判断该子叶是否设置 `stop_split: true`。当前不能直接移动 record lifecycle、删除 drained parent include、迁移 route/schema/state/persistence/response mapping/audit/frontend caller，或启动发布过渡连接。ASCII marker: `next closeout marker`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.start_orchestration` 已抽离时，必须说明只迁移了 `start_backtest_experiment` 到 `src/runtime/backtest/start_orchestration.rs`，并通过父级 `pub(crate) use` 保持 `crate::runtime::start_backtest_experiment` 兼容出口。不得宣称 start orchestration 已 closeout、`stop_split: true` 已设置、record lifecycle、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `118-runtime.backtest.experiment_sweep.start_orchestration抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/backtest/start_orchestration.rs` 进入全量树和模块树。
3. `src/runtime/backtest/experiment_sweep.rs` 保留父级私有模块声明、受控 re-export 和 record lifecycle sibling。
4. `src/runtime/mod.rs` 与 route registration 行为不变。
5. 治理门禁能发现本抽离记录、实际文件、`start_backtest_experiment`、禁止迁移边界和回归证据。
6. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
