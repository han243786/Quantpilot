# v4.16.0 runtime.backtest.replay_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DG-02
> 基准: `334-runtime.backtest.replay_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.backtest.replay_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DG-03 actual Rust import rewrite

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DG-02 `runtime.backtest.replay_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | explicit import rewrite、replay import、parent surface、release transition guard | 最小实际批次 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass` | replay import 收敛方案 |
| 模块树 | `runtime.backtest.replay_import_pass` | 抽离方案 |

---

## 适配性结论

BE-001DG-03 可以进入实际 Rust import rewrite，范围仅限:

```text
src/runtime/backtest/replay.rs
```

本文件是 direct route singleton，只有 `get_backtest_replay` 一个 public handler。它只读 backtest record、规范化 query、投影 replay response 并写入 replay page metric，适合作为单文件 import 收敛批次。

---

## 实际改写方案

删除:

```rust
use super::*;
```

新增:

```rust
use crate::{
    auth, backtest_replay_response_from_record, json_bad_request,
    load_backtest_record_from_state, normalized_replay_options, AppState, RuntimeReplayQuery,
    RuntimeReplayResponse,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::time::Instant;
```

该输入面继续通过父级白箱取得 shared helper 与 schema，保持父子通信规则；不新增 sibling horizontal link。

---

## 等价要求

BE-001DG-03 不得改变以下行为:

1. `get_backtest_replay` 的 route path、method、handler name 和 response schema。
2. `auth::UserId` scoped lookup 与 `load_backtest_record_from_state` owner。
3. `normalized_replay_options` 的 cursor、sequence cursor、checkpoint、limit clamp、filter cleanup 与 `RuntimeReplayQuery` schema。
4. `backtest_replay_response_from_record` 的 timeline、events、checkpoint、retention filter、bad cursor 和 pagination projection。
5. `json_bad_request("bad_replay_cursor", message)` 的 error code。
6. `state.evidence_metrics.record_replay_page` 的成功路径 metric 写入。

---

## 排除项

- BE-001DG-03 只改 `src/runtime/backtest/replay.rs` 顶部 import。
- 不处理 `src/runtime/backtest/experiment_sweep.rs`。
- 不处理 `src/runtime/backtest/execution_start.rs`。
- 不处理 `src/runtime/backtest/legacy_dispatch.rs`、`parameter_grid.rs`、`record_lifecycle.rs`、`start_orchestration.rs` 或 `v4_*` 文件。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不迁移 route registration、record lookup owner、query schema owner、response mapping owner、metrics owner、state owner、persistence owner 或 frontend caller。
- 不新增 sibling horizontal link。
- 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001DG-03 实际抽离后至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_backtest
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DG-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 下一步 BE-001DG-03 只允许改写 `src/runtime/backtest/replay.rs` 顶部 import。
3. 尚未改写 `use super::*`。
4. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest.replay_import_pass` 已抽离、`runtime.backtest_import_pass` 已完成或 parent import bridge 已清除。

---

## 验收标准

1. `335-runtime.backtest.replay_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001DG-03 只改 `src/runtime/backtest/replay.rs` 顶部 import。
3. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
