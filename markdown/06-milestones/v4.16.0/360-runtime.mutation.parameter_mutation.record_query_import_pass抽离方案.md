# v4.16.0 runtime.mutation.parameter_mutation.record_query_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DR-02
> 基准: `359-runtime.mutation.parameter_mutation.record_query_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.record_query_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.record_query_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DR-03 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DR-02 `runtime.mutation.parameter_mutation.record_query_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | single-file explicit import pass、read-only handler boundary、parent import bridge | 固定实际改写边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.record_query_import_pass` | record query import 方案 |
| 模块树 | `runtime.mutation.parameter_mutation.record_query_import_pass` | 准许下一步实施 |

---

## 采纳方案

BE-001DR-03 只允许改写一个文件:

```text
src/runtime/mutation/parameter_mutation/record_query.rs
single_file_record_query_import_rewrite
```

允许动作只有:

1. 删除该文件顶部的 `use super::*;`。
2. 增加显式 import。
3. 保持函数体、handler signature、可见性、排序、pagination、cache fallback 与 error mapping 不变。

---

## 固定 import 形状

BE-001DR-03 预期改写为:

```rust
use crate::{
    auth, clean_optional_filter, io_error, load_runtime_parameter_mutation_record,
    list_runtime_parameter_mutation_records, paginate, AppState, PaginatedResponse,
    PaginationQuery, RuntimeParameterMutationListQuery, RuntimeParameterMutationRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

实际顺序以 `cargo fmt` 为准。若编译发现缺少输入，只能补充本文件显式 import，不得恢复 wildcard import，也不得改父级 facade 代替本文件 import。

---

## 适配性校验

该方案可实施，因为:

1. `record_query.rs` 不调用 sibling helper，不需要横向连接。
2. 两个 handler 都是读路径，不写 run event、mutation store 或 snapshot。
3. `RuntimeParameterMutationListQuery`、`PaginatedResponse` 和 persistence helpers 已经能从 crate root / runtime parent bridge 显式引入。
4. 改写后预期只减少 1 个 mutation residual，不影响 parent facade 或其他 child。

预期收敛:

```text
expected_parent_import_bridge_22_to_21
expected_mutation_import_bridge_20_to_19
expected_record_query_import_bridge_1_to_0
```

---

## 排除项

- 本批不修改 Rust 代码。
- BE-001DR-03 不得处理 `proposal_creation.rs`。
- BE-001DR-03 不得处理 `transition_lifecycle.rs` 或其 6 个 child。
- BE-001DR-03 不得处理 `parameter_mutation.rs` parent facade。
- BE-001DR-03 不得处理 `ai_proposal`、`src/runtime/mod.rs` 或 test-only `src/runtime/run_guard.rs`。
- BE-001DR-03 不得改变 handler signature、schema、state lock、persistence owner 或 frontend caller。
- BE-001DR-03 不得新增 sibling horizontal link。
- BE-001DR-03 不得启动 release transition。
- 旧的三叶暂停目标继续保持取消: `old_three_leaf_pause_target_cancelled`。

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

BE-001DR-03 实际 import pass 至少补跑:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
```

---

## 幻觉检查点

AI 声称 BE-001DR-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. BE-001DR-03 只允许改写 `src/runtime/mutation/parameter_mutation/record_query.rs` 顶部 import。
3. `record_query.rs` 尚未改写，当前 parent bridge 剩余仍为 total 22、mutation 20。
4. 下一步只能进入 BE-001DR-03 实际抽离记录。
5. `proposal_creation`、`transition_lifecycle`、parent facade、`ai_proposal`、root bridge 与 test-only run_guard 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。
7. 旧的三叶暂停目标仍为取消状态。

不得宣称 record_query import 已改写、parameter mutation import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `360-runtime.mutation.parameter_mutation.record_query_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 下一步固定为 BE-001DR-03 单文件实际抽离。
3. 本方案没有 Rust 代码改动。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
