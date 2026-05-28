# v4.16.0 runtime.run.replay_status 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001K-03。  
> 基准: `66-runtime.run.replay_status单子叶等价基线.md`、`67-runtime.run.replay_status抽离方案.md`。  
> 判定: 按方案完成 `runtime.run.replay_status` 第一轮实际抽离；只移动 `get_run_replay` 与 `get_run_status` 两个 handler，不迁移 SSE、response mapping owner、schema owner、metrics owner、state owner、persistence owner 或 frontend route。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001K replay_status 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、最小迁移、SSE 排除、shared owner 保留 | 落地 |
| 引导矩阵 | `runtime.run.replay_status` 白箱节点 | 更新 |
| 模块树 | `runtime.run.replay_status` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.replay_status` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.replay_status` |
| 新真实文件 | `src/runtime/run/replay_status.rs` |
| 保留真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public 方法 | `get_run_replay`、`get_run_status` |
| 保留 shared helper | `load_run_record_from_state`、`normalized_replay_options`、`run_replay_response_from_record`、`run_status_response_from_record`、`json_bad_request`、`RuntimeEvidenceMetrics::record_replay_page` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 replay_status 子模块 | `src/runtime/run/replay_status.rs` | 承载 `get_run_replay` 与 `get_run_status` |
| 删除旧位置两个 handler | `src/runtime/run.rs` | 文件开头直接进入 `stream_run_events`；SSE 与后续 legacy blocks 保留原位 |
| 父级兼容出口 | `src/runtime/mod.rs` | 增加 `run_replay_status` 私有子模块和 `pub(crate) use` |
| route facade | `src/backend/runtime/routes/run.rs` | 未改动，仍调用 `crate::runtime::*` 两个 handler |

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| run replay | `GET /api/runtime/runs/:run_id/replay` 仍使用 `RuntimeReplayQuery`、`normalized_replay_options`、`load_run_record_from_state` 和 `run_replay_response_from_record` |
| bad cursor | replay cursor 错误仍映射为 `json_bad_request("bad_replay_cursor", message)` |
| replay metrics | replay 成功路径仍调用 `state.evidence_metrics.record_replay_page(...)` |
| run status | `GET /api/runtime/runs/:run_id/status` 仍使用 `run_status_response_from_record` |
| route facade | `src/backend/runtime/routes/run.rs` path、method、handler 调用顺序不变 |
| helper/schema owner | response mapping、frontend API schema、metrics、state 与 persistence owner 均保留原 owner |

---

## 明确未迁移

- 不迁移 `runtime.event_stream`，即 `stream_run_events` 与 `/api/runtime/runs/:run_id/events`。
- 不迁移 `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters` 或 `normalized_replay_options`。
- 不迁移 `run_replay_response_from_record`、`run_status_response_from_record` 或 response projection helper。
- 不迁移 `RuntimeReplayResponse`、`RunStatusResponse` 或 frontend schema owner。
- 不迁移 `RuntimeEvidenceMetrics`、`record_replay_page` owner。
- 不迁移 `state.runs`、`run_store_dir`、`load_run_record_from_state`、AppState owner 或 persistence owner。
- 不改 frontend API、store、route caller 或 UI。

---

## 回退点

若后续发现行为回归，可将两个 handler 从 `src/runtime/run/replay_status.rs` 放回 `src/runtime/run.rs`，并移除 `src/runtime/mod.rs` 中的 `run_replay_status` 私有模块与 re-export。`src/backend/runtime/routes/run.rs` 不需要回退，因为本批没有修改 route facade。

---

## 验证计划

本批收口必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_evidence_contract
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 `runtime.run.replay_status` 单叶整理 / closeout，确认两个 handler 抽离后与原功能等价，并判断本叶内部是否值得继续细拆。当前默认不继续拆 response mapping、schema、metrics、state 或 persistence helper；如要拆这些 shared owner，必须另起父级共享节点方案。

---

## 验收标准

1. `68-runtime.run.replay_status抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/run/replay_status.rs` 进入全量树和模块树。
3. `src/runtime/mod.rs` 保留 `crate::runtime::{get_run_replay,get_run_status}` 兼容出口。
4. `src/backend/runtime/routes/run.rs` route path/method 不变。
5. 治理门禁能发现本抽离记录缺失。
6. `api_run` 与 `api_evidence_contract` 证明 replay/status 相关服务级契约仍可通过。
