# v4.16.0 runtime.run.session_start 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001I-03。  
> 基准: `58-runtime.run.session_start单子叶等价基线.md`、`59-runtime.run.session_start抽离记录.md`。  
> 判定: `runtime.run.session_start` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆。`stop_split: true`。后续应回到 `runtime.run` sibling 队列，优先对 `runtime.run.record_store` 或 `runtime.run.replay_status` 另起等价基线，而不是继续把 `start_test_run` 内部 helper 切成更小文件。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001I handler 层单叶 closeout | 收口 |
| 规范矩阵 | session start owner、run lock、state owner、父子通信 | 固化 |
| 引导矩阵 | `runtime.run.session_start` 白箱节点 | 收口 |
| 模块树 | `runtime.run.session_start` | closeout |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.session_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 与根7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.session_start` |
| 真实文件 | `src/runtime/run/session_start.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_validation.rs`、`src/runtime_event_projection.rs`、`src/runtime_response_mapping.rs`、`src/compile_api.rs`、`src/capability_api.rs`、`src/collaboration.rs`、`src/graph_quantscript_api.rs`、`src/frontend_runtime_mapping.rs` |
| public 方法 | `start_test_run`、`FrontendRunRequest`、`RunStartResponse`、`validate_runtime_capability_guard`、`validate_runtime_config_capabilities`、`compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config`、`build_compile_runtime_targets_from_graph`、`merge_runtime_targets`、`runtime_governance_snapshot`、`collect_frontend_events`、`prepend_capability_snapshot_event`、`attach_runtime_event_envelopes`、`validate_runtime_event_envelopes`、`account_summary`、`normalize_actor_identity`、`collaboration_with_run_actor`、`run_start_response` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `/api/runtime/test-run` 仍经 `backend.runtime.routes.run -> crate::runtime::start_test_run` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `run_session_start` 私有子模块与 `pub(crate)` re-export |
| handler 文件 | 已抽离 | `start_test_run` 已迁入 `src/runtime/run/session_start.rs` |
| request / response | 等价 | `FrontendRunRequest`、`RunStartResponse` 和 `run_start_response` schema 不变 |
| run lock | 等价 | `RunInProgressGuard` 与 `AppState.run_in_progress` owner 不变 |
| run record state | 等价 | `state.runs.insert(scoped run record)` 写入语义不变 |
| downstream sibling | 未迁移 | record store、replay/status、SSE、persistence 仍在原 owner |
| 回归证据 | 通过 | `api_run` 覆盖 missing capability、created run、target mapping、report/replay 代表链路 |

---

## 细分价值判断

**最终判定**: `runtime.run.session_start` 当前不继续细拆。它已经是一个围绕 legacy `/api/runtime/test-run` 的完整 orchestration 叶子，内部步骤虽然多，但都服务同一条 session start 事务链。

| 候选 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.run.session_start.capability_guard` | 不拆 | 真实 owner 在 `runtime_validation`，本叶只是调用者；拆出 wrapper 会增加父子桥而不减少复杂度 |
| `runtime.run.session_start.compile_path` | 不拆 | 真实 owner 在 `compile_api`，本叶只是发起 QS compile 和 runtime protocol compile |
| `runtime.run.session_start.session_execution` | 暂不拆 | sandbox start / run session 与事件收集、governance envelope、record 写入构成单次启动事务；拆开会制造过细连接 |
| `runtime.run.session_start.event_envelope` | 不拆 | 真实 owner 在 `runtime_event_projection`，本叶只编排调用顺序 |
| `runtime.run.session_start.record_write` | 不拆 | 本叶只写入 in-memory `state.runs`；持久化 record store 是后续 sibling，不属于本叶内部拆分 |

因此本叶 closeout 后，递归流程应回到 `runtime.run` sibling 队列，而不是继续切 `src/runtime/run/session_start.rs`。

---

## 父子通信收口

```text
backend.runtime.routes.run
  -> crate::runtime::start_test_run
  -> runtime::run_session_start::start_test_run
  -> validation / compile / event projection / response mapping helpers
  -> AppState.run_in_progress
  -> AppState.runs
```

本叶只允许通过父级 `runtime` re-export 和 `backend.runtime.routes.run` 暴露。不得横向直接接管 `runtime.run.v4_handoff`、`runtime.run.record_store`、`runtime.run.replay_status`、`runtime.event_stream`、backtest、mutation、executor 或 frontend state。

---

## 后续 sibling 队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.run.record_store` | 值得 | 覆盖 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`，需先建等价基线 |
| `runtime.run.replay_status` | 值得 | 覆盖 `get_run_replay`、`get_run_status`，可独立于 session start |
| `runtime.event_stream` | 值得但属父级 route 子叶 | SSE 仍由 `backend.runtime.routes` 直接拥有，不属于 `backend.runtime.routes.run` facade |
| `runtime.run.session_start` | 停止 | 本叶已 closeout，当前不继续细拆 |

默认下一候选: `runtime.run.record_store`。理由是它与刚完成的 session start 共享 `RunRecord`，但职责不同，适合另起基线承接 run lifecycle 的下一段。

---

## 本批次不做

- 不改 `/api/runtime/test-run` route path、method、payload、response schema 或 error code。
- 不迁移 `run_in_progress` owner、AppState 字段 owner、runtime state owner、lock order、`state.runs` owner 或 persistence。
- 不拆 `runtime.run.record_store`、`runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.v4_handoff` 或 backtest/mutation/report owner。
- 不清理旧中文字符串或历史注释编码问题。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.run.session_start` 已完成时，必须说明只完成 legacy `/api/runtime/test-run` handler 子模块抽离与 closeout；`src/runtime/run.rs` 仍拥有 record/replay/status/SSE sibling，`AppState` 仍拥有 run lock 和 `state.runs`，persistence owner 仍未迁移。不得宣称 runtime run handler 全部完成、record store 完成、SSE 完成或发布版本过渡启动。

---

## 验收标准

1. `60-runtime.run.session_start单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.run.session_start` closeout 完成并停止内部细分。
3. 全量树覆盖本 closeout 文档与 `src/runtime/run/session_start.rs`。
4. 治理门禁能发现本 closeout 文档缺失。
5. `api_run` 代表测试继续通过。
