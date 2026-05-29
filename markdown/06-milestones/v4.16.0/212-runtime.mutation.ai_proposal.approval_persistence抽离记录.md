# v4.16.0 runtime.mutation.ai_proposal.approval_persistence 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BJ-03  
> 基线: `210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md`、`211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: `runtime.mutation.ai_proposal.approval_persistence` 第一轮实际抽离完成。`persist_approval` 与 `load_approval_from_disk` 已迁入 child 文件；父级通过 path-attributed child 和私有 helper import 维持原调用面。sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 和 release transition guard 均未迁移。下一步只能进入 BE-001BJ-04 单叶 closeout。  
> 代码动作: code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BJ-03 approval_persistence 实际抽离 | 物理抽离 |
| 规范矩阵 | helper owner、父级受控 import、非横向连接 | 约束执行 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_persistence` | child 文件落地 |
| 模块树 | `runtime.mutation.ai_proposal.approval_persistence` | 白箱抽离完成 |

---

## 文件变更

新增:

```text
src/runtime/mutation/ai_proposal/approval_persistence.rs
```

父级 `src/runtime/mutation/ai_proposal.rs` 新增:

```rust
#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;

use approval_persistence::{load_approval_from_disk, persist_approval};
```

child 固定:

```rust
use super::*;
```

---

## 实际迁移清单

已从父级迁入 child:

- `persist_approval`
- `load_approval_from_disk`

迁移后函数保持 `pub(super) async fn` 可见性，只向父级 `runtime.mutation.ai_proposal` 暴露。`approval_review` 仍经 `use super::*` 通过父级受控 helper 名称访问，不直接 import `approval_persistence` sibling。

---

## 行为等价说明

`persist_approval` 行为保持不变:

1. 输入仍为 `&FsPath` 与 `&RuntimeApprovalRecord`。
2. 仍先执行 `fs::create_dir_all(store_dir).await?`。
3. 仍使用 `store_dir.join(format!("{}.json", approval.approval_id))` 生成文件路径。
4. 仍通过 `crate::runtime_persistence::atomic_write_json(&file_path, approval).await` 写入。

`load_approval_from_disk` 行为保持不变:

1. 输入仍为 `&FsPath` 与 `approval_id: &str`。
2. 仍使用 `store_dir.join(format!("{}.json", approval_id))` 生成文件路径。
3. 仍通过 `fs::read(&file_path).await` 读取。
4. read miss 仍映射为 `json_bad_request("not_found", format!("审批单 '{}' 不存在", approval_id))`。
5. decode 仍使用 `serde_json::from_slice(&json)`。
6. decode error 仍映射为 `internal_error(anyhow::anyhow!("{}", error))`。

---

## 调用面保持

| 调用点 | 等价结果 |
| --- | --- |
| `create_runtime_ai_proposal` | 继续调用 `persist_approval(&state.approval_store_dir, &approval)` |
| `approval_review` | 继续经父级 `use super::*` 使用 `load_approval_from_disk` 与 `persist_approval` |
| sandbox background task | 继续调用 `persist_approval(&state_clone.approval_store_dir, &approval)` |

`src/backend/runtime/routes/mutation.rs`、`src/runtime/mod.rs`、`AppState`、schema owner、frontend caller、runtime persistence owner 与 release transition guard 均未改变。

---

## 非目标边界

BE-001BJ-03 未迁移或修改:

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

已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query` 与 `approval_review` 未回收、未重拆。

---

## 验证计划

实际抽离批次必须运行:

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

---

## 下一步

下一步只能进入:

```text
BE-001BJ-04 runtime.mutation.ai_proposal.approval_persistence 单叶 closeout
```

该 closeout 只能判断本叶是否停止细分，不得继续迁移 sandbox_trigger、status_transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BJ-03 完成时，必须说明当前只完成 `runtime.mutation.ai_proposal.approval_persistence` 第一轮实际抽离，尚未完成单叶 closeout。不得宣称 sandbox_trigger、status_transition、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变，也不得宣称 Rust backend 重构完成。

---

## 验收标准

1. `src/runtime/mutation/ai_proposal/approval_persistence.rs` 存在，并承接 `persist_approval` 与 `load_approval_from_disk`。
2. 父级 `src/runtime/mutation/ai_proposal.rs` 只保留 path-attributed child 与私有 helper import。
3. `approval_review` 仍经父级受控 helper 名称访问，不横向 import sibling。
4. `212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
5. 下一步固定为 BE-001BJ-04 单叶 closeout。
