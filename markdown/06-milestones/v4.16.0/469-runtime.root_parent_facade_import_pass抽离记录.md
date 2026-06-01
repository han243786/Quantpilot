# v4.16.0 runtime.root_parent_facade_import_pass 抽离记录
> 版本类型: MINOR architecture / code
> 执行档位: 标准
> 批次: BE-001FL-03
> 基线: `468-runtime.root_parent_facade_import_pass抽离方案.md`
> 目标子叶: `runtime.root_parent_facade_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass`
> 代码动作: single_file_root_parent_facade_import_pass
> 下一步: BE-001FL-04 `runtime.root_parent_facade_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001FL-03 `runtime.root_parent_facade_import_pass` 实际抽离 | 单文件实际抽离 |
| 规范矩阵 | explicit import pass / root facade import cleanup / parent bridge removal / release transition guard | 标准档位 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.root_parent_facade_import_pass` | root residual 清除 |
| 模块树 | `runtime.root_parent_facade_import_pass` | `root_parent_facade_import_pass extraction_complete` |

---

## 实际改动

本批只修改:

```text
src/runtime/mod.rs
```

已删除 `src/runtime/mod.rs` 尾部两个 unused root import residual:

```diff
-use super::*;
-use axum::extract::Query;
```

未新增替代 import。`module declaration surface`、`public re-export surface`、`private helper bridge surface`、`query_support parent surface` 与 `response_support parent surface` 均保持 BE-001FL-01 基线状态。

---

## 不变项

本批没有修改:

1. child module declaration。
2. `#[path = "..."]` 映射。
3. public handler owner。
4. `pub(crate) use` re-export surface。
5. private helper bridge surface。
6. route facade、schema owner、state owner、frontend caller。
7. test-local residual。
8. release transition 或 sibling horizontal link。

---

## 预期状态

生产级 root residual 应清零:

```text
remaining_runtime_parent_import_bridge_0
remaining_root_parent_import_bridge_0
```

下列检查应无输出:

```powershell
rg --line-number "^use super::\*;|use axum::extract::Query" src\runtime\mod.rs
```

test-local residual 不纳入本批:

```text
src/runtime/run_guard.rs
src/runtime/mutation/ai_proposal/static_check.rs
```

---

## 已执行验证

本批已执行并通过:

```powershell
git diff --check
rg --line-number "^use super::\*;|use axum::extract::Query" src\runtime\mod.rs
rg --line-number "^use super::\*;" src\runtime -g "*.rs"
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

`cargo check -p quantpilot` 已不再报告 `src/runtime/mod.rs` 的 unused root import warnings。

---

## 验证要求

本批提交前至少执行:

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

AI 声称 BE-001FL-03 完成时，必须说明:

1. 本批只做了 `src/runtime/mod.rs` 单文件 root import cleanup。
2. 删除了 `use super::*` 与 `use axum::extract::Query` 两行。
3. 未新增替代 import。
4. 未改 child module、handler、re-export、private helper bridge、route facade、schema、state、frontend 或 release transition。
5. 下一步只能进入 BE-001FL-04 单叶 closeout，不能直接宣称 backend.runtime 或 Rust 重构完成。
6. `old_three_leaf_pause_target_cancelled` 保持取消状态。
7. `progress_report_instruction_discarded` 保持丢弃状态。

---

## 验收标准

1. `469-runtime.root_parent_facade_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `root_parent_facade_import_pass extraction_complete` 已记录。
3. `src/runtime/mod.rs` 已无生产级 root parent import residual。
4. 治理门禁、全量树覆盖、UTF-8、Rust fmt/check、代表性 runtime API 测试和 `git diff --check` 均通过。
