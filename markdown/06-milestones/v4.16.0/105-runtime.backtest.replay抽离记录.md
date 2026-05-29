# v4.16.0 runtime.backtest.replay 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001U-03。  
> 基准: `104-runtime.backtest.replay抽离方案.md`、`103-runtime.backtest.replay单子叶等价基线.md`、`102-runtime.backtest.record_store单叶closeout.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: 按方案完成 `runtime.backtest.replay` 第一轮实际抽离；只迁移 `get_backtest_replay`，不迁移 query normalization、response mapping、schema、metrics、record lookup、state/persistence、artifact schema、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001U replay 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、route facade 不变、shared owner 保留 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.replay` | 物理抽离 |
| 模块树 | `runtime.backtest.replay` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.replay` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.replay` |
| 新真实文件 | `src/runtime/backtest/replay.rs` |
| 保留真实文件 | `src/runtime/backtest.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/frontend_api_types.rs`、`src/lib.rs` |
| public 方法 | `get_backtest_replay` |
| 保留 shared helper | `load_backtest_record_from_state`、`normalized_replay_options`、`backtest_replay_response_from_record`、`runtime_replay_response`、`filtered_replay_events`、`cursor_from_replay_options`、`replay_event_items`、`replay_checkpoints`、`timeline_items_from_events`、`record_replay_page`、`json_bad_request` |
| 保留 public 类型 | `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint`、`RuntimeReplayEventItem` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 replay 子模块 | `src/runtime/backtest/replay.rs` | 承载 `get_backtest_replay` |
| 删除旧位置 handler | `src/runtime/backtest.rs` | 文件继续承载 experiment sweep 和后续 sibling |
| 父级兼容出口 | `src/runtime/mod.rs` | 增加 `backtest_replay` 私有子模块和 `pub(crate) use` |
| route facade | `src/backend/runtime/routes/backtest.rs` | 未改动，仍调用 `crate::runtime::get_backtest_replay` |

父级 re-export 形态:

```rust
#[path = "backtest/replay.rs"]
mod backtest_replay;
pub(crate) use backtest_replay::get_backtest_replay;
```

子模块形态:

```rust
use super::*;

pub(crate) async fn get_backtest_replay(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(backtest_id): Path<String>,
    Query(query): Query<RuntimeReplayQuery>,
) -> Result<Json<RuntimeReplayResponse>, (StatusCode, String)>
```

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| route | `GET /api/runtime/backtests/:backtest_id/replay` path、method、route facade 和 handler 名不变 |
| record lookup | 仍经 `load_backtest_record_from_state` 按用户作用域读取 memory、artifact directory 或 transient fallback |
| query normalization | `RuntimeReplayQuery`、`normalized_replay_options`、默认 limit、max limit、checkpoint/cursor 优先级不变 |
| response mapping | 仍经 `backtest_replay_response_from_record` 和 `runtime_replay_response` 输出 artifact event log 优先的 replay response |
| bad cursor | response mapping error 仍映射为 `json_bad_request("bad_replay_cursor", message)` |
| metrics | 成功生成 replay response 后仍调用 `record_replay_page` |
| schema | `RuntimeReplayResponse`、`RuntimeReplayRecordKind`、`RuntimeReplayCheckpoint`、`RuntimeReplayEventItem` owner 不变 |

---

## 明确未迁移

- 不迁移 `runtime.backtest.record_store`，即 list/detail/save/discard handler 或 record store closeout 结论。
- 不迁移 `runtime.backtest.execution_start` 及其 v4 projection、request resolution、runtime execution、legacy dispatch 子叶。
- 不迁移 `runtime.backtest.experiment_sweep`，即 experiment list/detail/save/discard 或 variant persistence。
- 不迁移 `backtest_compare`、compare route、compare narrative 或 compare core。
- 不迁移 `RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayOptions`、`RuntimeReplayFilters`。
- 不迁移 `backtest_replay_response_from_record`、`runtime_replay_response`、filter/cursor/checkpoint/timeline helper。
- 不迁移 `load_backtest_record_from_state`、AppState、store dirs、persistence owner 或 artifact schema owner。
- 不迁移 `RuntimeReplayResponse` 等 frontend schema、frontend caller、route consumer 或 test asset strategy。
- 不进入整理、重构、发布版本过渡或性能连接优化。ASCII guard: `release transition guard`。

---

## 回退点

若后续发现行为回归，可将 `get_backtest_replay` 从 `src/runtime/backtest/replay.rs` 放回 `src/runtime/backtest.rs`，并移除 `src/runtime/mod.rs` 中的 `backtest_replay` 私有模块与 re-export。`src/backend/runtime/routes/backtest.rs` 不需要回退，因为本批没有修改 route facade。

---

## 验证计划

本批收口必须运行:

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

下一批应进入 `runtime.backtest.replay` 单叶 closeout，确认 `get_backtest_replay` 抽离后与原功能等价，并判断 replay handler 内部是否值得继续细拆。当前不能直接拆 query normalization、response mapping、schema、metrics、record lookup、state/persistence、artifact schema、frontend route 或发布过渡连接。ASCII marker: `next closeout marker`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.replay` 已抽离时，必须说明只迁移了 `get_backtest_replay` 到 `src/runtime/backtest/replay.rs`，且 `src/runtime/mod.rs` 只增加父级 re-export；route facade、query normalization、response mapping、schema、metrics、record lookup、state/persistence、artifact schema 和 frontend caller 均未迁移。不得宣称 replay 已 closeout、experiment/compare 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `105-runtime.backtest.replay抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/backtest/replay.rs` 进入全量树和模块树。
3. `src/runtime/mod.rs` 保留 `crate::runtime::get_backtest_replay` 兼容出口。
4. `src/backend/runtime/routes/backtest.rs` route path/method 不变。
5. 治理门禁能发现本抽离记录缺失。
6. `api_backtest`、`api_evidence_contract` 和 `api_run` 证明 backtest replay 与关联 evidence/run sibling 契约仍可通过。
