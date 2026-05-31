# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DT-02
> 基准: `364-runtime.mutation.parameter_mutation.proposal_creation_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.mutation.parameter_mutation.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.proposal_creation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DT-03 实际抽离记录

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DT-02 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 抽离方案 | 方案固定 |
| 规范矩阵 | single-file explicit import rewrite、parent white-box input、no sibling horizontal link | 实施边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.proposal_creation_import_pass` | proposal creation 抽离方案 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 固定下一步实施范围 |

---

## 实施范围

BE-001DT-03 只允许改写:

```text
src/runtime/mutation/parameter_mutation/proposal_creation.rs
single_file_proposal_creation_import_rewrite
```

允许动作:

1. 删除 `use super::*;`。
2. 增加显式 import。
3. 若编译确认需要，对 import 分组做最小调整。

禁止动作:

```text
no_function_body_change
no_handler_signature_change
no_visibility_change_by_default
no_transition_lifecycle_rewrite
no_parameter_mutation_parent_facade_rewrite
no_ai_proposal_rewrite
no_root_bridge_rewrite
no_sibling_horizontal_link
no_release_transition
old_three_leaf_pause_target_cancelled
```

---

## 预期显式输入

BE-001DT-03 的预期 import 面:

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

若编译确认 `mutation_parameter_mutation::validate_runtime_parameter_mutation_boundary` 的私有父级白箱路径不可用，允许在 BE-001DT-03 中只调整 import 路径，不得改函数体或改 lifecycle sibling visibility。若必须改 visibility，暂停并回到方案复核。

---

## 预期残余变化

```text
expected_parent_import_bridge_21_to_20
expected_mutation_import_bridge_19_to_18
expected_parameter_mutation_import_bridge_9_to_8
```

不应变化:

```text
record_query_import_bridge_0
run residual 0
backtest residual 0
root residual 1
test-only residual 1
```

---

## 等价要求

BE-001DT-03 必须保持:

1. `runtime_parameter_mutation_record_id` digest 输入不变。
2. `create_runtime_parameter_mutation` handler signature 和返回类型不变。
3. capability/source/target/boundary/actor/reason 校验顺序不变。
4. source run 读取、parameter version 计算、noop reject、governance 构造、event append、record persistence、metrics、cache 更新顺序不变。
5. 错误码和中文错误文案不变。
6. 未新增 release transition 或 sibling horizontal link。

---

## 下一步

下一步只能进入:

```text
BE-001DT-03
runtime.mutation.parameter_mutation.proposal_creation_import_pass
实际抽离记录
```

BE-001DT-03 完成后才能进入单叶 closeout；不得跳过实际抽离记录直接 closeout。

---

## 验证要求

BE-001DT-03 提交前至少执行:

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

AI 声称 BE-001DT-02 完成时，必须说明:

1. 本批是 `no code movement`。
2. 下一步只允许改 `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 顶部 import。
3. 不允许改 `transition_lifecycle`、parent facade、AI proposal、root bridge 或 release transition。
4. 预期 parent bridge 从 total 21 / mutation 19 降到 total 20 / mutation 18。
5. 旧三叶暂停目标保持取消。

---

## 验收标准

1. `365-runtime.mutation.parameter_mutation.proposal_creation_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001DT-03 实施范围被固定为单文件 import rewrite。
3. 方案明确禁止 sibling horizontal link 与 release transition。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
