# v4.16.0 backend.runtime.routes.event_stream 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BW-03  
> 基准: `242-backend.runtime.routes.event_stream抽离方案.md`、`241-backend.runtime.routes.event_stream单子叶等价基线.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes.event_stream` route facade 实际抽离已完成。`src/backend/runtime/routes/event_stream.rs` 已创建并承接 `/api/runtime/runs/:run_id/events` GET route registration；父级通过 `event_stream::register_routes(router)` 委托，并保持 `run -> event_stream -> evidence -> mutation` 顺序。handler、SSE frame contract、keepalive contract、schema owner、`AppState`、frontend caller、runtime persistence owner 和 release transition guard 均未迁移。  
> 代码动作: route facade extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BW-03 event stream route facade 实际抽离 | 实际抽离 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 执行 |
| 引导矩阵 | `root.backend.runtime.routes.event_stream` | 更新实际文件 |
| 模块树 | `backend.runtime.routes.event_stream` | `stop_split: pending` |

---

## 代码变更

| 文件 | 动作 | 说明 |
| --- | --- | --- |
| `src/backend/runtime/routes/event_stream.rs` | 新增 | 新增 route child facade，提供 `MODULE_ID` 与 `register_routes` |
| `src/backend/runtime/routes.rs` | 更新 | 声明 `pub mod event_stream`，并在 run 与 evidence 之间委托 `event_stream::register_routes(router)` |
| `src/runtime/event_stream.rs` | 保持不变 | `stream_run_events` handler owner 未迁移 |

---

## 抽离后结构

```text
src/backend/runtime/routes.rs
  pub mod event_stream;
  register_routes:
    backtest -> run -> event_stream -> evidence -> mutation -> report_ops -> experiment -> ops

src/backend/runtime/routes/event_stream.rs
  pub const MODULE_ID: &str = "backend.runtime.routes.event_stream";
  pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>
```

实际迁移的 route registration:

| route | method | handler | 当前 owner |
| --- | --- | --- | --- |
| `/api/runtime/runs/:run_id/events` | GET | `runtime_handlers::stream_run_events` | `src/backend/runtime/routes/event_stream.rs` |

---

## 等价边界

本批只迁移 route registration，不迁移:

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

抽离后固定为:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.event_stream
  -> crate::runtime::stream_run_events
```

`backend.runtime.routes.event_stream` 只作为 route facade。不得横向接管 `backend.runtime.routes.report_ops`、evidence、runtime report generation、frontend caller、runtime persistence owner 或 executor。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接或性能旁路。

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
BE-001BW-04 backend.runtime.routes.event_stream 单叶 closeout
```

BE-001BW-04 必须判断 `backend.runtime.routes.event_stream` 是否值得继续细拆。不得跳过 closeout 直接处理 report_ops。

---

## 幻觉检查点

AI 声称 BE-001BW-03 完成时，必须说明只完成 route facade 抽离；`stream_run_events` handler、SSE frame contract、keepalive contract、AppState、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。不得宣称 report_ops 已处理、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `src/backend/runtime/routes/event_stream.rs` 已创建并承接 event stream route registration。
2. `src/backend/runtime/routes.rs` 通过 `event_stream::register_routes(router)` 委托，并保持 `run -> event_stream -> evidence -> mutation` 顺序。
3. `243-backend.runtime.routes.event_stream抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 下一步固定为 BE-001BW-04 单叶 closeout。
5. 本批不迁移 handler、SSE contract、`AppState`、schema owner、frontend caller、runtime persistence owner 或 release transition guard。
