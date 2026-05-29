# v4.16.0 runtime.backtest.replay 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001U-02。  
> 基准: `103-runtime.backtest.replay单子叶等价基线.md`、`102-runtime.backtest.record_store单叶closeout.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: 建立 `runtime.backtest.replay` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。  
> 下一步: BE-001U-03 实际抽离记录。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001U replay 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 父级 re-export、route facade、record lookup、response mapping owner、schema/metrics owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.replay` | 细化 |
| 模块树 | `runtime.backtest.replay` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.replay` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.replay` |
| 当前真实文件 | `src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| 计划目标文件 | `src/runtime/backtest/replay.rs` |
| public 方法 | `get_backtest_replay` |
| 保留 shared helper | `load_backtest_record_from_state`、`normalized_replay_options`、`backtest_replay_response_from_record`、`runtime_replay_response`、`filtered_replay_events`、`cursor_from_replay_options`、`replay_event_items`、`replay_checkpoints`、`timeline_items_from_events`、`record_replay_page`、`json_bad_request` |
| 保留 public 类型 | `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint`、`RuntimeReplayEventItem` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 抽离目标

第一轮实际抽离只允许移动一个 backtest replay route handler:

| handler | route | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `get_backtest_replay` | `GET /api/runtime/backtests/:backtest_id/replay` | `src/runtime/backtest/replay.rs` | `RuntimeReplayQuery` 解析、`normalized_replay_options`、record lookup、bad cursor error、replay metrics、response schema、artifact event log 优先级 |

本方案不移动 list/detail/save/discard、execution_start、experiment、compare、report、response mapping、schema、metrics、state/persistence 或 artifact schema owner。

---

## 实施方案

1. 新建 `src/runtime/backtest/replay.rs`，只承载 `get_backtest_replay`。
2. 从 `src/runtime/backtest.rs` 移出 `get_backtest_replay`。
3. 保持 `src/runtime/backtest.rs` 继续拥有 experiment sweep、experiment record store 和后续 sibling。
4. 在 `src/runtime/mod.rs` 增加私有子模块和父级兼容出口:

```rust
#[path = "backtest/replay.rs"]
mod backtest_replay;
pub(crate) use backtest_replay::get_backtest_replay;
```

5. 保持 `src/backend/runtime/routes/backtest.rs` 不变；route facade 继续调用 `crate::runtime::get_backtest_replay`。
6. 保持 `RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayOptions`、`RuntimeReplayFilters` 在当前 owner；不得为了本叶抽离私有化，因为 run replay/status 仍复用这些 helper/schema。
7. 保持 `src/runtime_persistence.rs` 继续拥有 `load_backtest_record_from_state` 和 record lookup fallback。
8. 保持 `src/runtime_response_mapping.rs` 继续拥有 `backtest_replay_response_from_record`、`runtime_replay_response`、filter/cursor/checkpoint/timeline helper。
9. 保持 `src/frontend_api_types.rs` 继续拥有 `RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint`、`RuntimeReplayEventItem` 等 schema。
10. 保持 `src/lib.rs` 的 `RuntimeEvidenceMetrics::record_replay_page` owner 不变。
11. 代码移动后再补 BE-001U-03 实际抽离记录，并用 `api_backtest`、`api_evidence_contract`、`api_run` 证明等价。

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `runtime.backtest.record_store` | list/detail/save/discard 已 closeout，不能回头横向混入 |
| `runtime.backtest.execution_start` | backtest 创建路径及内部子叶已完成当前轮，不属于 replay handler |
| `runtime.backtest.experiment_sweep` | experiment list/detail/save/discard 与 variant persistence 需要另起基线 |
| `backtest_compare` | compare owner 在 `src/backtest_compare.rs`，不迁移 |
| `RuntimeReplayQuery` / replay options | run replay/status 与 backtest replay 共用，不能私有化到 backtest replay leaf |
| replay response mapping owner | `backtest_replay_response_from_record`、`runtime_replay_response`、filter/cursor/timeline helper 保留在 `src/runtime_response_mapping.rs` |
| schema owner | `RuntimeReplayResponse`、`RuntimeReplayFilters`、`RuntimeReplayRecordKind` 等保留在 `src/frontend_api_types.rs` |
| metrics owner | `RuntimeEvidenceMetrics::record_replay_page` 保留在 `src/lib.rs` |
| AppState / persistence owner | `state.backtests`、store dirs、`load_backtest_record_from_state` 不迁移 |
| artifact schema owner | event log / artifact bundle schema 不迁移 |
| frontend API | 不改前端 route、store、caller 或 response consumption |
| 整理/重构 | 不做目录美化、schema 改名、旧实现删除或测试资产汰换 |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| `include!("backtest.rs")` 与 re-export 重名 | 先从 `src/runtime/backtest.rs` 删除 `get_backtest_replay`，再在 `src/runtime/mod.rs` re-export，避免 duplicate definition |
| `normalized_replay_options` 可见性不足 | 保持在父级 `runtime` 模块，子模块通过父级可见性使用；不得移动到 replay 子模块 |
| response mapping 可见性不足 | 优先保持既有 module 可见性和 import，不迁移 mapping owner |
| record lookup owner 被误迁移 | `load_backtest_record_from_state` 必须留在 `src/runtime_persistence.rs` |
| artifact event log 优先级漂移 | `backtest_replay_response_from_record` 继续优先使用 `backtest_artifacts.event_log.events`，否则 fallback `record.events` |
| bad cursor 语义漂移 | response mapping error 仍映射为 `json_bad_request("bad_replay_cursor", message)` |
| metrics 漂移 | `record_replay_page` 调用必须保留在 replay 成功路径后，不改 health metrics 字段 |
| run replay 被误伤 | 不移动 `RuntimeReplayQuery`、`normalized_replay_options` 或 shared replay response mapping；运行 `api_run` 作为 sibling 保护 |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 route path、route method、response schema 或 error code。
2. 需要移动 `RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayOptions`、`RuntimeReplayFilters` 或 replay response schema。
3. 需要移动 `backtest_replay_response_from_record`、`runtime_replay_response`、filter/cursor/checkpoint/timeline helper 或 response mapping owner。
4. 需要移动 `load_backtest_record_from_state`、AppState、persistence owner、artifact schema owner 或 metrics owner。
5. 需要混入 record_store、execution_start、experiment_sweep、compare、report 或 frontend caller。
6. `cargo check -p quantpilot` 暴露的可见性问题无法通过父级 re-export 或显式 import 解决。
7. `cargo test -p quantpilot --test api_backtest`、`api_evidence_contract` 或 `api_run` 出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 BE-001U-03 `runtime.backtest.replay` 实际抽离记录: 按本方案移动 `get_backtest_replay` 到计划目标文件，保持父级 re-export、route facade、record lookup、query normalization、response mapping owner、schema owner、metrics owner、state/persistence owner、artifact schema owner 和 frontend route 不变。完成后再做单叶 closeout，判断本叶是否需要继续细拆。

---

## 幻觉检查点

AI 声称 `runtime.backtest.replay` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 `get_backtest_replay` 已迁移，不得宣称 record_store、execution_start、experiment、compare、response mapping owner、schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。ASCII guard: `release transition guard`。

---

## 验收标准

1. `104-runtime.backtest.replay抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.backtest.replay` 节点标记实际抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、真实文件和下一步计划目标。
4. 治理门禁能发现本方案文档、`no code movement`、下一步 BE-001U-03、禁止迁移边界和回归证据缺失。
5. 后续 BE-001U 实际抽离必须引用本方案，不得把 record_store、execution_start、experiment、compare、response mapping owner、schema owner、metrics owner、state owner、persistence owner 或 frontend route 混入第一轮迁移。
