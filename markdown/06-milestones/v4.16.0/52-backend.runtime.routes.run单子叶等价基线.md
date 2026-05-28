# v4.16.0 backend.runtime.routes.run 单子叶等价基线

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001G-01。
> 基准: `50-backend.runtime.routes单子叶等价基线.md`、`51-backend.runtime.routes抽离记录.md`。
> 判定: 选择 `backend.runtime.routes.run` 作为 `backend.runtime.routes` 下的第一片递归子叶；本批只固定 run route facade 的输入、输出、真实 owner、等价证据和禁止迁移边界，不迁移 `src/runtime/run.rs` 的 handler、state owner、event stream 或 persistence。

---

## 选择理由

`backend.runtime.routes.run` 是 `backend.runtime.routes` 下最适合作为第一片继续推进的子叶:

1. 它只覆盖 run / v4 run / run record / replay / status 这一组 HTTP routes，边界比整个 runtime aggregate 更小。
2. 它已有 `api_run` 作为强等价证据，可以验证 route path、method、payload、response schema 和 record 行为不漂移。
3. 它仍可完全通过父链 `backend.interface_boundary -> backend.runtime -> backend.runtime.routes -> backend.runtime.routes.run -> src/runtime/run.rs` 通信。
4. 它不触碰 `/api/runtime/runs/:run_id/events`，event stream 保留给后续 `runtime.event_stream` 候选，避免一次批次混入 SSE envelope 与 replay cursor 风险。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001G 单子叶等价基线、runtime routes 局部递归 | 扩展 |
| 规范矩阵 | run route owner、state owner 冻结、event stream 排除 | 固化 |
| 引导矩阵 | `backend.runtime.routes.run` 白箱节点 | 扩展 |
| 模块树 | `backend.runtime.routes.run` | 建立单子叶基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 runtime、根7.6 v4.16 |
| 模块树节点 | `backend.runtime.routes.run` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime/run.rs`、`src/runtime_persistence.rs`、`src/runtime_event_projection.rs` |
| public 方法 | `backend.runtime.routes::register_routes`、`backend.runtime.routes.run::register_routes`、`start_test_run`、`start_v4_runtime_run`、`list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`、`get_run_replay`、`get_run_status` |
| route 坐标 | `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/runs`、`/api/runtime/runs/:run_id`、`/api/runtime/runs/:run_id/save`、`/api/runtime/runs/:run_id/replay`、`/api/runtime/runs/:run_id/status` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_sse`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | Axum Router | `backend.runtime.routes` | 不改变 run route path、method 或 Axum handler 类型 |
| 输入 | run HTTP request | frontend、tests、local API caller | 不改 payload、query、path param 或 error code |
| 输入 | AppState | `backend.app_state_wiring`、`src/app_runtime_helpers.rs` | 不迁移 AppState 字段 owner 或锁顺序 |
| 输出 | run routes | `backend.runtime.routes` 汇总后返回 `backend.runtime` | 只登记 run route group，不接管 event stream |
| 输出 | run response | frontend、tests | 不改 run record、status、replay 或 v4 run response schema |
| 输出 | persisted run projection | `src/runtime_persistence.rs` | 不改 persistence owner 或清理策略 |

---

## route owner 基线

| 接口域 | facade owner | 真实 owner | 代表入口 | 当前处理 |
| --- | --- | --- | --- | --- |
| test run | `backend.runtime.routes.run` | `src/runtime/run.rs` | `/api/runtime/test-run` | 只登记 route facade |
| v4 run | `backend.runtime.routes.run` | `src/runtime/run.rs`、runtime validation/evidence helpers | `/api/runtime/v4/run` | 不迁移 state owner |
| run list/detail | `backend.runtime.routes.run` | `src/runtime/run.rs`、`src/runtime_persistence.rs` | `/api/runtime/runs`、`/api/runtime/runs/:run_id` | 不改 response schema |
| run save/discard | `backend.runtime.routes.run` | `src/runtime/run.rs`、persistence helpers | `/api/runtime/runs/:run_id/save`、`DELETE /api/runtime/runs/:run_id` | 不改 storage semantics |
| run replay/status | `backend.runtime.routes.run` | `src/runtime/run.rs`、`src/runtime_event_projection.rs` | `/api/runtime/runs/:run_id/replay`、`/api/runtime/runs/:run_id/status` | 不改 replay/status payload |
| run event stream | `backend.runtime.routes` 保留 | `src/runtime/run.rs`、`src/runtime_event_projection.rs` | `/api/runtime/runs/:run_id/events` | 本批排除，后续单独建基线 |

---

## 兼容桥

本基线建立后的目标兼容桥为:

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> src/runtime/run.rs pub(crate) handler
  -> existing state owner / persistence
```

`backend.runtime.routes.run` 只能作为 route facade，不拥有 run record 真源、runtime state owner、AppState 字段 owner、event stream envelope 或 persistence 清理策略。

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | Rust 模块、Axum route handler 类型 | facade 委托链不断 |
| `cargo test -p quantpilot --test api_run` | run/v4 run/list/detail/save/replay/status/report | run routes 和 response schema 不漂移 |
| `cargo test -p quantpilot --test api_sse` | `/api/runtime/runs/:run_id/events` 排除边界 | event stream 仍保留在父 route aggregate 中且可用 |
| `tools/check-matrix-governance.ps1` | 单子叶文档和模块树锚点 | 治理入口不丢 |
| `tools/check-full-feature-tree.ps1` | 全量树路径覆盖 | 新文件和真实文件可定位 |

---

## 本批次不做

- 不移动 `src/runtime/run.rs` 的 handler 实现。
- 不迁移 runtime state owner、AppState 字段 owner、锁顺序或 persistence。
- 不接管 `/api/runtime/runs/:run_id/events` 的 event stream。
- 不改 `/api/runtime/test-run`、`/api/runtime/v4/run`、`/api/runtime/runs*` 的 path、method、payload、response schema 或 error code。
- 不拆 `runtime.backtest`、`runtime.event_stream`、`runtime.mutation_ai_proposal` 或 `runtime.report_experiment`。
- 不宣称 `backend.runtime.routes`、`backend.runtime` 或 `root.backend` 完成。

---

## 后续判断

`backend.runtime.routes.run` 完成本轮基线后，如果抽离通过，下一步需要对本子叶做 closeout 判断:

| 判断项 | 当前结论 |
| --- | --- |
| 是否需要继续拆 run handler | 暂不拆，先完成 facade 等价抽离 |
| 是否需要把 event stream 纳入 run | 不纳入，event stream 独立候选更清楚 |
| 是否需要整理 persistence owner | 不在本批，后续若整理必须另起方案 |

---

## 验收标准

1. `52-backend.runtime.routes.run单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `backend.runtime.routes.run` 白箱节点。
3. 全量树能定位 `src/backend/runtime/routes/run.rs` 和本基线。
4. 治理门禁能发现本文件缺失。
5. 后续 run route 抽离必须引用本基线，不得绕过父模块直接迁移 handler。
