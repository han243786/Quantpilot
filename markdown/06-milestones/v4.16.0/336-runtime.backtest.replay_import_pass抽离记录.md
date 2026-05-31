# v4.16.0 runtime.backtest.replay_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DG-03
> 基准: `335-runtime.backtest.replay_import_pass抽离方案.md`
> 目标子叶: `runtime.backtest.replay_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass`
> 代码动作: actual Rust import rewrite
> 下一步: BE-001DG-04 `runtime.backtest.replay_import_pass` 单叶 closeout

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DG-03 `runtime.backtest.replay_import_pass` 实际抽离 | 实际抽离 |
| 规范矩阵 | explicit import rewrite、replay import、parent surface、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.replay_import_pass` | replay import 白箱收敛 |
| 模块树 | `runtime.backtest.replay_import_pass` | 抽离记录 |

---

## 实际改动

本批只改写:

```text
src/runtime/backtest/replay.rs
```

删除:

```rust
use super::*;
```

新增显式输入:

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

该输入面继续通过父级白箱取得 shared helper 与 schema，未新增 sibling horizontal link。

---

## 等价结果

- `get_backtest_replay` 行为未变。
- route path、method、handler name、status code、error code 和 response schema 均未改变。
- `normalized_replay_options`、`load_backtest_record_from_state`、`backtest_replay_response_from_record` 和 `record_replay_page` owner 均未迁移。
- bad cursor 仍映射为 `json_bad_request("bad_replay_cursor", message)`。
- runtime parent bridge 依赖文件数从 33 降为 32。

计数锚点: root 1 / run 0 / backtest 9 / mutation 21 / test-only 1 / total 32。

---

## 排除项

- 本批只改 `src/runtime/backtest/replay.rs` 顶部 import。
- 本批不处理 `src/runtime/backtest/experiment_sweep.rs`。
- 本批不处理 `src/runtime/backtest/execution_start.rs`。
- 本批不处理 `src/runtime/backtest/legacy_dispatch.rs`、`parameter_grid.rs`、`record_lifecycle.rs`、`start_orchestration.rs` 或 `v4_*` 文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route registration、record lookup owner、query schema owner、response mapping owner、metrics owner、state owner、persistence owner 或 frontend caller。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批提交前至少执行:

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

AI 声称 BE-001DG-03 完成时，必须说明:

1. 本批次是 actual Rust import rewrite。
2. 只改写 `src/runtime/backtest/replay.rs`。
3. runtime parent bridge 依赖文件数从 33 降为 32。
4. 下一步只能进入 BE-001DG-04 `runtime.backtest.replay_import_pass` 单叶 closeout。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已完成、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `336-runtime.backtest.replay_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/replay.rs` 不再包含 `use super::*` 或 `super::`。
3. runtime parent bridge 残余计数为 32。
4. Rust 编译、`api_backtest`、治理门禁、全量树覆盖和 `git diff --check` 均通过。
