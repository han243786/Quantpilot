# v4.16.0 runtime.event_stream 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001L-03。  
> 基准: `70-runtime.event_stream单子叶等价基线.md`、`71-runtime.event_stream抽离方案.md`。  
> 判定: 按方案完成 `runtime.event_stream` 第一轮实际抽离；只移动 `stream_run_events`，不迁移 route facade、record lookup、state owner、persistence owner、event projection、frontend caller 或发布版本过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001L event_stream 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、最小迁移、route owner 与 shared helper 保留 | 落地 |
| 引导矩阵 | `runtime.event_stream` 白箱节点 | 更新 |
| 模块树 | `runtime.event_stream` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.event_stream` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.event_stream` |
| 新真实文件 | `src/runtime/event_stream.rs` |
| 保留真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`tests/api_sse.rs` |
| public 方法 | `stream_run_events` |
| 保留 shared helper | `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、`KeepAlive::new`、`sleep` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 event_stream 子模块 | `src/runtime/event_stream.rs` | 承载 `stream_run_events` |
| 删除旧位置 handler | `src/runtime/run.rs` | SSE handler 已移出；后续 legacy runtime blocks 保留原位 |
| 父级兼容出口 | `src/runtime/mod.rs` | 增加 `event_stream` 私有子模块和 `pub(crate) use event_stream::stream_run_events` |
| route facade | `src/backend/runtime/routes.rs` | 未改动，仍注册 `GET /api/runtime/runs/:run_id/events` 并调用 `runtime_handlers::stream_run_events` |

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| route path | `GET /api/runtime/runs/:run_id/events` 仍由 `src/backend/runtime/routes.rs` 父级 aggregate 直接注册 |
| run lookup | `stream_run_events` 仍通过 `load_run_record_from_state` 做 scoped run lookup |
| start frame | `run_started` frame 仍输出 `run_id`、`graph_id`、`compile_id`、`status` |
| event frames | `runtime_event` frames 仍按 record events 顺序输出 |
| delay | 每个 runtime event 后仍使用 `sleep(Duration::from_millis(SSE_EVENT_DELAY_MS))` |
| account frame | `account` frame 仍输出 `record.account` |
| completed frame | `run_completed` frame 仍输出 `run_id`、`status`、`event_count` |
| keep alive | `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")` 不变 |

---

## 明确未迁移

- 不迁移 `src/backend/runtime/routes.rs` 中的 route registration。
- 不把 event stream 放入 `backend.runtime.routes.run` 或 `src/backend/runtime/routes/run.rs`。
- 不迁移 `runtime.run.replay_status`、record store、session start、v4 handoff、backtest、mutation 或 report experiment。
- 不迁移 `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、AppState owner、state lock 或 persistence owner。
- 不改变 `run_started`、`runtime_event`、`account`、`run_completed` 的 event name、frame order、payload schema、delay 或 completed event count。
- 不迁移 frontend SSE caller、UI、store 或 test asset。
- 不启动发布版本过渡，不新增横向连接。

---

## 回退点

若后续发现行为回归，可将 `stream_run_events` 从 `src/runtime/event_stream.rs` 放回 `src/runtime/run.rs`，并移除 `src/runtime/mod.rs` 中的 `event_stream` 私有模块与 re-export。`src/backend/runtime/routes.rs` 不需要回退，因为本批没有修改 route facade。

---

## 验证计划

本批收口必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_sse
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 `runtime.event_stream` 单叶整理 / closeout，确认 SSE handler 抽离后与原功能等价，并判断本叶内部是否值得继续细拆。当前默认不继续拆 `load_run_record_from_state`、`json_sse_event`、SSE lifecycle helper、state owner、persistence owner 或 frontend caller；如要拆这些 shared owner，必须另起父级共享节点方案。

---

## 验收标准

1. `72-runtime.event_stream抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/event_stream.rs` 进入全量树和模块树。
3. `src/runtime/mod.rs` 保留 `crate::runtime::stream_run_events` 兼容出口。
4. `src/backend/runtime/routes.rs` route path/method 不变。
5. 治理门禁能发现本抽离记录缺失。
6. `api_sse` 证明 SSE frame order、event envelope、keep-alive 和 completed event count 仍可通过。
