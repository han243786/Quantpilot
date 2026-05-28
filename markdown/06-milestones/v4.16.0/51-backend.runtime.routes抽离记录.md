# v4.16.0 backend.runtime.routes 抽离记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001F-02。
> 基准: `50-backend.runtime.routes单子叶等价基线.md`。
> 判定: `backend.runtime.routes` 已接管 runtime route aggregate 列表；真实 handler、state owner、event stream、artifact schema 和 persistence 仍保留在 `src/runtime/`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001F 单子叶抽离、runtime routes 等价验证 | 落地 |
| 规范矩阵 | 父子通信、route owner、handler 保留、state owner 冻结 | 固化 |
| 引导矩阵 | `backend.runtime.routes` | 抽离完成 |
| 模块树 | `backend.runtime.routes` | 更新白箱现状 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 runtime、根7.6 v4.16 |
| 模块树节点 | `backend.runtime.routes` |
| 真实文件 | `src/backend/runtime.rs`、`src/backend/runtime/routes.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs`、`src/backtest_compare.rs` |
| public 方法 | `backend.runtime::register_routes`、`backend.runtime.routes::register_routes`、runtime handler `pub(crate)` route targets |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run --test api_backtest --test api_sse`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 抽离结果

| 项 | 抽离前 | 抽离后 |
| --- | --- | --- |
| route aggregate owner | `src/runtime/mod.rs::register_runtime_routes` | `src/backend/runtime/routes.rs::register_routes` |
| handler owner | `src/runtime/*` 私有函数 | `src/runtime/*` `pub(crate)` route targets，代码仍保留原位 |
| compare route owner | `src/runtime/mod.rs` 调用 `compare_backtests` | `backend.runtime.routes` 调用 `backtest_compare::compare_backtests` |
| AppState / lock / persistence | `src/runtime/*` 和 `AppState` owner 保留 | 不迁移 |
| route path / method | `/api/runtime/*`、`/api/v1/*` runtime 相关路径 | 不改 |

代表 route: `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/backtest`、`/api/runtime/runs/:run_id/events`。

新的父子链路:

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> src/runtime/* pub(crate) handler
  -> existing state owner / persistence / event stream
```

---

## 保留边界

- `src/runtime/mod.rs` 不再拥有 runtime route aggregate 列表，但继续拥有 report、evidence、merge、generation、storage health 和 ops report handler。
- `src/runtime/run.rs` 继续拥有 run/v4 run/list/detail/save/replay/status/SSE handler。
- `src/runtime/backtest.rs` 继续拥有 backtest、experiment、backtest replay handler。
- `src/runtime/mutation.rs` 继续拥有 mutation、AI proposal、approval handler。
- `src/backtest_compare.rs` 只把 `compare_backtests` 提升为 `pub(crate)`，不迁移 compare 实现。

---

## 等价证据

| 证据 | 覆盖范围 | 判定 |
| --- | --- | --- |
| `cargo check -p quantpilot` | route facade、handler 可见性、Axum handler 类型 | 必须通过 |
| `cargo test -p quantpilot --test api_run` | run/v4 run/list/detail/save/replay/status/report | 必须通过 |
| `cargo test -p quantpilot --test api_backtest` | backtest/artifact/compare/replay/report source | 必须通过 |
| `cargo test -p quantpilot --test api_sse` | `/api/runtime/runs/:run_id/events` | 必须通过 |
| `tools/check-matrix-governance.ps1` | 本抽离记录、模块树、全量树锚点 | 必须通过 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 必须通过 |

---

## 本批次不做

- 不移动 runtime handler 代码到 `src/backend/runtime/routes.rs`。
- 不拆 `runtime.run`、`runtime.backtest`、`runtime.event_stream`、`runtime.mutation_ai_proposal` 或 `runtime.report_experiment`。
- 不改 `/api/runtime/*` 或 `/api/v1/*` runtime 相关 path/method/payload/response schema。
- 不迁移 AppState 字段 owner、runtime state owner、event stream、artifact schema 或 persistence。
- 不宣称 `backend.runtime` 或 `root.backend` 完成。

---

## 后续递归入口

如果继续 runtime，必须从 `50` 登记的候选中单选一个子叶:

| 候选 | 下一步 |
| --- | --- |
| `runtime.run` | 建等价基线后再迁移 run/v4 run handler |
| `runtime.backtest` | 建等价基线后再迁移 backtest/artifact/compare/replay |
| `runtime.event_stream` | 建等价基线后再迁移 SSE/replay cursor |
| `runtime.mutation_ai_proposal` | 建等价基线后再迁移 mutation/AI proposal/approval |
| `runtime.report_experiment` | 先判断 owner，再决定是否拆分 |
