# v4.16.0 runtime.backtest.replay_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DG-01
> 基准: `333-runtime.backtest_import_pass父叶残余判断.md`
> 目标子叶: `runtime.backtest.replay_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DG-02 `runtime.backtest.replay_import_pass` 抽离方案

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DG-01 `runtime.backtest.replay_import_pass` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | explicit import pass、replay import、parent surface、release transition guard | 等价冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass` | backtest replay import 白箱 |
| 模块树 | `runtime.backtest.replay_import_pass` | 新增基线 |

---

## 当前文件

```text
src/runtime/backtest/replay.rs
```

当前文件仍以父级通配导入作为输入面:

```rust
use super::*;
```

本基线只冻结该文件的 import 收敛边界，不改写 Rust。

---

## public 方法边界

| 方法 | 输入 | 输出 | 等价要求 |
| --- | --- | --- | --- |
| `get_backtest_replay` | `auth::UserId`、`State<AppState>`、`Path<String>`、`Query<RuntimeReplayQuery>` | `Json<RuntimeReplayResponse>` | scoped lookup、query normalization、bad cursor error、replay response projection 与 replay page metric 不变 |

---

## 当前调用链

```text
backend.runtime.routes.backtest
  -> GET /api/runtime/backtests/:backtest_id/replay
  -> crate::runtime::get_backtest_replay
  -> src/runtime/backtest/replay.rs::get_backtest_replay
  -> normalized_replay_options
  -> load_backtest_record_from_state
  -> backtest_replay_response_from_record
  -> state.evidence_metrics.record_replay_page
```

后续 import rewrite 必须保持 route facade、handler name、response schema、error code 和 metric 写入位置不变。

---

## 预期显式输入面

BE-001DG-03 实际抽离时，预计将 `use super::*` 收敛为如下父级白箱输入:

```rust
use crate::{
    auth, backtest_replay_response_from_record, json_bad_request, load_backtest_record_from_state,
    runtime::{normalized_replay_options, RuntimeReplayQuery},
    AppState, RuntimeReplayResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::time::Instant;
```

该输入面继续通过父级白箱取得 shared helper 与 schema，不新增 sibling horizontal link。

---

## 等价保护

1. 不改变 `GET /api/runtime/backtests/:backtest_id/replay` 的 path、method、handler name 或 response schema。
2. 不改变 `normalized_replay_options` 的 cursor、sequence cursor、checkpoint、limit clamp 或 filter cleanup 语义。
3. 不改变 `load_backtest_record_from_state` 的 scoped lookup 与 storage fallback owner。
4. 不改变 `backtest_replay_response_from_record` 的 timeline、events、checkpoint、retention filter、bad cursor 和 pagination projection。
5. 不改变 `json_bad_request("bad_replay_cursor", message)` 的 status/error code 语义。
6. 不改变 `state.evidence_metrics.record_replay_page` 的成功路径 metric 写入。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/replay.rs`。
- 本批不处理 `src/runtime/backtest/experiment_sweep.rs` 或其内部子文件。
- 本批不处理 `src/runtime/backtest/execution_start.rs` 或其内部子文件。
- 本批不处理 `src/runtime/backtest/legacy_dispatch.rs`、`parameter_grid.rs`、`record_lifecycle.rs`、`start_orchestration.rs` 或 `v4_*` 文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 单子叶等价基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_backtest
```

---

## 幻觉检查点

AI 声称 BE-001DG-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标文件只有 `src/runtime/backtest/replay.rs`。
3. 当前尚未改写 `use super::*`。
4. 下一步只能进入 BE-001DG-02 `runtime.backtest.replay_import_pass` 抽离方案。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest.replay_import_pass` 已抽离、`runtime.backtest_import_pass` 已完成或 parent import bridge 已清除。

---

## 验收标准

1. `334-runtime.backtest.replay_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 `src/runtime/backtest/replay.rs` 的 1 个 public 方法与预期显式输入面。
3. 下一步固定为 BE-001DG-02 `runtime.backtest.replay_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
