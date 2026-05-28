# v4.16.0 backend.runtime.routes.run 抽离记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001G-02。
> 基准: `52-backend.runtime.routes.run单子叶等价基线.md`。
> 判定: `backend.runtime.routes.run` 已接管 run route group；真实 handler、state owner、event stream、persistence 和 response schema 仍保留在 `src/runtime/run.rs` 与既有 runtime helpers 中。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001G 单子叶抽离、runtime run route 等价验证 | 落地 |
| 规范矩阵 | 父子通信、run route owner、event stream 排除、state owner 冻结 | 固化 |
| 引导矩阵 | `backend.runtime.routes.run` | 抽离完成 |
| 模块树 | `backend.runtime.routes.run` | 更新白箱现状 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend runtime、根7.6 v4.16 |
| 模块树节点 | `backend.runtime.routes.run` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime/run.rs`、`src/runtime_persistence.rs`、`src/runtime_event_projection.rs` |
| public 方法 | `backend.runtime.routes::register_routes`、`backend.runtime.routes.run::register_routes`、`run::register_routes`、`start_test_run`、`start_v4_runtime_run`、`list_runs`、`save_run_record`、`get_run_detail`、`discard_run_record`、`get_run_replay`、`get_run_status` |
| route 坐标 | `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/runs`、`/api/runtime/runs/:run_id`、`/api/runtime/runs/:run_id/save`、`/api/runtime/runs/:run_id/replay`、`/api/runtime/runs/:run_id/status` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_sse`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 抽离结果

| 项 | 抽离前 | 抽离后 |
| --- | --- | --- |
| run route group owner | `src/backend/runtime/routes.rs::register_routes` 直接挂载 run routes | `src/backend/runtime/routes/run.rs::register_routes` 独立挂载 run routes |
| runtime route aggregate owner | `backend.runtime.routes` | 保留，调用 `run::register_routes` 后继续挂载 event stream、evidence、mutation、report、experiment 等 routes |
| run handler owner | `src/runtime/run.rs` `pub(crate)` route targets | 保留原位，不移动 |
| event stream owner | `backend.runtime.routes` 直接挂载 `/api/runtime/runs/:run_id/events` | 保留在父 aggregate，本批不迁入 `backend.runtime.routes.run` |
| route path / method | `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/runs*` | 不改 |

新的父子链路:

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> src/runtime/run.rs pub(crate) handler
  -> existing state owner / persistence
```

---

## 保留边界

- `src/runtime/run.rs` 继续拥有 `start_test_run`、`start_v4_runtime_run`、`list_runs`、`save_run_record`、`get_run_detail`、`discard_run_record`、`get_run_replay`、`get_run_status` 和 `stream_run_events` 的真实实现。
- `/api/runtime/runs/:run_id/events` event stream 仍在 `backend.runtime.routes` 父 aggregate 中登记，后续如果要拆必须走 `runtime.event_stream` 单子叶基线。
- `src/runtime_persistence.rs` 和 `src/runtime_event_projection.rs` 继续是 persistence / replay / event projection 的真实 owner。
- `backend.runtime.routes.run` 不形成对外 public API；它只是 crate 内部 route facade。

---

## 等价证据

| 证据 | 覆盖范围 | 判定 |
| --- | --- | --- |
| `cargo check -p quantpilot` | route facade、handler 可见性、Axum handler 类型 | 必须通过 |
| `cargo test -p quantpilot --test api_run` | run/v4 run/list/detail/save/replay/status/report | 必须通过 |
| `cargo test -p quantpilot --test api_sse` | event stream 保留在父 aggregate 后的可用性 | 必须通过 |
| `tools/check-matrix-governance.ps1` | 本抽离记录、模块树、全量树锚点 | 必须通过 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 必须通过 |

---

## 本批次不做

- 不移动 `src/runtime/run.rs` handler 代码到 `src/backend/runtime/routes/run.rs`。
- 不接管 `/api/runtime/runs/:run_id/events` event stream。
- 不迁移 AppState 字段 owner、runtime state owner、lock order、persistence owner 或 event projection owner。
- 不改 run route path、method、payload、response schema 或 error code。
- 不宣称 `backend.runtime.routes.run` 已完成 handler 整理或更细拆分判断。
- 不宣称 `backend.runtime.routes`、`backend.runtime` 或 `root.backend` 已完成。

---

## 后续递归入口

`backend.runtime.routes.run` 抽离后，递归流程应先做本子叶整理/等价 closeout，再判断是否值得继续拆 handler 内部。若继续 `backend.runtime.routes` 的其他子叶，候选保持单选:

| 候选 | 下一步 |
| --- | --- |
| `runtime.event_stream` | 建等价基线后再迁移 `/api/runtime/runs/:run_id/events` |
| `runtime.backtest` | 建等价基线后再迁移 backtest/artifact/compare/replay |
| `runtime.mutation_ai_proposal` | 建等价基线后再迁移 mutation/AI proposal/approval |
| `runtime.report_experiment` | 先判断 owner，再决定是否拆分 |

---

## 验收标准

1. `53-backend.runtime.routes.run抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/backend/runtime/routes/run.rs` 存在且只注册 run route group。
3. `backend.runtime.routes::register_routes` 通过 `run::register_routes` 委托 run routes。
4. `/api/runtime/runs/:run_id/events` event stream 未被混入本子叶。
5. `api_run` 和 `api_sse` 能证明 run route 与 event stream 保持等价。
