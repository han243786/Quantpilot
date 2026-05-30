# v4.16.0 backend.runtime.routes.event_stream 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BW-01  
> 基准: `240-backend.runtime.routes第四轮父叶残余判断.md`、`73-runtime.event_stream单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `backend.runtime.routes.event_stream` 单子叶等价基线已建立。当前 `no code movement`，只冻结 event stream route facade 的 path、method、handler owner、父级注册位置、SSE frame contract、keepalive 语义和回归证据。下一步只能进入 BE-001BW-02 抽离方案，不得创建 `src/backend/runtime/routes/event_stream.rs` 或迁移 `stream_run_events` handler。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BW-01 event stream route facade 单子叶基线 | 新建基线 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.event_stream` | 新增单子叶基线 |
| 模块树 | `backend.runtime.routes.event_stream` | `stop_split: pending` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.event_stream` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes.rs`、`src/runtime/event_stream.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.event_stream` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/event_stream.rs`、`src/runtime/mod.rs`、`src/runtime/run/record_store.rs`、`tests/api_sse.rs`、`tests/api_run.rs` |
| public 方法 | `backend.runtime.routes::register_routes`、`runtime_handlers::stream_run_events`、`stream_run_events` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、`git diff --check` |

---

## 当前 route owner 基线

| route | method | handler | 当前 route owner | 当前 handler owner |
| --- | --- | --- | --- | --- |
| `/api/runtime/runs/:run_id/events` | GET | `runtime_handlers::stream_run_events` | `src/backend/runtime/routes.rs` | `src/runtime/event_stream.rs` |

父级注册片段:

```rust
let router = router.route(
    "/api/runtime/runs/:run_id/events",
    get(runtime_handlers::stream_run_events),
);
```

父级 route order 仍为:

```text
run -> event_stream -> evidence -> mutation -> report_ops -> experiment -> ops
```

---

## Handler 等价边界

| handler | 输入 | 输出 | 关键依赖 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `stream_run_events` | `auth::UserId`、`State<AppState>`、`Path(run_id)` | `Sse<impl Stream<Item = Result<Event, Infallible>>>` | `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、`KeepAlive` | 不得迁移 handler、改变 SSE frame、改变 keepalive 或绕过 run record lookup |

SSE frame contract 固定为:

| 顺序 | event | data contract |
| --- | --- | --- |
| 1 | `run_started` | `run_id`、`graph_id`、`compile_id`、`status: started` |
| 2..n | `runtime_event` | run record events 原样转成 JSON SSE payload |
| n+1 | `account` | run record account snapshot |
| n+2 | `run_completed` | `run_id`、`status: completed`、`event_count` |

Keepalive contract 固定为 `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")`。

---

## 保留边界

BE-001BW-01 不迁移、不修改:

- `src/backend/runtime/routes.rs` 中任何 route registration。
- planned `src/backend/runtime/routes/event_stream.rs`。
- `stream_run_events` handler。
- `load_run_record_from_state`。
- `json_sse_event`。
- `SSE_EVENT_DELAY_MS`。
- SSE event name、payload shape、frame order、delay 或 keepalive。
- `auth::UserId`、`State<AppState>`、`Path(run_id)` extractor 组合。
- `AppState`。
- schema owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。
- report_ops route group。

---

## 父子通信规则

当前固定:

```text
backend.runtime
  -> backend.runtime.routes
  -> crate::runtime::stream_run_events
  -> runtime.event_stream handler body
```

BE-001BW-01 只登记计划中的 route child 坐标。下一步若进入抽离方案，也只能规划 route registration facade，不得迁移 event stream handler、run record state owner、schema owner、AppState、frontend caller、runtime persistence owner 或发布过渡连接。

---

## 回归证据

| 证据 | 覆盖 |
| --- | --- |
| `cargo test -p quantpilot --test api_sse` | `/api/runtime/runs/:run_id/events` content-type、SSE event 顺序、runtime_event payload、run_completed event_count |
| `cargo test -p quantpilot --test api_run` | runtime run record 创建、读取与事件侧效应兼容 |
| `cargo test --no-run` | 编译所有 Rust 测试目标 |
| `tools/check-matrix-governance.ps1` | 模块树 / 里程碑 / gate token 覆盖 |
| `tools/check-full-feature-tree.ps1` | 全量树路径覆盖 |

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BW-02 backend.runtime.routes.event_stream 抽离方案
```

该方案只允许规划 `src/backend/runtime/routes/event_stream.rs` route facade 与父级 `event_stream::register_routes(router)` 委托；不得直接创建目标文件、不得迁移 handler、不得改变 `AppState` / schema owner / frontend caller / runtime persistence owner / release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BW-01 完成时，必须说明当前只是 `backend.runtime.routes.event_stream` 等价基线，`src/backend/runtime/routes/event_stream.rs` 尚未创建，route registration 仍在 `src/backend/runtime/routes.rs`，handler 仍在 `src/runtime/event_stream.rs`。不得宣称 event stream route 已迁移、SSE handler 已迁移、run record lookup 已迁移、report_ops 已处理、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `241-backend.runtime.routes.event_stream单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `backend.runtime.routes.event_stream` 白箱节点，并标记 `stop_split: pending`。
3. 基线明确 event stream route path、method、handler owner、SSE frame contract、keepalive contract 和非目标边界。
4. 下一步固定为 BE-001BW-02 抽离方案。
5. 本批保持 `no code movement`。
