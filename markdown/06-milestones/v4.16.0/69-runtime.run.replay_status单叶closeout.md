# v4.16.0 runtime.run.replay_status 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001K-04。  
> 基准: `66-runtime.run.replay_status单子叶等价基线.md`、`67-runtime.run.replay_status抽离方案.md`、`68-runtime.run.replay_status抽离记录.md`。  
> 判定: `runtime.run.replay_status` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆，`stop_split: true`。后续应回到父级 `backend.runtime.routes` sibling 队列，默认下一候选为 `runtime.event_stream`，不能把 SSE 混入本叶。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001K replay_status 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、shared owner 保留、细分停止条件 | 固化 |
| 引导矩阵 | `runtime.run.replay_status` 白箱节点 | closeout |
| 模块树 | `runtime.run.replay_status` | closeout |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.replay_status` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.replay_status` |
| 真实文件 | `src/runtime/run/replay_status.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public 方法 | `get_run_replay`、`get_run_status` |
| 保留 shared helper | `load_run_record_from_state`、`normalized_replay_options`、`run_replay_response_from_record`、`run_status_response_from_record`、`json_bad_request`、`RuntimeEvidenceMetrics::record_replay_page` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_evidence_contract`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `GET /api/runtime/runs/:run_id/replay` 与 `GET /api/runtime/runs/:run_id/status` 仍经 `backend.runtime.routes.run -> crate::runtime::*` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `run_replay_status` 私有子模块与 `pub(crate)` re-export |
| handler 文件 | 已抽离 | `get_run_replay` 与 `get_run_status` 已迁入 `src/runtime/run/replay_status.rs` |
| replay query | 等价 | `RuntimeReplayQuery`、`normalized_replay_options`、cursor/filter/limit clamp 仍保留父级 owner |
| replay response | 等价 | 仍由 `run_replay_response_from_record` 负责 timeline、events、page cursor 和 bad cursor 映射 |
| status response | 等价 | 仍由 `run_status_response_from_record` 负责 status projection |
| metrics | 等价 | replay 成功路径仍调用 `record_replay_page`，metrics owner 未迁移 |
| sibling 边界 | 保留 | `stream_run_events` 留在 `src/runtime/run.rs`，`runtime.event_stream` 未迁移 |
| shared owner | 保留 | response mapping、schema、AppState、persistence owner 均不私有化到本叶 |

---

## 细分价值判断

**最终判定**: `runtime.run.replay_status` 当前不继续细拆。它已经是围绕 run replay/status 两个读侧 route 的完整 handler 叶子；继续细拆会把共享 projection、schema、metrics 或 state/persistence helper 包成更小 facade，但不会产生更清晰的 owner。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.run.replay_status.replay_page` | 不拆 | `get_run_replay` 是薄 handler，真实分页与 timeline projection owner 在 `runtime_response_mapping` |
| `runtime.run.replay_status.status_projection` | 不拆 | `get_run_status` 只是 record lookup 加 response mapping 调用，继续拆只会复制 facade |
| `runtime.run.replay_status.query_options` | 不拆 | `normalized_replay_options` 被 run replay 与 backtest replay 复用，不能私有化到 run replay leaf |
| `runtime.run.replay_status.metrics` | 不拆 | `RuntimeEvidenceMetrics::record_replay_page` 属全局 evidence metrics owner，不属于本叶 |
| `runtime.run.replay_status.record_lookup` | 不拆 | `load_run_record_from_state` 同时服务 record/detail/replay/status/mutation 等路径，是共享 helper owner |
| `runtime.event_stream` | 不在本叶内拆 | SSE 是父级 `backend.runtime.routes` 直接 route 子叶候选，不能混入 replay/status closeout |

因此本叶 closeout 后，递归流程应退出 `runtime.run.replay_status` 内部，回到父级 sibling 队列。

---

## 父子通信收口

```text
backend.runtime.routes.run
  -> crate::runtime::{get_run_replay,get_run_status}
  -> runtime::run_replay_status::{get_run_replay,get_run_status}
  -> load_run_record_from_state / normalized_replay_options
  -> runtime_response_mapping::{run_replay_response_from_record,run_status_response_from_record}
  -> frontend_api_types / RuntimeEvidenceMetrics / AppState
```

本叶只能经父级 `runtime` re-export 和 `backend.runtime.routes.run` 暴露 replay/status routes，不得横向接管 `runtime.event_stream`、`runtime.run.record_store`、`runtime.run.session_start`、`runtime.run.v4_handoff`、backtest replay、mutation、executor 或 frontend state。

---

## 后续 sibling 队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.event_stream` | 默认下一候选 | 需回到父级 `backend.runtime.routes` 建立单子叶等价基线；不得从 `runtime.run.replay_status` 内部直接迁移 |
| `runtime.run.replay_status` | 停止 | 本叶已 closeout，当前不继续细拆 |
| `runtime.run.record_store` | 停止 | 已在 BE-001J-05 closeout |
| `runtime.run.session_start` | 停止 | 已在 BE-001I-03 closeout |
| `runtime.run.v4_handoff` | 停止 | 已在 BE-001H-03 closeout |

---

## 本批次不做

- 不迁移 `runtime.event_stream`，即 `stream_run_events` 或 `/api/runtime/runs/:run_id/events`。
- 不私有化 `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters` 或 `normalized_replay_options`。
- 不迁移 `run_replay_response_from_record`、`run_status_response_from_record` 或 response projection helper。
- 不迁移 `RuntimeReplayResponse`、`RunStatusResponse` 或 frontend schema owner。
- 不迁移 `RuntimeEvidenceMetrics`、`record_replay_page` owner。
- 不迁移 `state.runs`、`run_store_dir`、`load_run_record_from_state`、AppState owner 或 persistence owner。
- 不改 frontend API、store、route caller 或 UI。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.run.replay_status` 已完成时，必须说明只完成 run replay/status 两个 handler 的抽离与单叶 closeout；`stream_run_events`、response mapping、schema、metrics、state owner、persistence owner、frontend route 和发布版本过渡均未完成。不得宣称 runtime run handler 全部完成，也不得把 `runtime.event_stream` 说成本叶的一部分。

---

## 验收标准

1. `69-runtime.run.replay_status单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.run.replay_status` closeout 完成并停止内部细分。
3. 全量树覆盖本 closeout 文档与 `src/runtime/run/replay_status.rs`。
4. 治理门禁能发现本 closeout 文档缺失。
5. `api_run` 与 `api_evidence_contract` 代表测试继续通过。
