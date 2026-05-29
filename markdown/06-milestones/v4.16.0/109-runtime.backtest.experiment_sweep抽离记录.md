# v4.16.0 runtime.backtest.experiment_sweep 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001V-03。  
> 基准: `108-runtime.backtest.experiment_sweep抽离方案.md`、`107-runtime.backtest.experiment_sweep单子叶等价基线.md`、`106-runtime.backtest.replay单叶closeout.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: 按方案完成 `runtime.backtest.experiment_sweep` 第一轮实际抽离；只迁移 5 个 experiment handler 和 3 个参数网格 helper，不迁移 route aggregate、execution_start、persistence、response mapping、schema、state、audit、frontend caller 或发布过渡连接。  

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001V experiment_sweep 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、drained parent include、route aggregate 保留、shared owner 保留 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` | 物理抽离 |
| 模块树 | `runtime.backtest.experiment_sweep` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep` |
| 新真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 保留真实文件 | `src/runtime/backtest.rs`、`src/runtime/backtest/execution_start.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public handler | `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 同迁 helper | `normalize_experiment_float_axis`、`normalize_experiment_latency_axis`、`build_experiment_overrides` |
| 保留 shared helper | `execute_backtest_request`、`persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state`、`persist_backtest_record`、`delete_transient_backtest_record`、`experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 保留 public 类型 | `FrontendExperimentRequest`、`FrontendExecutionAssumptionSweepGrid`、`FrontendExecutionAssumptionOverrides`、`FrontendBacktestReplaySource`、`ExperimentRecord`、`ExperimentDefinitionSummary`、`ExperimentVariantSummary`、`ExperimentListItem`、`ExperimentDetailResponse`、`DiscardRuntimeArtifactResponse` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 experiment_sweep 子模块 | `src/runtime/backtest/experiment_sweep.rs` | 承载 5 个 experiment handler 和 3 个参数网格 helper |
| 清空旧 handler 残留 | `src/runtime/backtest.rs` | 仅保留 drained parent include 注释，等待后续 closeout/父叶残余判断 |
| 父级兼容出口 | `src/runtime/mod.rs` | 增加 `backtest_experiment_sweep` 私有子模块和 `pub(crate) use` |
| route aggregate | `src/backend/runtime/routes.rs` | 未改动，仍通过 `crate::runtime::*` handler 名调用 |

父级 re-export 形态:

```rust
#[path = "backtest/experiment_sweep.rs"]
mod backtest_experiment_sweep;
pub(crate) use backtest_experiment_sweep::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
    start_backtest_experiment,
};
```

子模块形态:

```rust
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
| experiment route | `/api/runtime/experiments/*` path、method、route aggregate 和 handler 名不变 |
| 参数网格 | 空网格拒绝、负数校验、去重顺序、空轴 base fallback、`MAX_EXPERIMENT_VARIANTS` 上限不变 |
| 创建 sweep | `start_backtest_experiment` 仍执行 capability guard、config capability guard、execution assumption override guard |
| backtest 复用桥 | 仍通过父级 runtime 内部 `execute_backtest_request` 生成每个 variant backtest |
| variant id | 仍使用 `experiment_{timestamp}` 和 `{}_v{index}` suffix |
| preview persistence | 创建时继续 `persist_experiment_record` 并写入 `state.experiments` |
| 列表/详情 | 仍经 `list_experiment_records`、`load_experiment_record_from_state` 和 response mapping owner 输出 |
| 保存 lifecycle | `save_experiment_record` 继续持久化 variant backtests、清理 transient 并写 graph audit |
| 丢弃 lifecycle | `discard_experiment_record` 继续拒绝 saved experiment，并清理 experiment 文件、state 和 transient variant backtests |
| schema/mapping/state | `FrontendExperimentRequest`、`ExperimentRecord`、`ExperimentDetailResponse`、response mapping、AppState 和锁 owner 保留原位 |

---

## 明确未迁移

- 不迁移 experiment route registration；`src/backend/runtime/routes.rs` 仍是真实 route aggregate owner。
- 不迁移 `runtime.backtest.execution_start`；`execute_backtest_request` 仍是父级 runtime 内部复用桥。
- 不迁移 `runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare` 或 compare route。
- 不私有化 `persist_experiment_record`、`list_experiment_records`、`load_experiment_record_from_state`、`persist_backtest_record`、`delete_transient_backtest_record`。
- 不私有化 `experiment_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_sweep_axes`。
- 不私有化 schema owner、AppState owner、store dir、lock owner、graph audit owner 或 frontend caller。
- 不进入整理、重构、发布过渡或横向连接优化。ASCII guard: `release transition guard`。

---

## 回退点

若后续发现行为回归，可将 5 个 handler 和 3 个 helper 从 `src/runtime/backtest/experiment_sweep.rs` 放回 `src/runtime/backtest.rs`，并移除 `src/runtime/mod.rs` 中的 `backtest_experiment_sweep` 私有模块与 re-export。`src/backend/runtime/routes.rs` 不需要回退，因为本批未修改 route aggregate。

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

下一批应进入 BE-001V-04 `runtime.backtest.experiment_sweep` 单叶 closeout，确认本叶抽离后与原功能等价，并判断内部是否值得继续细拆。当前不能直接删除 `src/runtime/backtest.rs` drained parent include，也不能把 route aggregate、execution_start、persistence、response mapping、schema、state、audit、frontend caller 或发布过渡连接混入 closeout。ASCII marker: `next closeout marker`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep` 已抽离时，必须说明只迁移了 5 个 experiment handler 和 3 个参数网格 helper 到 `src/runtime/backtest/experiment_sweep.rs`，并通过 `src/runtime/mod.rs` 父级 re-export 保持 `crate::runtime::*` 兼容出口。不得宣称 route aggregate、execution_start、record_store、replay、compare、persistence owner、response mapping owner、schema owner、state owner、audit owner、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `109-runtime.backtest.experiment_sweep抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/backtest/experiment_sweep.rs` 进入全量树和模块树。
3. `src/runtime/mod.rs` 保留 `crate::runtime::{start_backtest_experiment,list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}` 兼容出口。
4. `src/backend/runtime/routes.rs` route path/method 和 handler 调用不变。
5. 治理门禁能发现本抽离记录缺失、禁止迁移边界和回归证据缺失。
6. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
