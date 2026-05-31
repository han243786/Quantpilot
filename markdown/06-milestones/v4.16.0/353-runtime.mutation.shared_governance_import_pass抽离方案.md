# v4.16.0 runtime.mutation.shared_governance_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DO-02
> 基准: `352-runtime.mutation.shared_governance_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.shared_governance_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.shared_governance_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DO-03 `runtime.mutation.shared_governance_import_pass` 抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DO-02 `runtime.mutation.shared_governance_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge、explicit import pass、minimum batch、release transition guard | 单文件 import rewrite 方案 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.shared_governance_import_pass` | shared governance import 方案 |
| 模块树 | `runtime.mutation.shared_governance_import_pass` | 实改前约束 |

---

## 方案判定

BE-001DO-03 只允许处理:

```text
src/runtime/mutation/shared_governance.rs
runtime.mutation.shared_governance_import_pass single_file_rewrite
```

本批不做 Rust 改写。后续实际抽离只允许将文件顶部的 parent wildcard:

```rust
use super::*;
```

替换为显式输入。禁止移动函数、调整 helper visibility、修改 payload/event/status/governance 语义。

---

## 目标 import 形状

BE-001DO-03 的目标 import 区域固定为四组输入:

```rust
use crate::{
    auth, attach_runtime_event_envelope, canonical_json_sha256_digest, internal_error, io_error,
    json_bad_request, load_run_record_from_state, persist_run_record,
    validate_runtime_event_envelopes, AppState, FrontendRuntimeEvent, RuntimeEventEnvelope,
    RuntimeGovernanceSnapshot, RuntimeParameterMutationGovernance, RuntimeParameterMutationRecord,
    RuntimeParameterMutationStatus, RuntimeParameterMutationTarget, SUPPORTED_FRONTEND_MODULE_KEYS,
};
use axum::http::StatusCode;
use serde_json::{json, Value};
use tokio::fs;
```

该 import 形状必须保持:

1. `crate::{...}` 只引用当前 helper 函数体已经使用的父级/根级输入。
2. `StatusCode` 从 `axum::http` 明确进入本文件。
3. `json` 与 `Value` 从 `serde_json` 明确进入本文件。
4. `fs` 从 `tokio` 明确进入本文件。
5. 不从 sibling child 直接 import `parameter_mutation` 或 `ai_proposal`。

---

## 等价保护

BE-001DO-03 实改后必须保证以下 helper 行为完全等价:

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

等价检查重点:

1. `canonical_json_sha256_digest` 输入不变。
2. `SUPPORTED_FRONTEND_MODULE_KEYS` capability gate 不变。
3. `RuntimeParameterMutationStatus` 到 event/status/reason code 映射不变。
4. `FrontendRuntimeEvent` payload 字段、severity 和 summary 不变。
5. `append_parameter_mutation_events_to_run` 的 run record load、sequence、envelope、in-memory update、persist-if-exists 条件不变。
6. `RuntimeGovernanceSnapshot` 与 `RuntimeParameterMutationGovernance` projection 不变。

---

## 预期结果

BE-001DO-03 完成后，预期:

```text
expected_parent_import_bridge_23_to_22
expected_mutation_import_bridge_21_to_20
```

同时 `rg -l "use super::\*|super::" src\runtime\mutation\shared_governance.rs` 应为空。

---

## 排除项

- 本批不修改 Rust 代码。
- BE-001DO-03 不迁移 helper 到其他文件。
- BE-001DO-03 不处理 `src/runtime/mutation/parameter_mutation.rs` 或 `src/runtime/mutation/parameter_mutation/**`。
- BE-001DO-03 不处理 `src/runtime/mutation/ai_proposal.rs` 或 `src/runtime/mutation/ai_proposal/**`。
- BE-001DO-03 不处理 `src/runtime/mod.rs` root parent bridge。
- BE-001DO-03 不处理 test-only `src/runtime/run_guard.rs`。
- BE-001DO-03 不迁移 route facade、handler owner、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- BE-001DO-03 不新增 sibling horizontal link。
- BE-001DO-03 不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001DO-03 实际 import pass 至少执行:

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

AI 声称 BE-001DO-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. BE-001DO-03 只允许改写 `src/runtime/mutation/shared_governance.rs` 顶部 import。
3. `use super::*` 尚未在本批改写。
4. 下一步只能进入 BE-001DO-03 实际抽离记录。
5. `parameter_mutation`、`ai_proposal`、`src/runtime/mod.rs` 和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。
7. 旧的三叶暂停目标仍为取消状态。

不得宣称 shared governance import 已改写、mutation import 已完成、parent import bridge 已完全清除、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `353-runtime.mutation.shared_governance_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001DO-03 只处理 `src/runtime/mutation/shared_governance.rs` import。
3. 方案列明目标显式 import 形状与两条 runtime API 回归测试。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
