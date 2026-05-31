# v4.16.0 runtime.mutation.shared_governance_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DO-03
> 基准: `353-runtime.mutation.shared_governance_import_pass抽离方案.md`、`352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.shared_governance_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.shared_governance_import_pass`
> 代码动作: actual import rewrite
> 下一步: BE-001DO-04 `runtime.mutation.shared_governance_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DO-03 `runtime.mutation.shared_governance_import_pass` 抽离记录 | 实际抽离 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 单文件 import rewrite |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.shared_governance_import_pass` | shared governance import 记录 |
| 模块树 | `runtime.mutation.shared_governance_import_pass` | 实际收敛 |

---

## 实际改动

本批只修改:

```text
src/runtime/mutation/shared_governance.rs
runtime.mutation.shared_governance_import_pass actual_single_file_rewrite
```

已将顶部:

```rust
use super::*;
```

替换为:

```rust
use crate::{
    attach_runtime_event_envelope, auth, canonical_json_sha256_digest, internal_error, io_error,
    json_bad_request, load_run_record_from_state, persist_run_record,
    validate_runtime_event_envelopes, AppState, FrontendRuntimeEvent, RuntimeEventEnvelope,
    RuntimeGovernanceSnapshot, RuntimeParameterMutationGovernance, RuntimeParameterMutationRecord,
    RuntimeParameterMutationStatus, RuntimeParameterMutationTarget, SUPPORTED_FRONTEND_MODULE_KEYS,
};
use axum::http::StatusCode;
use serde_json::{json, Value};
use tokio::fs;
```

函数体未改动，helper 未迁移，visibility 未改动。

---

## 白箱 helper 等价

以下 9 个 helper 保持名称、visibility、返回类型和错误语义不变:

```text
canonical_runtime_parameter_version
validate_runtime_parameter_mutation_target
runtime_mode_from_events
status_contract_value
mutation_event_contract
build_runtime_parameter_mutation_event
append_parameter_mutation_events_to_run
runtime_parameter_mutation_governance
governance_with_parameter_version
```

本批不改变:

1. target/value canonical JSON sha256 版本生成。
2. capability gate 与 `SUPPORTED_FRONTEND_MODULE_KEYS` 校验。
3. mutation status 到 event type / reason code / status contract 的映射。
4. `FrontendRuntimeEvent` payload、severity、summary 和 envelope。
5. run record load、sequence 推进、内存态更新、磁盘持久化条件。
6. governance projection。

---

## 收敛结果

实际扫描结果:

```text
actual_parent_import_bridge_23_to_22
actual_mutation_import_bridge_21_to_20
root 1
run 0
backtest 0
mutation 20
test-only 1
total 22
```

`src/runtime/mutation/shared_governance.rs` 的 parent wildcard 残余已清零:

```text
rg -n "use super::\*|super::" src\runtime\mutation\shared_governance.rs
```

返回空结果。

---

## 仍未处理

剩余 parent bridge 队列为:

```text
src/runtime/mod.rs
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
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/proposal_creation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
src/runtime/run_guard.rs
```

---

## 排除项

- 本批未迁移 helper 到其他文件。
- 本批未处理 `src/runtime/mutation/parameter_mutation.rs` 或 `src/runtime/mutation/parameter_mutation/**`。
- 本批未处理 `src/runtime/mutation/ai_proposal.rs` 或 `src/runtime/mutation/ai_proposal/**`。
- 本批未处理 `src/runtime/mod.rs` root parent bridge。
- 本批未处理 test-only `src/runtime/run_guard.rs`。
- 本批未迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批未新增 sibling horizontal link。
- 本批未启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

本批实际 import pass 至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DO-03 完成时，必须说明:

1. 本批次只改写了 `src/runtime/mutation/shared_governance.rs` 顶部 import。
2. `src/runtime/mutation/shared_governance.rs` 已无 `use super::*` 或 `super::` 残余。
3. parent bridge 总数从 23 降到 22，mutation residual 从 21 降到 20。
4. 下一步只能进入 BE-001DO-04 `runtime.mutation.shared_governance_import_pass` 单叶 closeout。
5. `parameter_mutation`、`ai_proposal`、`src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。
7. 旧的三叶暂停目标仍为取消状态。

不得宣称 mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `354-runtime.mutation.shared_governance_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/mutation/shared_governance.rs` 使用显式 import，不再使用 `use super::*`。
3. `cargo check -p quantpilot` 通过。
4. `api_mutation` 与 `api_ai_proposal` 回归测试通过。
5. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
