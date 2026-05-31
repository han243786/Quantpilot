# v4.16.0 runtime.backtest.record_store_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DE-01
> 基准: `328-runtime.backtest_import_pass抽离方案.md`
> 目标子叶: `runtime.backtest.record_store_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.record_store_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DE-02 `runtime.backtest.record_store_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DE-01 `runtime.backtest.record_store_import_pass` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | explicit import pass、record store import、parent surface、release transition guard | 等价冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.record_store_import_pass` | backtest record store import 白箱 |
| 模块树 | `runtime.backtest.record_store_import_pass` | 新增基线 |

---

## 当前文件

```text
src/runtime/backtest/record_store.rs
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
| `list_backtests` | `State<AppState>`、`Query<PaginationQuery>` | `Json<PaginatedResponse<BacktestListItem>>` | saved backtest records 读取、降序排序、pagination 不变 |
| `get_backtest_detail` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<BacktestDetailResponse>` | scoped lookup 与 detail projection 不变 |
| `save_backtest_record` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<BacktestDetailResponse>` | persistence、transient deletion、memory update 与 audit 写入不变 |
| `discard_backtest_record` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<DiscardRuntimeArtifactResponse>` | saved record conflict、transient deletion、memory cleanup 与 not found 语义不变 |

---

## 预期显式输入面

BE-001DE-03 实际抽离时，预计将 `use super::*` 收敛为如下父级白箱输入:

```rust
use crate::{
    auth, backtest_detail_response_from_record, backtest_list_item_from_record,
    build_graph_audit_entry, delete_transient_backtest_record, io_error, list_backtest_records,
    load_backtest_record_from_state, paginate, persist_backtest_record, persist_graph_audit_entry,
    runtime::DiscardRuntimeArtifactResponse, sanitize_storage_path_segment, AppState,
    BacktestDetailResponse, BacktestListItem, GraphAuditAction, PaginatedResponse, PaginationQuery,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tokio::fs;
```

该输入面继续通过父级白箱 `crate::runtime::DiscardRuntimeArtifactResponse` 取得 shared DTO，不新增 sibling horizontal link。

---

## 等价保护

1. 不改变 backtest route path、method、handler name 或 response schema。
2. 不改变 `list_backtest_records`、`load_backtest_record_from_state`、`persist_backtest_record` 或 `delete_transient_backtest_record` 的 owner。
3. 不改变 audit 写入 owner: `build_graph_audit_entry`、`persist_graph_audit_entry`、`GraphAuditAction::BacktestCreated`。
4. 不改变 `sanitize_storage_path_segment` 的路径保护。
5. 不迁移 `BacktestDetailResponse`、`BacktestListItem`、`PaginatedResponse` 或 `PaginationQuery` schema owner。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/record_store.rs`。
- 本批不处理 `src/runtime/backtest/replay.rs`。
- 本批不处理 `src/runtime/backtest/experiment_sweep.rs` 或其内部子文件。
- 本批不处理 `src/runtime/backtest/execution_start.rs` 或其内部子文件。
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

AI 声称 BE-001DE-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 目标文件只有 `src/runtime/backtest/record_store.rs`。
3. 本批没有改写 `use super::*`。
4. 下一步只能进入 BE-001DE-02 `runtime.backtest.record_store_import_pass` 抽离方案。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest.record_store_import_pass` 已抽离、`runtime.backtest_import_pass` 已完成或 parent import bridge 已清除。

---

## 验收标准

1. `329-runtime.backtest.record_store_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 `record_store.rs` 的 4 个 public 方法与预期显式输入面。
3. 下一步固定为 BE-001DE-02 `runtime.backtest.record_store_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
