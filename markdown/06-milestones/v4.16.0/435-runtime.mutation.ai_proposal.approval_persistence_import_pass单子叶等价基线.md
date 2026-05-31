# v4.16.0 runtime.mutation.ai_proposal.approval_persistence_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EX-01
> 父叶判定: `434-runtime.mutation.ai_proposal_import_pass第六轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.approval_persistence_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_persistence_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EX-02 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EX-01 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | explicit import pass / approval persistence freeze / no release transition | 冻结审批持久化输入面 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.approval_persistence_import_pass` | approval persistence 白箱 |
| 模块树 | `runtime.mutation.ai_proposal.approval_persistence_import_pass` | 新基线 |

---

## 基线结论

本批只冻结 `src/runtime/mutation/ai_proposal/approval_persistence.rs` 的当前等价边界，不改 Rust 代码。

```text
runtime.mutation.ai_proposal.approval_persistence_import_pass baseline_frozen
runtime.mutation.ai_proposal.approval_persistence_import_pass no_code_movement
src/runtime/mutation/ai_proposal/approval_persistence.rs
current_parent_import_bridge: use super::*
next_step: BE-001EX-02 extraction plan
```

`approval_persistence.rs` 是 ai proposal approval record 的持久化白箱，负责审批单落盘和按 approval id 读取审批单。它不是 route facade，也不是 sandbox verification owner。

---

## 白箱节点

| 项 | 当前边界 |
| --- | --- |
| 输入 | approval store dir、`RuntimeApprovalRecord`、approval id |
| 输出 | `std::io::Result<()>`、`Result<RuntimeApprovalRecord, (StatusCode, String)>` |
| 处理者 | `persist_approval`、`load_approval_from_disk` |
| 调用方 | `proposal_creation.rs`、`approval_review.rs`、`sandbox_trigger.rs`、ai proposal parent facade |
| 禁止事项 | 不改 store path、不改 atomic write、不改 not_found 映射、不改 decode error 映射、不新增 sibling 横向连接 |

---

## 当前 public / 可见入口

本子叶对父模块暴露 2 个 `pub(super)` helper:

```text
persist_approval
load_approval_from_disk
```

---

## 当前隐式输入面

当前文件顶部仍为:

```rust
use super::*;
```

BE-001EX-03 预期只把该 parent wildcard import 收敛为显式输入面。预期输入面包括:

```rust
use crate::{internal_error, json_bad_request, RuntimeApprovalRecord};
use axum::http::StatusCode;
use std::path::Path as FsPath;
use tokio::fs;
```

该预期仅作为输入面基线，真正代码改写必须等 BE-001EX-03。

---

## 等价边界

### Approval persist

必须保持 `persist_approval` 的副作用顺序:

```text
fs::create_dir_all(store_dir).await
store_dir.join(format!("{}.json", approval.approval_id))
crate::runtime_persistence::atomic_write_json(&file_path, approval).await
```

不得改变文件名格式、目录创建顺序、atomic write owner 或返回类型。

### Approval load

必须保持 `load_approval_from_disk` 的读取规则:

```text
store_dir.join(format!("{}.json", approval_id))
fs::read(&file_path).await
missing file -> json_bad_request("not_found", format!("审批单 '{}' 不存在", approval_id))
serde_json::from_slice(&json)
decode error -> internal_error(anyhow::anyhow!("{}", error))
```

不得改变 not_found code、错误消息、decode error 映射或 StatusCode 返回面。

---

## 不变量

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

## 排除项

本批不处理:

1. 不改 Rust 代码。
2. 不删除 `use super::*`。
3. 不改函数体、可见性、错误 code、错误消息或 atomic write owner。
4. 不处理其他 ai proposal child import residual。
5. 不处理 `src/runtime/mutation/ai_proposal.rs` parent facade。
6. 不处理 `src/runtime/mod.rs` root parent bridge。
7. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
8. 不新增 sibling 横向连接。
9. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

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

AI 声称 BE-001EX-01 完成时，必须说明:

1. 本批只是 `no code movement` 单子叶等价基线。
2. `approval_persistence.rs` 仍未实际删除 `use super::*`。
3. 下一步只能进入 BE-001EX-02 `runtime.mutation.ai_proposal.approval_persistence_import_pass` 抽离方案。
4. 不得宣称 approval_persistence import、ai proposal import、mutation import、parent import bridge、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `435-runtime.mutation.ai_proposal.approval_persistence_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `approval_persistence.rs` 白箱输入、输出、处理者、调用方和禁止事项已冻结。
3. 下一步固定为 BE-001EX-02 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
