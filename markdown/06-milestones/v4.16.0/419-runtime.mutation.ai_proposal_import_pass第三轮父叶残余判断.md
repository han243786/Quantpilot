# v4.16.0 runtime.mutation.ai_proposal_import_pass 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001EQ-01
> 基线: `418-runtime.mutation.ai_proposal.record_query_import_pass单叶closeout.md`
> 目标父叶: `runtime.mutation.ai_proposal_import_pass`
> 判定: `runtime.mutation.ai_proposal_import_pass stop_split: false`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass`
> 代码动作: no code movement
> 下一步: BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001EQ-01 `runtime.mutation.ai_proposal_import_pass` 父叶残余判断 | 适配性校验 |
| 规范矩阵 | parent import bridge / recursive residual judgment / no release transition | 父叶继续拆分 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.ai_proposal_import_pass` | 下一 pocket 选择 |
| 模块树 | `runtime.mutation.ai_proposal_import_pass` | `stop_split: false` |

---

## 当前事实

`runtime.mutation.ai_proposal.record_query_import_pass` 已完成 closeout:

```text
runtime.mutation.ai_proposal.record_query_import_pass stop_split: true
runtime.mutation.ai_proposal_import_pass third_parent_residual_judgment
runtime.mutation.ai_proposal_import_pass stop_split: false
old_three_leaf_pause_target_cancelled
```

当前 residual:

```text
remaining_parent_import_bridge_11
remaining_mutation_import_bridge_9
remaining_ai_proposal_import_bridge_9
```

当前 `runtime.mutation.ai_proposal_import_pass` 剩余文件:

```text
src/runtime/mutation/ai_proposal.rs
src/runtime/mutation/ai_proposal/approval_persistence.rs
src/runtime/mutation/ai_proposal/approval_review.rs
src/runtime/mutation/ai_proposal/event_lifecycle.rs
src/runtime/mutation/ai_proposal/proposal_creation.rs
src/runtime/mutation/ai_proposal/sandbox_trigger.rs
src/runtime/mutation/ai_proposal/source_governance_identity.rs
src/runtime/mutation/ai_proposal/static_check.rs
src/runtime/mutation/ai_proposal/status_transition.rs
```

`record_query.rs` 已不再属于本父叶 residual。

---

## 候选复核

| 候选 | 文件范围 | 风险 | 判定 |
| --- | --- | --- | --- |
| `runtime.mutation.ai_proposal.source_governance_identity_import_pass` | `source_governance_identity.rs` | source context / governance projection / deterministic id，依赖面窄于 creation 与 review | 采纳 |
| `runtime.mutation.ai_proposal.static_check_import_pass` | `static_check.rs` | static-check 规则多，测试文本和 v4 domain binding 较多 | 延后 |
| `runtime.mutation.ai_proposal.event_lifecycle_import_pass` | `event_lifecycle.rs` | event envelope / persistence transition | 延后 |
| `runtime.mutation.ai_proposal.approval_persistence_import_pass` | `approval_persistence.rs` | approval disk read/write | 延后 |
| `runtime.mutation.ai_proposal.approval_review_import_pass` | `approval_review.rs` | approve / reject / claim 路由、lock order 与状态迁移较重 | 延后 |
| `runtime.mutation.ai_proposal.sandbox_trigger_import_pass` | `sandbox_trigger.rs` | background retry / sandbox gate / report URL | 延后 |
| `runtime.mutation.ai_proposal.proposal_creation_import_pass` | `proposal_creation.rs` | create path 依赖 sibling helper 与 approval/sandbox 副作用 | 延后 |
| `runtime.mutation.ai_proposal.status_transition_import_pass` | `status_transition.rs` | 状态迁移 guard 依赖 approval review | 延后 |
| `runtime.mutation.ai_proposal.parent_facade_import_pass` | `ai_proposal.rs` | parent facade 仍承接 child declaration 和 re-export | 最后处理 |

---

## 采纳方案

下一步固定为:

```text
BE-001ER-01 runtime.mutation.ai_proposal.source_governance_identity_import_pass 单子叶等价基线
```

目标文件:

```text
src/runtime/mutation/ai_proposal/source_governance_identity.rs
```

选择理由:

1. 它包含 source context loading、governance projection 和 deterministic record id 三个稳定 helper。
2. 它不直接处理 approval review、sandbox trigger、proposal creation side effect 或 route-facing handler。
3. 它可用单文件 import pocket 继续压缩 parent wildcard import。
4. 它能为后续 static_check、event_lifecycle 和 proposal_creation 的显式输入面提供模板。

---

## 排除项

本批不处理:

1. 不修改 Rust 代码。
2. 不处理 `static_check.rs`、`event_lifecycle.rs`、`approval_persistence.rs`、`approval_review.rs`、`sandbox_trigger.rs`、`proposal_creation.rs`、`status_transition.rs` 或 parent facade。
3. 不处理 `src/runtime/mod.rs` root parent bridge。
4. 不处理 test-only `src/runtime/run_guard.rs`。
5. 不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
6. 不新增 sibling horizontal link。
7. 不启动 release transition。

---

## 验证要求

本批为 `no code movement` 父叶残余判断，提交前至少执行:

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

AI 声称 BE-001EQ-01 完成时，必须说明:

1. 本批只是 `no code movement` 父叶残余判断。
2. `runtime.mutation.ai_proposal_import_pass stop_split: false`。
3. 下一步只能进入 BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线。
4. 不得宣称 source governance identity import 已改写、ai proposal import 已完成、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 或 Rust 重构已完成。

---

## 验收标准

1. `419-runtime.mutation.ai_proposal_import_pass第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`。
3. 下一步固定为 BE-001ER-01 `runtime.mutation.ai_proposal.source_governance_identity_import_pass` 单子叶等价基线。
4. 治理门禁、全量树覆盖和 Rust 验证均通过。
