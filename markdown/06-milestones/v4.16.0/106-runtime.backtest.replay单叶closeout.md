# v4.16.0 runtime.backtest.replay 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001U-04。  
> 基准: `105-runtime.backtest.replay抽离记录.md`、`104-runtime.backtest.replay抽离方案.md`、`103-runtime.backtest.replay单子叶等价基线.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: `runtime.backtest.replay` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆，`stop_split: true`。后续应回到 `runtime.backtest` sibling 队列，默认下一候选为 `runtime.backtest.experiment_sweep`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001U replay 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、shared helper owner、schema/metrics/state owner、细分停止条件 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.replay` | 单叶 closeout |
| 模块树 | `runtime.backtest.replay` | 设置停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.replay` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.replay` |
| 真实文件 | `src/runtime/backtest/replay.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public 方法 | `get_backtest_replay` |
| 保留 shared helper | `load_backtest_record_from_state`、`normalized_replay_options`、`backtest_replay_response_from_record`、`runtime_replay_response`、`filtered_replay_events`、`cursor_from_replay_options`、`replay_event_items`、`replay_checkpoints`、`timeline_items_from_events`、`record_replay_page`、`json_bad_request` |
| 保留 public 类型 | `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint`、`RuntimeReplayEventItem` |
| closeout 判定 | `stop_split: true` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `GET /api/runtime/backtests/:backtest_id/replay` 仍经 `backend.runtime.routes.backtest -> crate::runtime::get_backtest_replay` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `backtest_replay` 私有子模块与 `pub(crate)` re-export |
| handler 文件 | 已抽离 | `get_backtest_replay` 已迁入 `src/runtime/backtest/replay.rs` |
| record lookup | 等价 | 仍由 `load_backtest_record_from_state` 处理 scoped memory、artifact directory 与 transient fallback |
| query normalization | 等价 | 仍使用共享 `RuntimeReplayQuery` 与 `normalized_replay_options`，不改变 cursor/checkpoint/limit/filter 语义 |
| response mapping | 等价 | 仍经 `backtest_replay_response_from_record` 和 `runtime_replay_response` 输出 artifact event log 优先的 timeline |
| bad cursor | 等价 | response mapping error 仍映射为 `json_bad_request("bad_replay_cursor", message)` |
| metrics | 等价 | 成功生成 replay response 后继续调用 `record_replay_page` |
| shared owner | 保留 | query/options、response mapping、schema、metrics、state/persistence、artifact schema 和 frontend caller 均保留原 owner |
| sibling 边界 | 保留 | record_store、execution_start、experiment_sweep、compare sibling 均未迁移 |

---

## 细分价值判断

**最终判定**: `runtime.backtest.replay` 当前不继续细拆，设置 `stop_split: true`。

`get_backtest_replay` 抽离后只剩一层很薄的 route orchestration: 计时、query normalization、record lookup、response mapping、metrics 记录和 JSON 返回。继续拆成更小文件会增加父级 re-export 与导入面，但不会形成新的 owner；真正复杂的 replay query、record lookup、response mapping、schema 和 metrics 都是跨 run/backtest 的共享 owner，不能被本叶私有化。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.backtest.replay.query_adapter` | 不拆 | `RuntimeReplayQuery` 与 `normalized_replay_options` 同时服务 run/backtest replay，私有化会破坏共享语义 |
| `runtime.backtest.replay.record_lookup` | 不拆 | `load_backtest_record_from_state` 是 persistence/state 共享 owner，不属于 replay handler |
| `runtime.backtest.replay.response_projection` | 不拆 | `backtest_replay_response_from_record`、`runtime_replay_response` 和 timeline helper 属于 response mapping owner |
| `runtime.backtest.replay.metrics_hook` | 不拆 | `record_replay_page` 属于 `RuntimeEvidenceMetrics` owner，本叶只调用 |
| `runtime.backtest.replay.bad_cursor_error` | 不拆 | bad cursor 只是 response mapping error adapter，不值得形成独立叶子 |
| `runtime.backtest.replay.timeline_filter` | 不拆 | filter/cursor/checkpoint/timeline helper 是 run/backtest replay 共用 mapping owner |

---

## 父子通信收口

```text
backend.runtime.routes.backtest
  -> crate::runtime::get_backtest_replay
  -> runtime::backtest_replay::get_backtest_replay
  -> runtime_persistence / runtime_response_mapping / RuntimeEvidenceMetrics
  -> AppState::{backtests,backtest_store_dir,transient_backtest_store_dir,evidence_metrics}
```

本叶只能经父级 `runtime` re-export 和 `backend.runtime.routes.backtest` 暴露，不得横向接管 `runtime.backtest.record_store`、`runtime.backtest.execution_start`、`runtime.backtest.experiment_sweep`、`backtest_compare`、response mapping owner、schema owner、state/persistence owner、frontend caller 或其他 sibling。发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续 sibling 队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest.experiment_sweep` | 默认下一候选 | 覆盖 experiment start/list/detail/save/discard 和 variant persistence，必须先建立单子叶等价基线 |
| `backtest_compare` | 独立 owner | compare core 在 `src/backtest_compare.rs`，若抽离应按 compare owner 单独处理 |
| `runtime.backtest.replay` | 停止 | 本叶已 closeout，当前不继续细拆 |
| `runtime.backtest.record_store` | 停止 | 已 closeout 并设置 `stop_split: true` |
| `runtime.backtest.execution_start` | 停止当前轮 | 已完成内部子叶 closeout 与父叶残余判断，不回流 record/state/persistence |

---

## 本批次不做

- 不迁移 `runtime.backtest.experiment_sweep`，即 `start_backtest_experiment`、experiment save/detail/list/discard 或 variant persistence。
- 不迁移 `backtest_compare`，即 compare core、compare narrative 或 compare route owner。
- 不迁移 `runtime.backtest.record_store` 或 `runtime.backtest.execution_start`。
- 不私有化 `RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayOptions`、`RuntimeReplayFilters`。
- 不私有化 `backtest_replay_response_from_record`、`runtime_replay_response`、filter/cursor/checkpoint/timeline helper。
- 不私有化 `load_backtest_record_from_state`、AppState、store dirs、persistence owner 或 artifact schema owner。
- 不私有化 `RuntimeReplayResponse` 等 frontend schema、frontend caller、route consumer 或 test asset strategy。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.backtest.replay` 已完成时，必须说明只是 `get_backtest_replay` handler 子模块完成抽离与 closeout，并设置 `stop_split: true`；`src/runtime/backtest.rs` 仍拥有 experiment sweep 和其他 sibling；query normalization、response mapping、schema、metrics、state owner、persistence owner、artifact schema owner、frontend caller、发布版本过渡、整理和重构均未完成。不得宣称 backtest handler 全部完成。

---

## 验收标准

1. `106-runtime.backtest.replay单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.backtest.replay` closeout 完成并设置 `stop_split: true`。
3. 全量树覆盖本 closeout 文档与 `src/runtime/backtest/replay.rs`。
4. 治理门禁能发现本 closeout 文档、`stop_split: true`、禁止迁移边界、下一候选和回归证据缺失。
5. `api_backtest`、`api_evidence_contract` 和 `api_run` 代表测试继续通过。
