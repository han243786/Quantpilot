# v4.16.0 backend.runtime.routes.experiment 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BS-04  
> 基准: `231-backend.runtime.routes.experiment单子叶等价基线.md`、`232-backend.runtime.routes.experiment抽离方案.md`、`233-backend.runtime.routes.experiment抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: `backend.runtime.routes.experiment` route facade 已完成等价 closeout，并设置 `stop_split: true`。本叶只承接五个 experiment route registration；继续拆成 sweep/list/save/detail/discard 微 facade 不会形成新的稳定 owner。下一步只能回到 BE-001BT-01 `backend.runtime.routes` 父叶残余判断。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BS 从实际抽离进入单叶 closeout；后续回到父叶残余判断 | 收口 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.experiment` | closeout |
| 模块树 | `backend.runtime.routes.experiment` | 设置 `stop_split: true` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.experiment` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes/experiment.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.experiment` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/backend/runtime/routes/experiment.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/record_lifecycle.rs`、`src/runtime/mod.rs` |
| public 方法 | `backend.runtime.routes.experiment::register_routes`、`start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、`git diff --check` |

---

## 等价判定

| 检查项 | 结论 |
| --- | --- |
| route path | 等价。五个 experiment route path 均由 `src/backend/runtime/routes/experiment.rs` 注册 |
| route method | 等价。POST/GET/DELETE 绑定保持不变 |
| handler 调用 | 等价。仍调用 `runtime_handlers::*`，handler owner 未迁移 |
| 父级委托 | 等价。`src/backend/runtime/routes.rs` 仍通过 `experiment::register_routes(router)` 接入 |
| route order | 等价。保持 reports -> experiment -> ops 相对顺序 |
| response schema | 未变更 |
| AppState / 锁 / 持久化 | 未变更 |
| frontend caller | 未变更 |
| 发布过渡 | 未启动，不新增横向连接或性能旁路 |

---

## 当前 route facade 结果

| route | method | 当前 owner | handler |
| --- | --- | --- | --- |
| `/api/runtime/experiments/backtest-sweep` | POST | `src/backend/runtime/routes/experiment.rs` | `runtime_handlers::start_backtest_experiment` |
| `/api/runtime/experiments` | GET | `src/backend/runtime/routes/experiment.rs` | `runtime_handlers::list_experiments` |
| `/api/runtime/experiments/:experiment_id/save` | POST | `src/backend/runtime/routes/experiment.rs` | `runtime_handlers::save_experiment_record` |
| `/api/runtime/experiments/:experiment_id` | GET | `src/backend/runtime/routes/experiment.rs` | `runtime_handlers::get_experiment_detail` |
| `/api/runtime/experiments/:experiment_id` | DELETE | `src/backend/runtime/routes/experiment.rs` | `runtime_handlers::discard_experiment_record` |

---

## 保留边界

- `start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record` 与 `discard_experiment_record` 的 handler owner 仍在 `runtime.backtest.experiment_sweep` / `start_orchestration` / `record_lifecycle` 子树。
- `AppState`、experiments/backtests state、store dir、runtime persistence owner、artifact schema owner、compare owner、schema owner 和 frontend caller 均未迁移。
- `backend.runtime.routes.evidence`、`backend.runtime.routes.report_ops` 与 `backend.runtime.routes.event_stream` 仍是父叶后续候选，不属于本 closeout。
- 不主动提出发布版本过渡，不新增子模块横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 细分价值判断

`backend.runtime.routes.experiment` 本身停止细分，设置 `stop_split: true`。

理由:

| 候选微叶 | 判断 | 理由 |
| --- | --- | --- |
| sweep route | 不拆 | 只是单条 route registration，handler owner 不在 route facade 内 |
| list route | 不拆 | 继续拆只会增加父级 import 面，不产生独立状态 owner |
| save route | 不拆 | persistence 语义仍归 handler / runtime persistence owner |
| detail / discard route | 不拆 | scoped lookup 与 preview-only guard 均由 handler owner 维护 |

本叶只是一层 route registration facade。继续拆成多个微文件会让父级接线变碎，却无法减少真实耦合，因此停止内部递归。

---

## 下一步

下一批只能回到父叶:

```text
BE-001BT-01 backend.runtime.routes 父叶残余判断
```

父叶残余判断应重新检查 direct route residual:

- `backend.runtime.routes.evidence`
- `backend.runtime.routes.report_ops`
- `backend.runtime.routes.event_stream`

不得从本 closeout 直接跳到 evidence/report_ops/event_stream 实际抽离；必须先做父叶残余判断并选择下一候选。

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BS-04 完成时，必须说明只完成 `backend.runtime.routes.experiment` route facade closeout，并设置 `stop_split: true`。不得宣称 experiment handler、AppState、schema owner、frontend caller、runtime persistence owner 已迁移；不得宣称 `backend.runtime.routes` 父叶完成；不得宣称发布过渡、整理或重构已经完成。

---

## 验收标准

1. `234-backend.runtime.routes.experiment单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `backend.runtime.routes.experiment` 设置 `stop_split: true`。
3. closeout 明确 handler、AppState、schema、frontend caller、runtime persistence owner 和 release transition guard 均未改变。
4. closeout 明确下一步只能进入 BE-001BT-01 `backend.runtime.routes` 父叶残余判断。
5. 治理验证通过。
