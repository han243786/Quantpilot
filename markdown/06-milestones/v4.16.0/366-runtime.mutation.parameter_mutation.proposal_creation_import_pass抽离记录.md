# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DT-03
> 基准: `365-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离方案.md`
> 目标子叶: `runtime.mutation.parameter_mutation.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.proposal_creation_import_pass`
> 代码动作: actual Rust import rewrite
> 下一步: BE-001DT-04 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DT-03 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 实际抽离 | 单文件实施 |
| 规范矩阵 | explicit import pass、proposal creation equivalence、parent white-box input | parent wildcard 清理 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.proposal_creation_import_pass` | proposal creation 白箱实际收敛 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 实际抽离记录 |

---

## 实际改动

本批只改写:

```text
src/runtime/mutation/parameter_mutation/proposal_creation.rs
single_file_proposal_creation_import_rewrite
```

删除:

```rust
use super::*;
```

新增显式 import:

```rust
use crate::{
    auth, canonical_json_sha256_digest, current_time_ms, internal_error, io_error,
    json_bad_request, json_bad_request_with_details, load_run_record_from_state,
    normalize_actor_identity, persist_runtime_parameter_mutation_record,
    runtime::{
        append_parameter_mutation_events_to_run, build_runtime_parameter_mutation_event,
        canonical_runtime_parameter_version, governance_with_parameter_version,
        mutation_parameter_mutation::validate_runtime_parameter_mutation_boundary,
        runtime_parameter_mutation_governance, validate_runtime_parameter_mutation_target,
    },
    validate_runtime_capability_guard, AppState, CreateRuntimeParameterMutationRequest,
    RuntimeEvidenceSourceKind, RuntimeParameterMutationRecord, RuntimeParameterMutationStatus,
};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
```

函数体、handler signature、visibility、校验顺序、persistence、metrics 和 cache 写入均未改动。

```text
function_bodies_unchanged
handler_signatures_unchanged
actual_parent_import_bridge_21_to_20
actual_mutation_import_bridge_19_to_18
actual_parameter_mutation_import_bridge_9_to_8
actual_proposal_creation_import_bridge_1_to_0
old_three_leaf_pause_target_cancelled
```

---

## 实施校正

方案中的父级白箱路径:

```text
crate::runtime::mutation_parameter_mutation::validate_runtime_parameter_mutation_boundary
```

已经通过 `cargo check -p quantpilot` 验证可用。该路径指向 `parameter_mutation` 父级白箱，不是直接访问 lifecycle sibling 的横向连接；未调整 lifecycle 子树 visibility。

---

## 等价核查

保持不变:

1. `runtime_parameter_mutation_record_id` digest 输入字段和 ID 前缀。
2. `create_runtime_parameter_mutation` handler signature 与返回类型。
3. capability/source/target/boundary/actor/reason 校验顺序。
4. source run loading、old/new parameter version canonicalization、noop reject 语义。
5. governance snapshot、mutation event、run event append、record persistence、metrics 和 in-memory cache 更新顺序。
6. 错误码和中文错误文案。
7. release transition 未启动，未新增 sibling horizontal link。

---

## 当前残余

parent bridge 剩余:

```text
root 1
run 0
backtest 0
mutation 18
test-only 1
total 20
remaining_parent_import_bridge_20
remaining_mutation_import_bridge_18
remaining_parameter_mutation_import_bridge_8
```

剩余 `parameter_mutation` pocket:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_snapshot_side_effect.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/transition_record_persistence.rs
```

---

## 验证要求

本批提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DT-03 完成时，必须说明:

1. 本批只改写 `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 顶部 import。
2. `proposal_creation.rs` 已无 `use super::*` / `super::` residual。
3. parent bridge 剩余从 total 21 / mutation 19 降到 total 20 / mutation 18。
4. `parameter_mutation` residual 从 9 文件降到 8 文件。
5. 下一步只能进入 BE-001DT-04 单叶 closeout。
6. 未改 lifecycle sibling、parent facade、AI proposal、root bridge 或 release transition。
7. 旧三叶暂停目标保持取消。

不得宣称 parameter mutation import 已完成、mutation import 已完成、parent import bridge 已清零、backend.runtime 已完成或 Rust 重构已完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/proposal_creation.rs` parent wildcard import 已删除并改为显式 import。
2. `366-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. parent bridge residual 更新为 total 20 / mutation 18 / parameter_mutation 8。
4. 下一步固定为 BE-001DT-04 单叶 closeout。
5. Rust / 治理 / 全量树门禁均通过。
