# v4.16.0 runtime.mutation.ai_proposal.source_governance_identity_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001ER-01
> 基线: `419-runtime.mutation.ai_proposal_import_pass第三轮父叶残余判断.md`
> 目标子叶: `runtime.mutation.ai_proposal.source_governance_identity_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass`
> 真实文件: `src/runtime/mutation/ai_proposal/source_governance_identity.rs`
> 代码动作: no code movement
> 下一步: BE-001ER-02 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | staged explicit import pass / minimum batch / no release transition | 单文件 import pocket |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal.source_governance_identity_import_pass` | source governance 白箱 |
| 模块树 | `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | 新基线 |

---

## 当前事实

BE-001EQ-01 已选择 source governance identity 作为下一枚 ai proposal import pocket:

```text
runtime.mutation.ai_proposal_import_pass stop_split: false
runtime.mutation.ai_proposal.source_governance_identity_import_pass baseline_frozen
old_three_leaf_pause_target_cancelled
```

当前 parent bridge residual:

```text
remaining_parent_import_bridge_11
remaining_mutation_import_bridge_9
remaining_ai_proposal_import_bridge_9
```

本批只冻结 `source_governance_identity.rs`，不改写 Rust import。

---

## 目标文件范围

```text
src/runtime/mutation/ai_proposal/source_governance_identity.rs
```

当前文件顶部仍为:

```rust
use super::*;
```

---

## 白箱 public / helper 面

本基线冻结以下类型与 helper:

```text
RuntimeAiProposalSourceContext
load_runtime_ai_proposal_source_context
runtime_ai_proposal_governance
runtime_ai_proposal_record_id
```

语义边界:

1. `RuntimeAiProposalSourceContext` 必须继续承载 `graph_id`、`event_count`、`current_sequence_no` 与 `governance`。
2. `load_runtime_ai_proposal_source_context` 必须继续按 `RuntimeEvidenceSourceKind::Run` / `Backtest` 分支读取 source record。
3. source context 的 `current_sequence_no` 必须继续使用最后一个 event envelope sequence number，空 events 时回退到 events length。
4. `runtime_ai_proposal_governance` 必须继续从 source governance 投影 capability、deployment、strategy、parameter version 与 AI write policy。
5. `runtime_ai_proposal_record_id` 必须继续使用 canonical JSON SHA-256 digest，并保持 id 格式 `ai_proposal_{created_at_ms}_{digest12}`。

---

## 当前隐式输入面

BE-001ER-02 需要复核，BE-001ER-03 才允许把 `use super::*` 收敛为显式 import。预期输入面至少包括:

```text
auth::UserId
StatusCode
AppState
RuntimeEvidenceSourceKind
RuntimeGovernanceSnapshot
RuntimeAiProposalGovernance
CreateRuntimeAiProposalRequest
load_run_record_from_state
load_backtest_record_from_state
canonical_json_sha256_digest
internal_error
json!
```

实际实现以 `cargo check -p quantpilot` 为准，不得恢复 wildcard import。

---

## 不进入范围

本批不处理:

1. 不修改 `src/runtime/mutation/ai_proposal/source_governance_identity.rs`。
2. 不处理 `static_check.rs`、`event_lifecycle.rs`、`approval_persistence.rs`、`approval_review.rs`、`sandbox_trigger.rs`、`proposal_creation.rs`、`status_transition.rs` 或 parent facade。
3. 不处理 `src/runtime/mod.rs` root parent bridge。
4. 不处理 test-only `src/runtime/run_guard.rs`。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling horizontal link。
7. 不启动 release transition。

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

AI 声称 BE-001ER-01 完成时，必须说明:

1. 本批只是 `no code movement` 单子叶等价基线。
2. 目标文件为 `src/runtime/mutation/ai_proposal/source_governance_identity.rs`。
3. `use super::*` 尚未改写。
4. 下一步只能进入 BE-001ER-02 抽离方案。
5. 不得宣称 source governance identity import 已改写、ai proposal import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `420-runtime.mutation.ai_proposal.source_governance_identity_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 source context、governance projection 与 deterministic record id 的输入输出。
3. 下一步固定为 BE-001ER-02 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
