# v4.16.0 runtime.run_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DB-03
> 基准: `323-runtime.run_import_pass抽离方案.md`
> 目标子叶: `runtime.run_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass`
> 代码动作: actual Rust import rewrite

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DB-03 `runtime.run_import_pass` 实际抽离 | 实际抽离 |
| 规范矩阵 | explicit import pass、run child import、parent surface、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.run_import_pass` | run import 白箱收敛 |
| 模块树 | `runtime.run_import_pass` | 抽离记录 |

---

## 实际改动

本批只改写 4 个 run child 顶部 import:

```text
src/runtime/run/v4_handoff.rs
src/runtime/run/session_start.rs
src/runtime/run/record_store.rs
src/runtime/run/replay_status.rs
```

4 个文件均删除:

```rust
use super::*;
```

并改为显式 import。

### `src/runtime/run/v4_handoff.rs`

新增显式输入:

```rust
use crate::runtime::RunInProgressGuard;
use crate::{current_time_ms, internal_error, json_bad_request_with_code, AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
```

### `src/runtime/run/session_start.rs`

新增显式输入包含 `auth`、run compile / sandbox / governance helpers、`RunInProgressGuard`、`FrontendRunRequest`、`RunRecord`、`RunStartResponse`、`RealTimeSandbox`、`RuntimeCoordinator`、`RUN_WINDOW_MS`、Axum extractors、`compile_runtime_protocol_config` 与 `Sandbox` trait。

`RunInProgressGuard` 继续通过 `crate::runtime::RunInProgressGuard` 父级白箱输入，未新增 sibling horizontal link。

### `src/runtime/run/record_store.rs`

新增显式输入包含 `auth`、run persistence helpers、graph audit helpers、pagination helpers、response builders、`DiscardRuntimeArtifactResponse`、`RunDetailResponse`、`RunListItem`、Axum extractors 与 `tokio::fs`。

`DiscardRuntimeArtifactResponse` 继续通过 `crate::runtime::DiscardRuntimeArtifactResponse` 父级白箱输入。

### `src/runtime/run/replay_status.rs`

新增显式输入包含 `auth`、run record loading、replay/status response builders、`RuntimeReplayQuery`、`normalized_replay_options`、`RuntimeReplayResponse`、`RunStatusResponse`、Axum extractors 与 `Instant`。

`RuntimeReplayQuery` 与 `normalized_replay_options` 继续通过 `crate::runtime::{...}` 父级白箱输入。

---

## 等价结果

- `start_v4_runtime_run` 行为未变。
- `start_test_run` 行为未变。
- run list/detail/save/discard 行为未变。
- run replay/status 行为未变。
- `RunInProgressGuard` run mutex 行为未变。
- `RunStartResponse`、`RunDetailResponse`、`RunListItem`、`DiscardRuntimeArtifactResponse`、`RuntimeReplayResponse` 与 `RunStatusResponse` schema 未变。
- 未修改 `src/runtime/mod.rs` 的 root parent bridge。
- 未处理 backtest/mutation 子树或 test-only `src/runtime/run_guard.rs`。
- 未新增 sibling horizontal link，未启动 release transition。

runtime parent bridge 依赖文件数从 38 降为 34。

---

## 排除项

- 不修改 `src/runtime/mod.rs`。
- 不处理 `src/runtime/backtest/**`。
- 不处理 `src/runtime/mutation/**`。
- 不处理 test-only `src/runtime/run_guard.rs`。
- 不移动 handler、type、helper、route facade、schema owner、state owner 或 frontend caller。
- 不新增 sibling horizontal link。
- 不启动 release transition。

---

## 验证要求

本批实际 Rust import rewrite 提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_sse
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001DB-04 runtime.run_import_pass 单叶 closeout
```

BE-001DB-04 需要判断 `runtime.run_import_pass` 是否设置 `stop_split: true`，并决定后续回到 `runtime.parent_import_bridge` 父叶残余判断还是进入其他候选。

---

## 幻觉检查点

AI 声称 BE-001DB-03 完成时，必须说明:

1. 本批只改写 4 个 run child 的 import。
2. `src/runtime/mod.rs` root parent bridge 尚未处理。
3. runtime parent bridge 依赖文件数只从 38 降为 34。
4. 下一步只能进入 BE-001DB-04 `runtime.run_import_pass` 单叶 closeout。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 parent import bridge 已消除、`backend.runtime` 已完成或 Rust 重构已完成。

---

## 验收标准

1. 4 个目标 run child 不再包含 `use super::*` 或 `super::`。
2. `cargo check -p quantpilot` 与 `api_run` / `api_sse` 通过。
3. `324-runtime.run_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 下一步固定为 BE-001DB-04 `runtime.run_import_pass` 单叶 closeout。
