# v4.16.0 runtime.mutation.ai_proposal.approval_persistence 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BJ-04  
> 基线: `210-runtime.mutation.ai_proposal.approval_persistence单子叶等价基线.md`、`211-runtime.mutation.ai_proposal.approval_persistence抽离方案.md`、`212-runtime.mutation.ai_proposal.approval_persistence抽离记录.md`、`src/runtime/mutation/ai_proposal/approval_persistence.rs`  
> 判定: `runtime.mutation.ai_proposal.approval_persistence` 单叶 closeout 完成，设置 `stop_split: true`。`persist_approval` 与 `load_approval_from_disk` 共同构成同一 approval record persistence owner；继续拆成 read/write 微叶不会产生新的稳定状态 owner、锁 owner、schema owner、route facade 或 runtime persistence owner，只会增加父子接线和治理挂载面。下一步只能进入 BE-001BK-01 `runtime.mutation.ai_proposal` 第六轮父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BJ-04 approval_persistence 单叶 closeout | 收口 |
| 规范矩阵 | stop_split、父子通信、persistence owner、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_persistence` | 白箱 closeout |
| 模块树 | `runtime.mutation.ai_proposal.approval_persistence` | 设置 `stop_split: true` |

---

## closeout 结论

`runtime.mutation.ai_proposal.approval_persistence` 已完成当前范围内的等价基线、抽离方案和实际抽离。

本叶设置:

```text
stop_split: true
```

原因:

- `persist_approval` 与 `load_approval_from_disk` 共享同一 `RuntimeApprovalRecord` store path、JSON 文件命名和 `approval_store_dir` / `FsPath` 输入边界。
- read/write 两侧都围绕 approval record disk fallback 与 atomic write 语义服务，不形成独立状态 owner。
- `fs::create_dir_all`、`atomic_write_json`、`fs::read`、`serde_json::from_slice`、`json_bad_request` / `not_found` 与 `internal_error(anyhow::anyhow)` 是同一 persistence helper contract 的两端。
- 继续拆为 read/write 微叶不会减少依赖，反而会增加父级 import、visibility 和治理索引面。
- runtime persistence owner 仍在 `src/runtime_persistence.rs`，本叶只是 approval record 调用侧 helper owner，不应私有化底层 persistence。

---

## 已落地文件

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
```

父级保留:

```rust
#[path = "ai_proposal/approval_persistence.rs"]
mod approval_persistence;

use approval_persistence::{load_approval_from_disk, persist_approval};
```

child 保持:

```rust
use super::*;
```

---

## 等价确认

已确认:

- `persist_approval` 继续接收 `&FsPath` 与 `&RuntimeApprovalRecord`。
- `persist_approval` 继续先执行 `fs::create_dir_all(store_dir).await?`。
- `persist_approval` 继续按 `approval.approval_id` 生成同名 JSON 路径。
- `persist_approval` 继续调用 `crate::runtime_persistence::atomic_write_json`。
- `load_approval_from_disk` 继续接收 `&FsPath` 与 `approval_id: &str`。
- `load_approval_from_disk` 继续按 `approval_id` 生成同名 JSON 路径。
- `load_approval_from_disk` 继续使用 `fs::read` 和 `serde_json::from_slice`。
- read miss 继续映射 `json_bad_request("not_found", ...)`。
- decode error 继续映射 `internal_error(anyhow::anyhow!("{}", error))`。
- `approval_review` 继续经父级 `use super::*` 访问 helper，不横向 import sibling。

---

## 未迁移边界

本 closeout 不迁移:

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

---

## 验证证据

BE-001BJ-03 实际抽离后已验证:

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

本 closeout 批次为 `no code movement`，提交前继续执行治理门禁。

---

## 下一步

下一步只能进入:

```text
BE-001BK-01 runtime.mutation.ai_proposal 第六轮父叶残余判断
```

该父叶残余判断只能评估 `sandbox_trigger`、`status_transition` 与 proposal create orchestration 等剩余稳定职责，不得回改已 closeout 的 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review` 或 `approval_persistence`。

---

## 幻觉检查点

AI 声称 BE-001BJ-04 完成时，必须说明 `runtime.mutation.ai_proposal.approval_persistence` 已 closeout 并设置 `stop_split: true`，但 `runtime.mutation.ai_proposal` 父叶尚未完成。不得宣称 sandbox_trigger、status_transition、AppState/schema/frontend caller、route facade、runtime persistence owner、release transition 或 Rust backend 重构已完成。

---

## 验收标准

1. `213-runtime.mutation.ai_proposal.approval_persistence单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.approval_persistence` 标记为 `stop_split: true`。
3. 下一步固定为 BE-001BK-01 `runtime.mutation.ai_proposal` 第六轮父叶残余判断。
4. approval_persistence 不再继续细拆，除非未来有新的独立状态/锁/schema/route owner 证据并重新走提案流程。
