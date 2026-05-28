# v4.16.0 runtime.event_stream 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001L-01。  
> 基准: `69-runtime.run.replay_status单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.event_stream` 单子叶等价基线，不移动代码，`no code movement`，不迁移 SSE handler、route facade、state owner、record lookup、event envelope 或 frontend caller。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | R5 父级 runtime route sibling 队列从 `runtime.run.replay_status` closeout 转向 `runtime.event_stream` | 推进 |
| 规范矩阵 | SSE route 生命周期、keep-alive、event frame、父级 route owner 和 shared helper 保留 | 冻结 |
| 引导矩阵 | `runtime.event_stream` 白箱节点 | 新增基线 |
| 模块树 | `runtime.event_stream` | 新增 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.event_stream` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.event_stream` |
| 当前真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/run.rs`、`src/runtime/mod.rs`、`tests/api_sse.rs` |
| public 方法 | `stream_run_events` |
| 保留 shared helper | `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、`KeepAlive::new`、`sleep` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 当前白箱边界

| 项 | 当前 owner | 说明 |
| --- | --- | --- |
| route | `src/backend/runtime/routes.rs` | `GET /api/runtime/runs/:run_id/events` 由父级 runtime route aggregate 直接注册 |
| handler | `src/runtime/run.rs` | `stream_run_events` 读取 run record 并返回 `Sse<impl Stream<Item = Result<Event, Infallible>>>` |
| record lookup | `src/runtime/run.rs` shared helper | `load_run_record_from_state` 继续处理 scoped in-memory 优先与 manifest fallback |
| frame builder | runtime shared helper | `json_sse_event` 继续生成 `run_started`、`runtime_event`、`account`、`run_completed` frames |
| stream timing | `src/runtime/run.rs` | 每个 runtime event 之间保留 `sleep(Duration::from_millis(SSE_EVENT_DELAY_MS))` |
| keep alive | Axum SSE | `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")` 保持不变 |
| tests | `tests/api_sse.rs` | 验证 content-type、四类 event frame、run id、runtime event envelope 与 completed event count |

---

## 输入输出基线

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `UserId` | auth middleware | scoped user id | 仅用于 scoped run lookup，不迁移 auth owner |
| `AppState` | `backend.app_state_wiring` | shared app state | 只使用既有 `runs`、`run_store_dir` 等字段 |
| `run_id` | path param | string | lookup 语义必须与 detail/replay/status 一致 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| `run_started` frame | frontend SSE panel、tests | SSE event + JSON data | 必须包含 `run_id`、`graph_id`、`compile_id`、`status` |
| `runtime_event` frames | frontend SSE panel、tests | SSE event + runtime event JSON | 必须保持原事件顺序和 envelope |
| `account` frame | frontend SSE panel、tests | SSE event + account JSON | 必须保持 account summary payload |
| `run_completed` frame | frontend SSE panel、tests | SSE event + JSON data | 必须包含 `run_id`、`status`、`event_count` |
| keep-alive | frontend SSE client | SSE keepalive comment/text | 5 秒 interval 和 `keepalive` 文本保持不变 |

---

## 关键 public 方法

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `stream_run_events` | `UserId`、`AppState`、`run_id` | Axum `Sse` stream | `GET /api/runtime/runs/:run_id/events` | 不得混入 replay/status、record store、mutation 或 frontend state |
| `load_run_record_from_state` | `AppState`、`UserId`、`run_id` | `RunRecord` | `stream_run_events` | 不得改变 current runtime 优先与 manifest fallback |
| `json_sse_event` | event name、JSON payload | Axum `Event` | `stream_run_events` | 不得改变 frame envelope 或 event name |
| `KeepAlive::new` | interval/text | SSE keep-alive | Axum SSE | 不得改变 keepalive interval 或文本 |

---

## 明确排除

- 不迁移 `stream_run_events` 或 `/api/runtime/runs/:run_id/events`。
- 不把 `runtime.event_stream` 放入 `backend.runtime.routes.run` 或 `runtime.run.replay_status` 内部。
- 不迁移 `get_run_replay`、`get_run_status`、record store、session start、v4 handoff 或 backtest handler。
- 不迁移 `load_run_record_from_state`、`state.runs`、`run_store_dir`、AppState owner 或 persistence owner。
- 不改变 SSE event name、frame order、payload schema、keep-alive interval 或 delay。
- 不修改 frontend SSE caller、UI、store 或 test asset。
- 不主动提出发布版本过渡或横向连接。

---

## 适配性风险与暂停条件

| 风险 | 处理 |
| --- | --- |
| SSE route 被误放入 run route facade | 中止；本 route 当前属于 `backend.runtime.routes` 父级 aggregate，不属于 `backend.runtime.routes.run` |
| replay/status 与 event stream 混淆 | 中止；replay/status 已 closeout，SSE 必须另起父级子叶 |
| `load_run_record_from_state` 可见性或 owner 被改变 | 中止；shared record lookup 不属于本基线迁移目标 |
| frame order 或 event name 被改 | 中止；`api_sse` 是本基线的代表证据 |
| keep-alive、delay 或 stream type 需要调整 | 中止并另起方案；这会改变 SSE 生命周期语义 |

---

## 下一步

本基线通过后，下一批若继续，应进入 `BE-001L-02 runtime.event_stream 抽离方案`。方案只能讨论是否把 `stream_run_events` 迁入计划目标文件，例如 `src/runtime/event_stream.rs` 或 `src/runtime/run/event_stream.rs`，并必须先确认父级 route owner；不得直接移动代码，不得混入 replay/status、record store、response mapping、schema、state owner 或 frontend route。

---

## 幻觉检查点

AI 声称 `runtime.event_stream` 已建立基线时，必须说明这只是 SSE route 的等价基线；当前没有迁移代码，没有迁移 `stream_run_events`，没有改变 event frame、keep-alive、record lookup、state owner、persistence owner 或 frontend caller。不得宣称 runtime run handler 全部完成，也不得把 SSE 说成 `runtime.run.replay_status` 的一部分。

---

## 验收标准

1. `70-runtime.event_stream单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树新增 `runtime.event_stream` 白箱节点。
3. 全量树覆盖本基线文档和真实文件。
4. 治理门禁能发现本基线文档缺失。
5. `api_sse` 继续证明 SSE route、frame order、event envelope 和 completed count 等价。
