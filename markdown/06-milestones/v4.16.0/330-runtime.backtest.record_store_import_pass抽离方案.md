# v4.16.0 runtime.backtest.record_store_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 轻量
> 批次: BE-001DE-02
> 基准: `329-runtime.backtest.record_store_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.backtest.record_store_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.record_store_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DE-03 actual Rust import rewrite

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DE-02 `runtime.backtest.record_store_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | explicit import rewrite、record store import、parent surface、release transition guard | 最小实际批次 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.record_store_import_pass` | record_store import 收敛方案 |
| 模块树 | `runtime.backtest.record_store_import_pass` | 抽离方案 |

---

## 适配性结论

BE-001DE-03 可以进入实际 Rust import rewrite，范围仅限:

```text
src/runtime/backtest/record_store.rs
```

本文件是 direct route singleton，没有内部子模块；4 个 public handler 共用同一组 persistence、response、audit 和 path safety helper，适合同批收敛。

---

## 实际改写方案

删除:

```rust
use super::*;
```

新增:

```rust
use crate::{
    auth, backtest_detail_response_from_record, backtest_list_item_from_record,
    build_graph_audit_entry, delete_transient_backtest_record, io_error, list_backtest_records,
    load_backtest_record_from_state, paginate, persist_backtest_record,
    persist_graph_audit_entry, runtime::DiscardRuntimeArtifactResponse,
    sanitize_storage_path_segment, AppState, BacktestDetailResponse, BacktestListItem,
    GraphAuditAction, PaginatedResponse, PaginationQuery,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tokio::fs;
```

`DiscardRuntimeArtifactResponse` 继续通过 `crate::runtime::DiscardRuntimeArtifactResponse` 父级白箱输入，保持父子通信规则；不新增 sibling horizontal link。

---

## 等价要求

BE-001DE-03 不得改变以下行为:

1. `list_backtests` 的 saved records 读取、降序排序和 pagination。
2. `get_backtest_detail` 的 scoped lookup 和 response projection。
3. `save_backtest_record` 的 persistence、transient deletion、memory update、audit 写入和 response projection。
4. `discard_backtest_record` 的 saved record conflict、transient deletion、memory cleanup、not found 和 discard response schema。
5. path safety: `sanitize_storage_path_segment` 调用必须保留。

---

## 排除项

- BE-001DE-03 只改 `src/runtime/backtest/record_store.rs` 顶部 import。
- 不处理 `src/runtime/backtest/replay.rs`。
- 不处理 `src/runtime/backtest/experiment_sweep.rs` 或其内部子文件。
- 不处理 `src/runtime/backtest/execution_start.rs` 或其内部子文件。
- 不处理 `src/runtime/mod.rs` root parent bridge。
- 不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 不迁移 route registration、schema、persistence owner、audit owner、response mapping owner、state owner 或 frontend caller。
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

BE-001DE-03 实际抽离后至少执行:

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

AI 声称 BE-001DE-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. 下一步 BE-001DE-03 只允许改写 `src/runtime/backtest/record_store.rs` 顶部 import。
3. 尚未改写 `use super::*`。
4. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest.record_store_import_pass` 已抽离、`runtime.backtest_import_pass` 已完成或 parent import bridge 已清除。

---

## 验收标准

1. `330-runtime.backtest.record_store_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001DE-03 只改 `src/runtime/backtest/record_store.rs` 顶部 import。
3. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
