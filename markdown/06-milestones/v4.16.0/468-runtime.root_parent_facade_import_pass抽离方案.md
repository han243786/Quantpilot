# v4.16.0 runtime.root_parent_facade_import_pass 抽离方案
> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001FL-02
> 基线: `467-runtime.root_parent_facade_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.root_parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass`
> 代码动作: no code movement
> 下一步: BE-001FL-03 `runtime.root_parent_facade_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FL-02 `runtime.root_parent_facade_import_pass` 抽离方案 | 方案冻结 |
| 规范矩阵 | explicit import pass / root facade import cleanup / no sibling horizontal link / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass` | 单文件 import rewrite 方案 |
| 模块树 | `runtime.root_parent_facade_import_pass` | `root_parent_facade_import_pass plan_frozen` |

---

## 当前判断

BE-001FL-01 已冻结 `src/runtime/mod.rs` 的 module declaration、public re-export、private helper bridge、query_support parent surface 与 response_support parent surface。当前 `cargo check -p quantpilot` 只对两个根 import residual 发出 warning:

```rust
use super::*;
use axum::extract::Query;
```

`use super::*` 不再承载有效 root parent input；`use axum::extract::Query` 也未被 `src/runtime/mod.rs` 使用。下一批可以执行 `single_file_root_parent_facade_import_pass`，但只能在 BE-001FL-03 中发生。

---

## BE-001FL-03 固定改动

`single_file_root_parent_facade_import_pass` 只允许修改一个文件:

```text
src/runtime/mod.rs
```

固定改动:

```diff
-use super::*;
-use axum::extract::Query;
```

不新增替代 import。原因:

1. child module declaration 已在本文件顶部显式列出。
2. public re-export 已由 child module 路径显式提供。
3. private helper bridge 已由 child module 路径显式提供。
4. query_support / response_support parent surface 已由 sibling module 显式提供。
5. 当前两个 import 已被编译器判定为 unused。

---

## 禁止项

BE-001FL-03 不得:

1. 不得修改任何 `src/runtime/**/*.rs` child 文件。
2. 不得改动 `#[path = "..."]` 或 `mod` 声明。
3. 不得迁移 public handler owner。
4. 不得改动 `pub(crate) use` re-export surface。
5. 不得改动 private helper bridge surface。
6. 不得改动 route facade、schema owner、state owner、frontend caller。
7. 不得新增 sibling horizontal link。
8. 不得启动 release transition。
9. 不得把 test-local residual 混入本批。

---

## 预期结果

BE-001FL-03 完成后，以下检查应成立:

```powershell
rg --line-number "^use super::\*;|use axum::extract::Query" src\runtime\mod.rs
```

无输出。

生产级 root residual 应从:

```text
remaining_runtime_parent_import_bridge_1
remaining_root_parent_import_bridge_1
```

降为:

```text
remaining_runtime_parent_import_bridge_0
remaining_root_parent_import_bridge_0
```

test-local residual 仍不纳入本批:

```text
src/runtime/run_guard.rs
src/runtime/mutation/ai_proposal/static_check.rs
```

---

## 验证要求

BE-001FL-02 提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
```

BE-001FL-03 实际抽离后至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001FL-02 完成时，必须说明:

1. 本批仍是 `no code movement` 抽离方案。
2. `src/runtime/mod.rs` 尚未改写。
3. BE-001FL-03 只能删除 `use super::*` 与 `use axum::extract::Query` 两行。
4. 不允许新增替代 import。
5. 不允许改 child module、handler、re-export、private helper bridge、route facade、schema、state、frontend 或 release transition。
6. `old_three_leaf_pause_target_cancelled` 保持取消状态。
7. `progress_report_instruction_discarded` 保持丢弃状态。

不得宣称 runtime parent bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `468-runtime.root_parent_facade_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `root_parent_facade_import_pass plan_frozen` 已记录。
3. 下一步固定为 BE-001FL-03 实际抽离记录。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check 和 `git diff --check` 均通过。
