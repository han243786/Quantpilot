# v4.16.0 runtime.event_stream 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001L-02。  
> 基准: `70-runtime.event_stream单子叶等价基线.md`、`13-递归模块化全局根流程.md`。  
> 判定: 本批只建立 `runtime.event_stream` 抽离方案，`no code movement`。下一批若实施，只允许移动 `stream_run_events`，不得迁移 route facade、record lookup、state owner、persistence owner、event projection、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | R5 父级 runtime route sibling 队列: `runtime.event_stream` 从等价基线进入抽离方案 | 推进 |
| 规范矩阵 | SSE handler 最小移动、父级 route owner、shared helper owner、keep-alive 和 frame order | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.event_stream` | 抽离方案 |
| 模块树 | `runtime.event_stream` 白箱节点 | 补方案状态 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.event_stream` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.event_stream` |
| 当前真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/run.rs`、`src/runtime/mod.rs`、`tests/api_sse.rs` |
| 下一批计划目标文件 | `src/runtime/event_stream.rs` |
| 当前 public 方法 | `stream_run_events` |
| 保留 shared helper | `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、`KeepAlive::new`、`sleep` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 最小抽离方案

| 项 | 方案 |
| --- | --- |
| 目标 handler | 下一批只把 `stream_run_events` 从 `src/runtime/run.rs` 迁入 `src/runtime/event_stream.rs` |
| 父级出口 | `src/runtime/mod.rs` 新增私有模块声明并继续 `pub(crate) use event_stream::stream_run_events` |
| route facade | `src/backend/runtime/routes.rs` 保持注册 `GET /api/runtime/runs/:run_id/events`，仍调用 `runtime_handlers::stream_run_events` |
| run route facade | `backend.runtime.routes.run` 不接管 event stream，`src/backend/runtime/routes/run.rs` 不变 |
| shared helper | `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS` 继续保留当前 owner |
| SSE 生命周期 | `run_started`、`runtime_event`、`account`、`run_completed` frame order、delay 和 keep-alive 不变 |
| 状态/持久化 | `state.runs`、`run_store_dir`、persistence owner、event projection owner 和 frontend caller 不迁移 |

下一批的 `src/runtime/mod.rs` 目标形态应保持父级兼容出口:

```rust
#[path = "event_stream.rs"]
mod event_stream;
pub(crate) use event_stream::stream_run_events;
```

---

## 明确排除

- 不迁移 `src/backend/runtime/routes.rs` 中的 route registration。
- 不把 `GET /api/runtime/runs/:run_id/events` 放入 `backend.runtime.routes.run`。
- 不迁移 `get_run_replay`、`get_run_status`、`runtime.run.replay_status`、record store、session start、v4 handoff、backtest、mutation 或 report experiment。
- 不迁移 `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、AppState owner、state lock、persistence owner 或 frontend caller。
- 不改变 `run_started`、`runtime_event`、`account`、`run_completed` 的 event name、frame order、payload schema、delay 或 completed event count。
- 不改变 `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")` 的 interval 或文本。
- 不主动提出发布版本过渡，不新增子模块横向连接。

---

## 适配性风险与暂停条件

| 风险 | 暂停规则 |
| --- | --- |
| route owner 冲突 | 若必须把 SSE route 放入 `backend.runtime.routes.run`，中止并回到方案讨论 |
| shared helper 牵连 | 若移动 `stream_run_events` 必须同时迁移 `load_run_record_from_state` 或 `json_sse_event`，中止并另起 helper owner 方案 |
| SSE 语义漂移 | 若 frame order、event name、payload schema、delay 或 keep-alive 需要调整，中止并另起 SSE 生命周期方案 |
| state/persistence 牵连 | 若需要改 `state.runs`、`run_store_dir`、manifest fallback 或锁顺序，中止并另起 state owner 方案 |
| 回归失败 | 若 `cargo test -p quantpilot --test api_sse` 或 `cargo test -p quantpilot --test api_run` 失败，中止并先修复等价缺口 |

---

## 下一步

下一批进入 `BE-001L-03 runtime.event_stream 抽离记录`。实施范围只能是:

1. 新建 `src/runtime/event_stream.rs`。
2. 将 `stream_run_events` 迁入该文件。
3. 在 `src/runtime/mod.rs` 保留 `pub(crate)` 兼容出口。
4. 从 `src/runtime/run.rs` 删除对应 handler 实现。
5. 运行 `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run` 和治理门禁。

---

## 幻觉检查点

AI 声称 `runtime.event_stream` 已有抽离方案时，必须说明本批没有迁移代码，只允许下一批最小移动 `stream_run_events`。不得宣称 `src/runtime/event_stream.rs` 已存在，不得宣称 SSE 已抽离完成，不得宣称 runtime run handler 全部完成，也不得把本叶说成 `runtime.run.replay_status` 或 `backend.runtime.routes.run` 的内部子叶。

---

## 验收标准

1. `71-runtime.event_stream抽离方案.md` 进入 v4.16 里程碑索引、全量树和模块树。
2. 治理门禁能发现本方案缺失。
3. 方案明确下一批只迁移 `stream_run_events` 到 `src/runtime/event_stream.rs`。
4. 方案明确 `src/backend/runtime/routes.rs`、`src/runtime/run.rs`、`src/runtime/mod.rs` 和 `tests/api_sse.rs` 的当前等价证据。
5. 方案明确 `GET /api/runtime/runs/:run_id/events`、`run_started`、`runtime_event`、`account`、`run_completed`、`KeepAlive::new` 和 `keepalive` 均保持等价。
6. 本批不发生代码移动。
