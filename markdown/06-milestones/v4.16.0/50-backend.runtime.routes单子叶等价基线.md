# v4.16.0 backend.runtime.routes 单子叶等价基线

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001F-01。
> 基准: `34-backend.runtime单叶closeout.md`、`44-backend.runtime子叶抽离完成记录.md`。
> 判定: 选择 `backend.runtime.routes` 作为 BE-001F 首个单子叶等价基线；本批只固定 route aggregate facade 的输入、输出、真实 owner、等价证据和禁止迁移边界，不迁移 runtime handler、state owner、event stream 或 persistence。

---

## 选择理由

`backend.runtime.routes` 是高价值但可控的单子叶:

1. 它处在 `backend.interface_boundary -> backend.runtime -> backend.runtime.routes -> src/runtime/mod.rs` 的父子链路上，符合递归流程。
2. 它覆盖 runtime run、backtest、SSE、mutation、AI proposal、report、experiment 等接口面，值得先建立等价基线。
3. 当前只做 route aggregate facade，不触碰 runtime state owner、event stream、artifact schema 或 persistence。
4. 它已有 `api_run`、`api_backtest`、`api_sse` 等强回归证据，适合做单子叶整理/等价基线试点。

不选择 `backend.storage_security.*`，因为安全域仍保留安全决策暂停；不选择横向多个子叶，因为 BE-001E 后续必须进入单子叶递归。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001F 单子叶等价基线、backend R5 局部递归 | 扩展 |
| 规范矩阵 | runtime route owner、state owner 冻结、父子通信 | 固化 |
| 引导矩阵 | `backend.runtime.routes` 白箱节点 | 扩展 |
| 模块树 | `backend.runtime.routes` | 建立单子叶基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 runtime、根7.6 v4.16 |
| 模块树节点 | `backend.runtime.routes` |
| 真实文件 | `src/backend/runtime.rs`、`src/backend/runtime/routes.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs`、`src/runtime_event_projection.rs`、`src/runtime_persistence.rs`、`src/backtest_artifacts.rs` |
| public 方法 | `backend.runtime::register_routes`、`backend.runtime.routes::register_routes`、`crate::runtime::register_runtime_routes`、`/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/backtest`、`/api/runtime/runs/:run_id/events` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_sse`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | Axum Router | `backend.interface_boundary`、`backend.runtime` | 不改变 route registration 顺序 |
| 输入 | `AppState` | `backend.app_state_wiring`、`src/app_runtime_helpers.rs` | 不迁移 AppState 字段 owner 或锁顺序 |
| 输入 | runtime HTTP request | frontend、tests、local API caller | 不改 `/api/runtime/*` path、method、payload 或 error code |
| 输出 | Runtime routes | Axum Router | 仍由 `crate::runtime::register_runtime_routes` 真正注册 |
| 输出 | Runtime response | frontend、tests | 不改 run/backtest/report/experiment response schema |
| 输出 | Runtime event stream | frontend SSE panel、tests | 不改 SSE frame、event envelope 或 replay cursor |

---

## route owner 基线

| 接口域 | facade owner | 真实 owner | 代表入口 | 当前处理 |
| --- | --- | --- | --- | --- |
| runtime aggregate | `backend.runtime.routes` | `src/runtime/mod.rs` | `register_runtime_routes` | 只保留 route aggregate facade |
| test run | `backend.runtime.routes` | `src/runtime/run.rs` through `src/runtime/mod.rs` | `/api/runtime/test-run` | 不迁移 handler |
| v4 run | `backend.runtime.routes` | `src/runtime/run.rs`、runtime validation/evidence helpers | `/api/runtime/v4/run` | 不迁移 state owner |
| run record/list/detail | `backend.runtime.routes` | `src/runtime/mod.rs`、`src/runtime_persistence.rs` | `/api/runtime/runs`、`/api/runtime/runs/:run_id` | 不改 persistence projection |
| run SSE/replay/status | `backend.runtime.routes` | `src/runtime/mod.rs`、`src/runtime_event_projection.rs` | `/api/runtime/runs/:run_id/events`、`/replay`、`/status` | 不改 event envelope |
| backtest | `backend.runtime.routes` | `src/runtime/backtest.rs`、`src/backtest_artifacts.rs` | `/api/runtime/backtest`、`/api/runtime/backtests/*` | 不改 artifact schema |
| mutation / AI proposal | `backend.runtime.routes` | `src/runtime/mutation.rs` | `/api/runtime/mutations/*`、`/api/runtime/ai-proposals/*` | 不改 approval/audit rules |
| report / experiment | `backend.runtime.routes` | `src/runtime/mod.rs`、`src/runtime/backtest.rs` | `/api/runtime/reports/*`、`/api/runtime/experiments/*` | 不改 report source binding |

---

## 兼容桥

本基线建立时的兼容桥为:

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> crate::runtime::register_runtime_routes
  -> existing runtime handler
  -> existing runtime state owner
```

本批允许继续使用该桥证明等价；不允许把 `src/runtime/mod.rs` 的真实 handler 拆入 `src/backend/runtime/routes.rs`。

`51-backend.runtime.routes抽离记录.md` 已在本基线之后接管 route aggregate 列表；该后续抽离只迁移 route owner，不迁移真实 handler。

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | Rust 模块与 Axum route 类型 | facade 委托链不断 |
| `cargo test -p quantpilot --test api_run` | run、v4 run、list/detail/save/replay/status/report | runtime routes 和 response schema 不漂移 |
| `cargo test -p quantpilot --test api_backtest` | backtest、artifact、compare、replay、report source | backtest route 和 artifact schema 不漂移 |
| `cargo test -p quantpilot --test api_sse` | run event stream | SSE frame 和 event envelope 不漂移 |
| `tools/check-matrix-governance.ps1` | 单子叶文档和模块树锚点 | 治理入口不丢 |
| `tools/check-full-feature-tree.ps1` | 全量树路径覆盖 | 新基线和真实文件可定位 |

---

## 本批次不做

- 不移动 `src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/backtest.rs`、`src/runtime/mutation.rs`。
- 不迁移 runtime handler。
- 不迁移 runtime state owner、AppState 字段 owner、锁顺序、event stream 或 persistence。
- 不改 `/api/runtime/*` route、method、payload、response schema 或 error code。
- 不把 report、experiment、AI proposal 或 mutation 作为独立子叶迁移。
- 不宣称 `backend.runtime` 或 `root.backend` 已完成。

---

## 后续判断

`backend.runtime.routes` 本轮先建立等价基线。后续如果继续拆，必须在新提案中从下列候选里单选一个子叶，并补独立等价证据:

| 候选子叶 | 是否值得继续拆 | 原因 |
| --- | :--: | --- |
| `runtime.run` | 值得 | run/v4 run、status、record owner 独立 |
| `runtime.backtest` | 值得 | backtest artifact、compare、replay 和 report source 独立 |
| `runtime.event_stream` | 值得 | SSE、event envelope、replay cursor 可单独验证 |
| `runtime.mutation_ai_proposal` | 值得 | mutation approval、AI proposal audit 和 capability gate 独立 |
| `runtime.report_experiment` | 待判断 | report 与 experiment 共享 runtime source binding，需要先评估 owner |

---

## 验收标准

1. `50-backend.runtime.routes单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `backend.runtime.routes` 白箱节点。
3. 全量树能定位本基线和真实 runtime 文件。
4. 治理门禁能发现本文件缺失。
5. 后续 runtime 继续拆分必须引用本基线，不得绕过父模块直接迁移 handler。
