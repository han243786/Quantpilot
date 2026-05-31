# v4.16.0 runtime.mutation.ai_proposal.approval_persistence_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EX-02
> 基线: `435-runtime.mutation.ai_proposal.approval_persistence_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.ai_proposal.approval_persistence_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_persistence_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EX-03 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EX-02 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | single-file import rewrite / approval persistence freeze / no release transition | 固定抽离边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_persistence_import_pass` | approval persistence import pocket |
| 模块树 | `runtime.mutation.ai_proposal.approval_persistence_import_pass` | 抽离方案 |

---

## 方案结论

BE-001EX-03 只能改写 `src/runtime/mutation/ai_proposal/approval_persistence.rs` 顶部 import，禁止改函数体、可见性、store path、atomic write、not_found 映射、decode error 映射或 sibling owner。

```text
runtime.mutation.ai_proposal.approval_persistence_import_pass
root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_persistence_import_pass
approval_persistence_import_pass plan_frozen
single_file_approval_persistence_import_rewrite
next_step: BE-001EX-03 extraction record
```

---

## 允许改动

唯一允许的 Rust 改动:

```diff
-use super::*;
+use crate::{internal_error, json_bad_request, RuntimeApprovalRecord};
+use axum::http::StatusCode;
+use std::path::Path as FsPath;
+use tokio::fs;
```

允许 cargo fmt 对 import 分组做机械格式化。

---

## 不允许改动

BE-001EX-03 不得修改:

```text
persist_approval body
load_approval_from_disk body
pub(super) visibility
store_dir.join format
approval_id file name format
fs::create_dir_all call
crate::runtime_persistence::atomic_write_json call
fs::read call
json_bad_request not_found code
not_found Chinese message
serde_json::from_slice call
internal_error(anyhow::anyhow!("{}", error)) mapping
```

---

## 等价守卫

必须保持:

```text
no_approval_persistence_rewrite
no_atomic_write_rewrite
no_load_not_found_mapping_rewrite
no_decode_error_mapping_rewrite
no_store_path_rewrite
no_visibility_rewrite
no_sibling_owner_migration
old_three_leaf_pause_target_cancelled
```

---

## 残余预期

BE-001EX-03 完成后，预期 residual 从:

```text
remaining_runtime_parent_import_bridge_7
remaining_mutation_import_bridge_6
remaining_ai_proposal_import_bridge_6
```

降为:

```text
remaining_runtime_parent_import_bridge_6
remaining_mutation_import_bridge_5
remaining_ai_proposal_import_bridge_5
```

`src/runtime/mod.rs` 与 `src/runtime/mutation/ai_proposal.rs` 仍不在 BE-001EX-03 范围内。

---

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不执行 BE-001EX-03 实际 import rewrite。
3. 不处理其他 ai proposal child import residual。
4. 不处理 parent facade 或 root parent bridge。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling 横向连接。
7. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
git diff --check
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
```

---

## 幻觉检查点

AI 声称 BE-001EX-02 完成时，必须说明:

1. 本批只是 `no code movement` 抽离方案。
2. BE-001EX-03 只能改写 `src/runtime/mutation/ai_proposal/approval_persistence.rs` 顶部 import。
3. 不得宣称 `approval_persistence.rs` 已完成实际抽离。
4. 不得宣称 ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `436-runtime.mutation.ai_proposal.approval_persistence_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001EX-03 的允许改动被限制为单文件 import rewrite。
3. BE-001EX-03 成为唯一下一步。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
