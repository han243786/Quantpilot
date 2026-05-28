# v4.16.0 runtime.event_stream 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001L-04。  
> 基准: `70-runtime.event_stream单子叶等价基线.md`、`71-runtime.event_stream抽离方案.md`、`72-runtime.event_stream抽离记录.md`。  
> 判定: `runtime.event_stream` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆，`stop_split: true`。后续应回到父级 `backend.runtime.routes` sibling 队列，默认下一候选为 `runtime.backtest`，不能把 backtest、mutation、report、shared helper、state owner、persistence owner 或 frontend caller 混入本叶。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001L event_stream 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、route owner 保留、SSE lifecycle 保留、细分停止条件 | 固化 |
| 引导矩阵 | `runtime.event_stream` 白箱节点 | closeout |
| 模块树 | `runtime.event_stream` | closeout |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.event_stream` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.event_stream` |
| 真实文件 | `src/runtime/event_stream.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/backend/runtime/routes.rs`、`tests/api_sse.rs` |
| public 方法 | `stream_run_events` |
| 保留 shared helper | `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、`KeepAlive::new`、`sleep` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test -p quantpilot --test api_sse`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `GET /api/runtime/runs/:run_id/events` 仍经 `backend.runtime.routes -> crate::runtime::stream_run_events` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `event_stream` 私有子模块与 `pub(crate)` re-export |
| handler 文件 | 已抽离 | `stream_run_events` 已迁入 `src/runtime/event_stream.rs` |
| record lookup | 等价 | 仍由 `load_run_record_from_state` 处理 scoped in-memory 优先与 manifest fallback |
| frame order | 等价 | `run_started`、`runtime_event`、`account`、`run_completed` 顺序不变 |
| delay | 等价 | 每个 runtime event 后仍使用 `sleep(Duration::from_millis(SSE_EVENT_DELAY_MS))` |
| keep-alive | 等价 | 仍使用 `KeepAlive::new().interval(Duration::from_secs(5)).text("keepalive")` |
| sibling 边界 | 保留 | replay/status、record store、session start、v4 handoff、backtest、mutation、report 均不属于本叶 |
| shared owner | 保留 | AppState、persistence、event projection、frontend caller 均不私有化到本叶 |

---

## 细分价值判断

**最终判定**: `runtime.event_stream` 当前不继续细拆。它已经是围绕 run SSE route 的完整 handler 叶子；继续细拆会把 frame builder、record lookup、delay/keep-alive 或 frontend caller 包成更小 facade，但会制造 shared owner 纠缠。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.event_stream.frame_order` | 不拆 | frame order 是 `stream_run_events` 的核心行为，拆出会降低可读性并引入无收益 facade |
| `runtime.event_stream.record_lookup` | 不拆 | `load_run_record_from_state` 同时服务 detail/replay/status/mutation/report 等路径，是共享 helper owner |
| `runtime.event_stream.json_event_builder` | 不拆 | `json_sse_event` 是通用 SSE event helper，不能私有化到本叶 |
| `runtime.event_stream.keep_alive` | 不拆 | keep-alive 当前只是 Axum SSE 配置，继续拆会包装库调用 |
| `runtime.event_stream.delay_policy` | 不拆 | `SSE_EVENT_DELAY_MS` 是全局 SSE pacing 常量，不在本叶内独立成模块 |
| `runtime.event_stream.frontend_caller` | 不拆 | frontend SSE caller 不在后端抽离本批内，后续需前端抽离方案 |
| `runtime.backtest` | 不在本叶内拆 | backtest 是父级 `backend.runtime.routes` sibling 候选，不能混入 event stream closeout |

因此本叶 closeout 后，递归流程应退出 `runtime.event_stream` 内部，回到父级 sibling 队列。

---

## 父子通信收口

```text
backend.runtime.routes
  -> crate::runtime::stream_run_events
  -> runtime::event_stream::stream_run_events
  -> load_run_record_from_state
  -> json_sse_event / Sse / KeepAlive / sleep
  -> frontend SSE client
```

本叶只能经父级 `runtime` re-export 和 `backend.runtime.routes` 暴露 event stream route，不得横向接管 `backend.runtime.routes.run`、`runtime.run.replay_status`、record store、backtest、mutation、executor、report 或 frontend state。

---

## 后续 sibling 队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest` | 默认下一候选 | 需回到父级 `backend.runtime.routes` 建立单子叶等价基线；不得从 `runtime.event_stream` 内部直接迁移 |
| `runtime.mutation_ai_proposal` | 后续候选 | 涉及 mutation、AI proposal、approval，需另起边界基线 |
| `runtime.report_experiment` | 后续候选 | 涉及 evidence report、experiment 与 ops reports，需另起边界基线 |
| `runtime.event_stream` | 停止 | 本叶已 closeout，当前不继续细拆 |
| `backend.runtime.routes.run` | 停止 | 已在 BE-001G-03 closeout |

---

## 本批次不做

- 不迁移 `src/backend/runtime/routes.rs` 中的 route registration。
- 不把 event stream 放入 `backend.runtime.routes.run` 或 `src/backend/runtime/routes/run.rs`。
- 不迁移 `runtime.backtest`、`runtime.mutation_ai_proposal`、`runtime.report_experiment` 或 `runtime.run.*` sibling。
- 不迁移 `load_run_record_from_state`、`json_sse_event`、`SSE_EVENT_DELAY_MS`、AppState owner、state lock 或 persistence owner。
- 不改变 `run_started`、`runtime_event`、`account`、`run_completed` 的 event name、frame order、payload schema、delay 或 completed event count。
- 不迁移 frontend SSE caller、UI、store 或 test asset。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.event_stream` 已完成时，必须说明只完成 run SSE handler 的抽离与单叶 closeout；route facade、shared helper、state owner、persistence owner、frontend caller、backtest、mutation、report 和发布版本过渡均未完成。不得宣称 runtime route aggregate 全部完成，也不得把 `runtime.backtest` 说成本叶的一部分。

---

## 验收标准

1. `73-runtime.event_stream单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.event_stream` closeout 完成并停止内部细分。
3. 全量树覆盖本 closeout 文档与 `src/runtime/event_stream.rs`。
4. 治理门禁能发现本 closeout 文档缺失。
5. `api_sse` 代表测试继续证明 SSE frame order、event envelope、keep-alive 和 completed event count 等价。
