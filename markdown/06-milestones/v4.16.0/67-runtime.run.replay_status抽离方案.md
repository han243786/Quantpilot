# v4.16.0 runtime.run.replay_status 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001K-02。  
> 基准: `66-runtime.run.replay_status单子叶等价基线.md`。  
> 判定: 建立 `runtime.run.replay_status` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001K replay_status 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 父级 re-export、SSE 排除、response mapping owner、schema/metrics owner 保留 | 固化 |
| 引导矩阵 | `runtime.run.replay_status` 白箱节点 | 细化 |
| 模块树 | `runtime.run.replay_status` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.replay_status` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.replay_status` |
| 当前真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| 计划目标文件 | `src/runtime/run/replay_status.rs` |
| public 方法 | `get_run_replay`、`get_run_status` |
| 保留 shared helper | `load_run_record_from_state`、`normalized_replay_options`、`run_replay_response_from_record`、`run_status_response_from_record`、`json_bad_request`、`RuntimeEvidenceMetrics::record_replay_page` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 抽离目标

第一轮实际抽离只允许移动两个 replay/status route handler:

| handler | route | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `get_run_replay` | `GET /api/runtime/runs/:run_id/replay` | `src/runtime/run/replay_status.rs` | `RuntimeReplayQuery` 解析、`normalized_replay_options`、record lookup、bad cursor error、replay metrics、response schema |
| `get_run_status` | `GET /api/runtime/runs/:run_id/status` | `src/runtime/run/replay_status.rs` | record lookup、`run_status_response_from_record`、`RunStatusResponse` schema |

本方案不移动 `stream_run_events`，不新建或修改 `/api/runtime/runs/:run_id/events`，不改 replay/status route path、method、payload、response schema 或 error code。

---

## 实施方案

1. 新建 `src/runtime/run/replay_status.rs`，只承载 `get_run_replay` 和 `get_run_status`。
2. 从 `src/runtime/run.rs` 移出 `get_run_replay` 和 `get_run_status`。
3. 保持 `src/runtime/run.rs` 继续拥有 `stream_run_events`、approval/merge legacy blocks 和后续 sibling。
4. 在 `src/runtime/mod.rs` 增加私有子模块和父级兼容出口:

```rust
#[path = "run/replay_status.rs"]
mod run_replay_status;
pub(crate) use run_replay_status::{get_run_replay, get_run_status};
```

5. 保持 `src/backend/runtime/routes/run.rs` 不变；route facade 继续调用 `crate::runtime::{get_run_replay,get_run_status}`。
6. 保持 `RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayOptions`、`RuntimeReplayFilters` 在当前 owner；不得为了本叶抽离私有化，因为 backtest replay 仍复用这些 helper/schema。
7. 保持 `src/runtime_response_mapping.rs` 继续拥有 `run_replay_response_from_record`、`run_status_response_from_record` 和 replay response projection helper。
8. 保持 `src/frontend_api_types.rs` 继续拥有 `RuntimeReplayResponse`、`RunStatusResponse` 等 schema。
9. 保持 `src/lib.rs` 的 `RuntimeEvidenceMetrics::record_replay_page` owner 不变。
10. 代码移动后再补 BE-001K-03 实际抽离记录，并用 `api_run` 与 `api_evidence_contract` 证明等价。

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `runtime.event_stream` | `stream_run_events` 是 SSE 流协议和 keep-alive 生命周期，不属于 replay/status handler 抽离 |
| `runtime.run.record_store` | list/detail/save/discard 已 closeout，不能回头横向混入 |
| `runtime.run.session_start` / `runtime.run.v4_handoff` | 两者已 closeout，当前不继续细拆 |
| `RuntimeReplayQuery` / replay options | backtest replay 复用，不能私有化到 run replay leaf |
| `run_replay_response_from_record` / `run_status_response_from_record` | response mapping owner 保留在 `src/runtime_response_mapping.rs` |
| `RuntimeReplayResponse` / `RunStatusResponse` | schema owner 保留在 `src/frontend_api_types.rs` |
| `RuntimeEvidenceMetrics` | metrics owner 保留在 `src/lib.rs` |
| AppState / persistence owner | `state.runs`、`run_store_dir`、`load_run_record_from_state` 不迁移 |
| frontend API | 本批不改前端 route、store 或 caller |
| 整理/重构 | 不做目录美化、schema 改名、旧实现删除或测试资产汰换 |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| `include!("run.rs")` 与 re-export 重名 | 先从 `src/runtime/run.rs` 删除两个 handler，再在 `src/runtime/mod.rs` re-export，避免 duplicate definition |
| `normalized_replay_options` 可见性不足 | 保持在父级 `runtime` 模块，子模块通过 `use super::*` 使用；不得移动到子模块 |
| response mapping 可见性不足 | 优先保持既有 module 可见性和 import，不迁移 mapping owner |
| backtest replay 被误伤 | 不移动 `RuntimeReplayQuery`、`normalized_replay_options` 或 `RuntimeReplayOptions`；运行 `api_evidence_contract` 作为 replay contract 代表证据 |
| SSE 被误混入 | `stream_run_events` 留在 `src/runtime/run.rs`；任何需要改 SSE route 或 keep-alive 的问题都中止本方案 |
| metrics 漂移 | `record_replay_page` 调用必须保留在 replay 成功路径后，不改 health metrics 字段 |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 route path、route method、response schema 或 error code。
2. 需要把 `stream_run_events` 或 `/api/runtime/runs/:run_id/events` 混入本叶。
3. 需要移动 `RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayOptions` 或 `RuntimeReplayFilters`。
4. 需要移动 `run_replay_response_from_record`、`run_status_response_from_record` 或 schema owner。
5. 需要改变 `RuntimeEvidenceMetrics`、AppState、persistence owner 或 lock/state 语义。
6. `cargo check -p quantpilot` 暴露的可见性问题无法通过父级 re-export 或显式 import 解决。
7. `cargo test -p quantpilot --test api_run` 或 `cargo test -p quantpilot --test api_evidence_contract` 出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

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

下一批应进入 BE-001K-03 `runtime.run.replay_status` 实际抽离记录: 按本方案移动 `get_run_replay` 和 `get_run_status` 到 `src/runtime/run/replay_status.rs`，保持父级 re-export、route facade、SSE、response mapping owner、schema owner、metrics owner、state owner 和 persistence owner 不变。完成后再做单叶 closeout，判断本叶是否需要继续细拆。

---

## 验收标准

1. `67-runtime.run.replay_status抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.run.replay_status` 节点标记抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、真实文件和下一步计划目标。
4. 治理门禁能发现本方案文档缺失。
5. 后续 BE-001K 实际抽离必须引用本方案，不得把 `runtime.event_stream`、response mapping owner、schema owner、metrics owner、state owner 或 persistence owner 混入第一轮迁移。
