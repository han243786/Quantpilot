# v4.16.0 runtime.backtest.record_store_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DE-03
> 基准: `330-runtime.backtest.record_store_import_pass抽离方案.md`
> 目标子叶: `runtime.backtest.record_store_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.record_store_import_pass`
> 代码动作: actual Rust import rewrite
> 下一步: BE-001DE-04 `runtime.backtest.record_store_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DE-03 `runtime.backtest.record_store_import_pass` 实际抽离 | 实际抽离 |
| 规范矩阵 | explicit import rewrite、record store import、parent surface、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.record_store_import_pass` | record_store import 白箱收敛 |
| 模块树 | `runtime.backtest.record_store_import_pass` | 抽离记录 |

---

## 实际改动

本批只改写:

```text
src/runtime/backtest/record_store.rs
```

删除:

```rust
use super::*;
```

新增显式输入:

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

`DiscardRuntimeArtifactResponse` 继续经 `crate::runtime::DiscardRuntimeArtifactResponse` 父级白箱输入，未新增 sibling horizontal link。

---

## 等价结果

- `list_backtests` 行为未变。
- `get_backtest_detail` 行为未变。
- `save_backtest_record` 行为未变。
- `discard_backtest_record` 行为未变。
- route path、method、handler name、status code、error string、response schema 均未改变。
- persistence owner、audit owner、response mapping owner、state owner 与 frontend caller 均未迁移。

计数锚点: runtime parent bridge 依赖文件数从 34 降为 33。

---

## 排除项

- 本批只改 `src/runtime/backtest/record_store.rs` 顶部 import。
- 本批不处理 `src/runtime/backtest/replay.rs`。
- 本批不处理 `src/runtime/backtest/experiment_sweep.rs` 或其内部子文件。
- 本批不处理 `src/runtime/backtest/execution_start.rs` 或其内部子文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
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

AI 声称 BE-001DE-03 完成时，必须说明:

1. 本批次是 actual Rust import rewrite。
2. 只改写 `src/runtime/backtest/record_store.rs`。
3. runtime parent bridge 依赖文件数从 34 降为 33。
4. 下一步只能进入 BE-001DE-04 `runtime.backtest.record_store_import_pass` 单叶 closeout。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已完成、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `331-runtime.backtest.record_store_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/backtest/record_store.rs` 不再包含 `use super::*` 或 `super::`。
3. runtime parent bridge 残余计数为 33。
4. Rust 编译、`api_backtest`、治理门禁、全量树覆盖和 `git diff --check` 均通过。
