# v4.16.0 runtime.mutation.ai_proposal_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EO-02
> 基线: `413-runtime.mutation.ai_proposal_import_pass单子叶等价基线.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 代码动作: no code movement
> 下一步: BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EO-02 `runtime.mutation.ai_proposal_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge / minimum batch / explicit import pass / release transition guard | 拒绝 10 文件整批 rewrite |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | 下一 pocket 选择 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 当前事实

BE-001EO-01 已冻结 10 个 ai proposal residual 文件:

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/record_query.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
```

当前 parent bridge 剩余仍为:

```text
root 1
run 0
backtest 0
mutation 10
test-only 1
total 12
remaining_parent_import_bridge_12
remaining_mutation_import_bridge_10
remaining_ai_proposal_import_bridge_10
```

---

## 适配性校验

本父叶存在多类风险:

1. `proposal_creation.rs` 会创建 AI proposal、生成 approval、追加 run event、写 proposal store、触发 sandbox verification。
2. `approval_review.rs` 会处理 approve / reject / claim，涉及 approval lock order、reviewer vectors、lifecycle 与 proposal status transition。
3. `sandbox_trigger.rs` 会触发后台重试、更新 sandbox report URL 与 failure lifecycle。
4. `static_check.rs` 会校验 model identity、hash identity、config-domain binding 与 v4 source-kind contract。
5. parent facade `ai_proposal.rs` 当前仍承担 child module declaration、re-export 与 helper bridge 输入面。

因此本批拒绝 10 文件整批改写:

```text
reject_ai_proposal_bulk_rewrite_10_files
runtime.mutation.ai_proposal_import_pass stop_split: false
old_three_leaf_pause_target_cancelled
```

---

## 候选比较

| 候选 | 文件范围 | 风险 | 判定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.record_query_import_pass` | `record_query.rs` | 只读 list/detail 与 state/disk fallback，依赖面窄 | 采纳 |
| `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | `source_governance_identity.rs` | source evidence 与 canonical id，依赖 run/backtest source | 延后 |
| `runtime.mutation.ai_proposal.static_check_import_pass` | `static_check.rs` | static-check contract 与 v4 domain binding，测试面较重 | 延后 |
| `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | `event_lifecycle.rs` | event envelope 与 persistence transition | 延后 |
| `runtime.mutation.ai_proposal.approval_persistence_import_pass` | `approval_persistence.rs` | approval disk read/write，风险中低 | 延后 |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | approve / reject / claim 路由，lock order 与状态迁移重 | 延后 |
| `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | `sandbox_trigger.rs` | background retry / sandbox gate / report URL | 延后 |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create path 依赖多个 sibling helper 和 approval/sandbox 副作用 | 延后 |
| `runtime.mutation.ai_proposal.status_transition_import_pass` | `status_transition.rs` | 状态迁移 guard，依赖 approval review | 延后 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | parent facade 当前仍是 child import bridge | 最后处理 |

---

## 采纳方案

下一步固定为:

```text
BE-001EP-01 runtime.mutation.ai_proposal.record_query_import_pass 单子叶等价基线
```

理由:

1. `record_query.rs` 只包含 `load_runtime_ai_proposal_for_user`、`list_runtime_ai_proposals` 与 `get_runtime_ai_proposal_detail`。
2. 它不改变 proposal 创建、approval review、sandbox trigger、static-check、source governance、event lifecycle 或 status transition 语义。
3. 它能在不触碰 parent facade 的情况下先消除一个 child 的 `use super::*`。
4. 它为后续处理 approval、sandbox、static-check 与 proposal creation 提供低风险 import 模板。

后续 BE-001EP-03 的实际改写范围必须只允许:

```text
src/runtime/mutation/ai_proposal/record_query.rs
```

---

## 预期显式输入面

BE-001EP-01 需要冻结，BE-001EP-02 需要复核，BE-001EP-03 才允许使用类似输入面:

```rust
use crate::{
    auth, clean_optional_filter, io_error, load_runtime_ai_proposal_record,
    list_runtime_ai_proposal_records, AppState, RuntimeAiProposalListQuery,
    RuntimeAiProposalRecord,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
```

实际实现时以 `cargo fmt` 和编译结果为准；不得为了减少 import 而恢复 wildcard import。

---

## 等价边界

后续 record query import pass 必须保持:

1. `list_runtime_ai_proposals` 与 `get_runtime_ai_proposal_detail` handler signature 不变。
2. `load_runtime_ai_proposal_for_user` 的 state cache 优先、disk fallback 与 `auth::scoped_key` 语义不变。
3. list 过滤 `source_kind`、`source_id`、`status` 与排序行为不变。
4. 不改变 `RuntimeAiProposalListQuery` 或 `RuntimeAiProposalRecord` schema。
5. 不触碰 proposal creation、approval review、sandbox trigger、static-check、source governance、event lifecycle 或 status transition owner。
6. 不新增 sibling horizontal link，不启动 release transition。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `record_query.rs` import。
- 本批不处理 `proposal_creation.rs`。
- 本批不处理 `approval_review.rs`、`approval_persistence.rs`、`sandbox_trigger.rs`、`static_check.rs`、`source_governance_identity.rs`、`event_lifecycle.rs` 或 `status_transition.rs`。
- 本批不处理 `ai_proposal.rs` parent facade。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001EO-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 本批拒绝 `reject_ai_proposal_bulk_rewrite_10_files`。
4. 下一步只能进入 BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线。
5. `record_query.rs` 尚未改写，当前 parent bridge 剩余仍为 total 12、mutation 10。
6. approval、sandbox、static-check、source governance、event lifecycle、proposal creation、status transition、parent facade、root bridge 与 test-only run_guard 尚未处理。
7. release transition 未启动，未新增 sibling horizontal link。
8. 旧的三叶暂停目标仍为取消状态。

不得宣称 ai proposal import 已改写、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `414-runtime.mutation.ai_proposal_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶设置 `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 10 文件整批 rewrite 被明确拒绝。
4. 下一步固定为 BE-001EP-01 `runtime.mutation.ai_proposal.record_query_import_pass` 单子叶等价基线。
5. 治理门禁、全量树覆盖和 Rust 验证均通过。
