# v4.16.0 runtime.mutation.ai_proposal.approval_persistence 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BJ-02  
> 基线: `210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: 固定 BE-001BJ-03 的实际抽离方案。下一步只允许创建 `src/runtime/mutation/ai_proposal/approval_persistence.rs`，迁移 `persist_approval` 与 `load_approval_from_disk` 两个 approval record persistence helper，并通过父级 path-attributed child 与受控 helper import 保持调用面不变。当前 `no code movement`。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BJ-02 approval_persistence 抽离方案 | 方案固化 |
| 规范矩阵 | 父子通信、helper import、visibility、回退点 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_persistence` | 抽离路径 |
| 模块树 | `runtime.mutation.ai_proposal.approval_persistence` | 计划物理抽离 |

---

## 目标文件与父级接线

BE-001BJ-03 允许创建:

```text
src/runtime/mutation/ai_proposal/approval_persistence.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 允许新增:

```rust
#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;

use approval_persistence::{load_approval_from_disk, persist_approval};
```

child 文件固定使用:

```rust
use super::*;
```

迁移后 `approval_review` 仍只能通过父级 `use super::*` 使用 `persist_approval` 与 `load_approval_from_disk`，不得直接 import `approval_persistence` sibling。

---

## 允许迁移清单

BE-001BJ-03 只允许把下列函数从父级移动到 child:

- `persist_approval`
- `load_approval_from_disk`

函数迁移后固定使用 `pub(super) async fn` 可见性，只向父级 `runtime.mutation.ai_proposal` 暴露，不向 route facade、`src/runtime/mod.rs` 或 sibling child 公开。

---

## 行为不变要求

BE-001BJ-03 必须保持以下行为等价:

1. `persist_approval` 继续接收 `&FsPath` 和 `&RuntimeApprovalRecord`。
2. `persist_approval` 继续先执行 `fs::create_dir_all(store_dir).await?`。
3. `persist_approval` 继续使用 `store_dir.join(format!("{}.json", approval.approval_id))` 构造路径。
4. `persist_approval` 继续调用 `crate::runtime_persistence::atomic_write_json(&file_path, approval).await`，不得私有化或复制 runtime persistence owner。
5. `load_approval_from_disk` 继续接收 `&FsPath` 和 `approval_id: &str`。
6. `load_approval_from_disk` 继续使用 `store_dir.join(format!("{}.json", approval_id))` 构造路径。
7. `load_approval_from_disk` 继续使用 `fs::read(&file_path).await` 读取。
8. read miss 继续映射为 `json_bad_request("not_found", format!("审批单 '{}' 不存在", approval_id))`。
9. decode 继续使用 `serde_json::from_slice(&json)`。
10. decode error 继续映射为 `internal_error(anyhow::anyhow!("{}", error))`。

---

## 调用面保持

迁移后以下调用点必须保持无需改业务语义:

| 调用点 | 当前调用 | 迁移后调用要求 |
| --- | --- | --- |
| `create_runtime_ai_proposal` | `persist_approval(&state.approval_store_dir, &approval)` | 继续通过父级 import 解析 |
| `approval_review` | `load_approval_from_disk` / `persist_approval` | 继续经 `use super::*` 使用父级受控 helper |
| sandbox background task | `persist_approval(&state_clone.approval_store_dir, &approval)` | 继续通过父级 import 解析 |

不允许新增 route facade 接线，不允许改变 `src/runtime/mod.rs` re-export，不允许改变 `AppState`、schema owner、frontend caller、runtime persistence owner 或 release transition guard。

---

## 非目标边界

BE-001BJ-02 不移动代码。BE-001BJ-03 也不得迁移或修改:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `sandbox_trigger`
- `status_transition`
- `approval_review`
- `AppState`
- schema owner
- frontend caller
- route facade
- runtime persistence owner
- release transition guard

不得回收或重拆 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query` 或 `approval_review` 已 closeout 子叶。

---

## 回退点

如果 BE-001BJ-03 编译或测试失败，回退方式固定为:

1. 删除 `src/runtime/mutation/ai_proposal/approval_persistence.rs`。
2. 从 child 恢复 `persist_approval` 与 `load_approval_from_disk` 到 `src/runtime/mutation/ai_proposal.rs` 原位置。
3. 删除父级 `mod approval_persistence` 与 `use approval_persistence::{...};`。
4. 不回退 closed child: `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`。
5. 不改变 AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 测试策略

BE-001BJ-03 实际抽离必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

本方案批次 `no code movement`，只需要治理门禁。

---

## 下一步

下一步只能进入:

```text
BE-001BJ-03 runtime.mutation.ai_proposal.approval_persistence 实际抽离
```

该步骤才允许创建目标文件并迁移允许清单内两个 helper。

---

## 幻觉检查点

AI 声称 BE-001BJ-02 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.approval_persistence` 抽离方案，尚未创建目标文件，也尚未迁移 `persist_approval` 或 `load_approval_from_disk`。不得宣称 approval_persistence 已抽离、sandbox_trigger 已迁移、status_transition 已迁移、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001BJ-03 的目标文件、父级声明、helper import、`pub(super) async fn` visibility、迁移清单、非目标和回退点已固定。
3. 下一步固定为 BE-001BJ-03 实际抽离。
4. 本批不产生代码变更，不回收 closed child，不启动 release transition。
