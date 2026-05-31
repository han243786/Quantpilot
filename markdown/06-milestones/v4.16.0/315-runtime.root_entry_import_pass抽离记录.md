# v4.16.0 runtime.root_entry_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CY-03
> 基准: `314-runtime.root_entry_import_pass抽离方案.md`
> 目标子叶: `runtime.root_entry_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass`
> 代码动作: actual Rust import rewrite

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CY-03 `runtime.root_entry_import_pass` 实际抽离 | 实际抽离 |
| 规范矩阵 | explicit import pass、minimum batch、parent import bridge、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_entry_import_pass` | root entry 白箱 import 收敛 |
| 模块树 | `runtime.root_entry_import_pass` | 抽离记录 |

---

## 实际改动

本批只执行 BE-001CY-02 指定的 two-handler root entry pilot:

```text
src/runtime/event_stream.rs
src/runtime/evidence_health.rs
```

### `src/runtime/event_stream.rs`

删除 parent wildcard:

```rust
use super::*;
```

改为显式 import:

```rust
use crate::{
    auth, json_sse_event, load_run_record_from_state, sleep, stream, AppState, Duration, Event,
    Infallible, KeepAlive, Path, Sse, State, StatusCode, SSE_EVENT_DELAY_MS,
};
```

`auth::UserId`、`AppState`、`State`、`Path`、`Sse`、`Event`、`KeepAlive`、`Infallible`、`stream!`、`Duration`、`sleep`、`StatusCode`、`SSE_EVENT_DELAY_MS`、`load_run_record_from_state` 与 `json_sse_event` 仍来自既有父级白箱 surface，不新增横向依赖。

### `src/runtime/evidence_health.rs`

删除 parent wildcard:

```rust
use super::*;
```

改为显式 import:

```rust
use crate::{
    cleanup_transient_runtime_report_outputs, current_time_ms, io_error,
    list_runtime_report_records, runtime_evidence_cleanup_policy, AppState,
    RuntimeEvidenceCleanupRequest, RuntimeEvidenceCleanupResponse, RuntimeEvidenceHealthResponse,
    RuntimeEvidenceReportRecord, RuntimeEvidenceReportStatusCounts, RuntimeReportLifecycleStatus,
};
use axum::{extract::State, http::StatusCode, Json};
```

`RuntimeEvidenceReportRecord`、`RuntimeEvidenceReportStatusCounts`、`RuntimeReportLifecycleStatus`、`RuntimeEvidenceHealthResponse`、`RuntimeEvidenceCleanupRequest`、`RuntimeEvidenceCleanupResponse`、runtime report store helpers、cleanup helpers、`current_time_ms` 与 `io_error` 的调用边界保持不变。

---

## 等价结果

- `stream_run_events` 的 route path、auth scope、run lookup、SSE frame order、`run_started` / `runtime_event` / `account` / `run_completed` envelope、keep alive 与 `SSE_EVENT_DELAY_MS` 均未变更。
- `get_runtime_evidence_health` 与 `cleanup_runtime_evidence` 的 response schema、cleanup policy、runtime report store helpers、metrics owner、storage lifecycle owner 与 `AppState` lock order 均未变更。
- `src/runtime/report_ops.rs` 未处理，后续另起 `runtime.report_ops_import_pass`。
- `src/runtime/run_guard.rs` 未处理，其 `use super::*` 仍仅属于 test-only super import。
- `src/runtime/mod.rs` 父桥未删除，parent import bridge 仍需后续 staged explicit import pass。
- 本批未新增 sibling horizontal link，未启动 release transition。

当前 `src/runtime/**.rs` 中存在 `use super::*` 或 `super::` 依赖的文件数从 44 降为 42。

---

## 排除项

- 不处理 `src/runtime/report_ops.rs`、`src/runtime/run_guard.rs` 或 `src/runtime/mod.rs`。
- 不处理 run/backtest/mutation/report_ops 子树。
- 不改变 `pub(crate) use` route-facing surface。
- 不改变 API path/method、response schema、SSE event name、event order、cleanup policy 或 report store persistence。
- 不启动 release transition，不新增性能旁路或 sibling horizontal link。

---

## 验证要求

本批实际 Rust import rewrite 提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_sse
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_v1_ops_health
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

若后续发现 report output 受影响，必须补跑 `cargo test -p quantpilot --test api_v1_reports`。当前批次未改动 report output。

---

## 下一步

下一步只允许进入:

```text
BE-001CY-04 runtime.root_entry_import_pass 单叶 closeout
```

BE-001CY-04 需要判断 `runtime.root_entry_import_pass` 是否等价完成、是否继续细拆，以及下一轮 parent import bridge staged pass 应进入 `runtime.report_ops_import_pass` 还是先回到 `backend.runtime` 父叶残余判断。

---

## 幻觉检查点

AI 声称 BE-001CY-03 完成时，必须说明:

1. 本批只改写 `src/runtime/event_stream.rs` 与 `src/runtime/evidence_health.rs` 两个文件的 parent wildcard import。
2. parent import bridge 尚未消除，`src/runtime/mod.rs` 未处理。
3. `src/runtime/report_ops.rs` 将进入后续 `runtime.report_ops_import_pass`，`src/runtime/run_guard.rs` 当前仍是 test-only super import。
4. `use super::*` / `super::` 依赖文件数只从 44 降为 42。
5. 下一步只能进入 BE-001CY-04 单叶 closeout。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. 两个目标文件不再通过 `use super::*` 获取父级白箱输入。
2. `cargo check -p quantpilot` 与指定 API 测试通过。
3. `315-runtime.root_entry_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 下一步固定为 BE-001CY-04 `runtime.root_entry_import_pass` 单叶 closeout。
