# v4.16.0 backend.runtime.routes.event_stream 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BW-02  
> 基准: `241-backend.runtime.routes.event_stream单子叶等价基线.md`、`240-backend.runtime.routes第四轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes.event_stream` 抽离方案已建立。当前 `no code movement`，只规划 route facade 迁移；下一步 BE-001BW-03 才允许创建 `src/backend/runtime/routes/event_stream.rs` 并迁移 `/api/runtime/runs/:run_id/events` GET route registration。handler、SSE frame contract、keepalive contract、schema owner、`AppState`、frontend caller、runtime persistence owner 和 release transition guard 均不得迁移。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BW-02 event stream route facade 抽离方案 | 新建方案 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.event_stream` | 更新下一步 |
| 模块树 | `backend.runtime.routes.event_stream` | `stop_split: pending` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.event_stream` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes.rs`、`src/runtime/event_stream.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.event_stream` |
| 当前真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/event_stream.rs`、`src/runtime/mod.rs`、`src/runtime/run/record_store.rs`、`tests/api_sse.rs`、`tests/api_run.rs` |
| 计划目标文件 | `src/backend/runtime/routes/event_stream.rs` |
| public 方法 | `backend.runtime.routes::register_routes`、`event_stream::register_routes(router)`、`runtime_handlers::stream_run_events`、`stream_run_events` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 目标形态

BE-001BW-03 的唯一目标是把 event stream route registration 从父 aggregate 移入 route child facade:

```text
src/backend/runtime/routes.rs
  pub mod event_stream;
  register_routes:
    backtest -> run -> event_stream -> evidence -> mutation -> report_ops -> experiment -> ops

src/backend/runtime/routes/event_stream.rs
  pub const MODULE_ID: &str = "backend.runtime.routes.event_stream";
  pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>
```

必须保持 route order:

```text
run -> event_stream -> evidence -> mutation
```

允许迁移的 route registration 仅限:

| route | method | handler |
| --- | --- | --- |
| `/api/runtime/runs/:run_id/events` | GET | `runtime_handlers::stream_run_events` |

---

## 允许迁移清单

BE-001BW-03 只能执行以下代码动作:

1. 创建 `src/backend/runtime/routes/event_stream.rs`。
2. 在 `src/backend/runtime/routes.rs` 中声明 `pub mod event_stream`。
3. 把 `/api/runtime/runs/:run_id/events` GET route registration 移入 `event_stream::register_routes(router)`。
4. 在父级中以 `event_stream::register_routes(router)` 委托，并保持 `run -> event_stream -> evidence -> mutation` 的相对顺序。
5. 保留父级对 report_ops / ops routes 仍需使用的 `get` import；不得为本批做无关 import 风格调整。

---

## 非目标边界

本方案和下一步实际抽离均不得迁移或修改:

- `stream_run_events` handler body。
- `load_run_record_from_state`。
- `json_sse_event`。
- `SSE_EVENT_DELAY_MS`。
- `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")`。
- SSE event name、payload shape、frame order、delay 或 keepalive。
- `auth::UserId`、`State<AppState>`、`Path(run_id)` extractor 组合。
- run record store、state owner 或锁顺序。
- `AppState`。
- schema owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。
- report_ops route group。

---

## 父子通信规则

固定为:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.event_stream
  -> crate::runtime::stream_run_events
```

`backend.runtime.routes.event_stream` 只能作为 route facade。不得横向接管 `backend.runtime.routes.report_ops`、evidence、runtime report generation、frontend caller、runtime persistence owner 或 executor。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接或性能旁路。

---

## 回退点

若 BE-001BW-03 验证失败，回退只允许:

1. 删除 `src/backend/runtime/routes/event_stream.rs`。
2. 移除父级 `pub mod event_stream`。
3. 把 `/api/runtime/runs/:run_id/events` GET route registration 放回 `src/backend/runtime/routes.rs` 的 run 与 evidence 之间。

不得用回退作为理由迁移 handler、SSE contract、schema、`AppState`、frontend caller 或 runtime persistence owner。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_sse
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BW-03 backend.runtime.routes.event_stream 实际抽离
```

BE-001BW-03 完成后必须进入单叶 closeout，判断 `backend.runtime.routes.event_stream` 是否还值得继续细拆。不得跳过 closeout 直接处理 report_ops。

---

## 幻觉检查点

AI 声称 BE-001BW-02 完成时，必须说明当前仍是 `no code movement` 的抽离方案，`src/backend/runtime/routes/event_stream.rs` 尚未创建，handler、SSE frame contract、keepalive contract 与 state/persistence owner 均未改变。不得宣称 event stream route 已迁移、report_ops 已处理、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `242-backend.runtime.routes.event_stream抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树 `backend.runtime.routes.event_stream` 节点更新为 BE-001BW-02 抽离方案已建立。
3. 方案明确 BE-001BW-03 的目标文件、父级委托、route order、允许迁移清单和非目标边界。
4. 下一步固定为 BE-001BW-03 实际抽离。
5. 本批保持 `no code movement`。
