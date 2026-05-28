# v4.16.0 runtime.backtest 抽离记录

> 版本类型: MINOR architecture / governance.  
> 执行档位: 标准。  
> 批次: BE-001M-03。  
> 基准: `74-runtime.backtest单子叶等价基线.md`、`75-runtime.backtest抽离方案.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只完成 backtest route facade 最小抽离；不迁移 handler、artifact schema、compare owner、persistence owner、replay helper、state owner、schema owner、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001M `runtime.backtest` 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | backtest route facade 独立、父子通信规则保持、handler owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest` | route facade 抽离 |
| 模块树 | `backend.runtime.routes`、`runtime.backtest` 白箱节点 | 补真实文件与状态 |

---

## 实际变更

| 项 | 结果 |
| --- | --- |
| 新增 facade | `src/backend/runtime/routes/backtest.rs` |
| 父级 route aggregate | `src/backend/runtime/routes.rs` 新增 `pub mod backtest;` 并调用 `backtest::register_routes(router)` |
| 保持顺序 | 先注册 backtest routes，再注册 run routes，再注册 event stream、evidence、mutation、report、experiment、approval 与 ops routes |
| handler owner | `src/runtime/backtest.rs` 保持不变，继续通过 `crate::runtime::*` 暴露既有 handler |
| compare owner | `src/backtest_compare.rs` 保持不变，route facade 只调用 `backtest_compare::compare_backtests` |
| run / event stream | `src/backend/runtime/routes/run.rs` 与 `src/runtime/event_stream.rs` 均不变 |
| artifact / persistence / schema | `src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs` 均不变 |

---

## Route 等价对照

| route | method | 抽离前 handler | 抽离后 handler |
| --- | --- | --- | --- |
| `/api/runtime/backtest` | POST | `runtime_handlers::start_backtest_run` | `runtime_handlers::start_backtest_run` |
| `/api/runtime/backtests` | GET | `runtime_handlers::list_backtests` | `runtime_handlers::list_backtests` |
| `/api/runtime/backtests/compare` | POST | `backtest_compare::compare_backtests` | `backtest_compare::compare_backtests` |
| `/api/runtime/backtests/:backtest_id/save` | POST | `runtime_handlers::save_backtest_record` | `runtime_handlers::save_backtest_record` |
| `/api/runtime/backtests/:backtest_id` | GET | `runtime_handlers::get_backtest_detail` | `runtime_handlers::get_backtest_detail` |
| `/api/runtime/backtests/:backtest_id` | DELETE | `runtime_handlers::discard_backtest_record` | `runtime_handlers::discard_backtest_record` |
| `/api/runtime/backtests/:backtest_id/replay` | GET | `runtime_handlers::get_backtest_replay` | `runtime_handlers::get_backtest_replay` |

---

## 明确未迁移

- 不迁移 `src/runtime/backtest.rs` 中的 `start_backtest_run`、`list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay`。
- 不迁移 `execute_backtest_request`、`execute_v4_backtest_request`、`build_backtest_artifact_views`、`maybe_spill_transient_backtest_record`、`load_backtest_record_from_state`、`persist_backtest_record`、`normalized_replay_options`。
- 不迁移 `src/backtest_compare.rs`、`src/backtest_artifacts.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`。
- 不把 `start_backtest_experiment`、`/api/runtime/experiments/*`、report、mutation、approval、event stream、run routes 放入 backtest route facade。
- `runtime.event_stream` 与 `runtime.run` 明确排除在本批之外。
- 不改变 route path、method、handler 调用、response schema、error code、AppState 字段、store dir 或锁顺序。
- 不主动提出发布版本过渡，不新增子模块横向连接。ASCII guard: `release transition guard`。

---

## 父子通信结果

`backend.runtime.routes` 现在通过两个 route child facade 承载已拆分的 route group:

1. `backend.runtime.routes.backtest` 只注册 backtest route group。
2. `backend.runtime.routes.run` 只注册 run route group。
3. 其余 event stream、evidence、mutation、report、experiment、approval 和 ops routes 仍留在父级 aggregate，等待后续独立基线或 closeout 判断。

`runtime.backtest` handler 域仍是后续候选，不因本批 route facade 抽离而宣告完成。

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

下一批进入 `BE-001M-04 runtime.backtest 单叶 closeout`。closeout 只允许判断 route facade 这一刀是否等价，以及 `runtime.backtest` handler 域是否值得继续细拆；不得直接移动 handler、artifact、compare、persistence、schema、state 或 frontend owner。

---

## 幻觉检查点

AI 声称 `runtime.backtest` 已完成抽离时，必须说明本批只完成 backtest route registration 到 `src/backend/runtime/routes/backtest.rs` 的 facade 抽离。不得宣称 handler、artifact schema、compare owner、persistence owner、state owner、schema owner、frontend caller、experiment/report/mutation 或发布过渡已经迁移。

---

## 验收标准

1. `src/backend/runtime/routes/backtest.rs` 存在并只注册 backtest route group。
2. `src/backend/runtime/routes.rs` 通过 `backtest::register_routes(router)` 接入 backtest route facade，并保持 run route、event stream、evidence、mutation、report、experiment、approval 与 ops route 语义不变。
3. 所有 backtest route path、method、handler 调用保持等价。
4. `src/runtime/backtest.rs`、`src/backtest_compare.rs`、artifact、persistence、response mapping、frontend API schema 和 AppState owner 均保持原位。
5. 模块树、全量树、里程碑索引和治理门禁均能定位本抽离记录与新增 route facade。
