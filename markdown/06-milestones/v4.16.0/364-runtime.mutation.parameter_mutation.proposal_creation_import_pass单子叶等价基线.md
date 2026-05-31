# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DT-01
> 基准: `363-runtime.mutation.parameter_mutation_import_pass父叶残余判断.md`
> 目标子叶: `runtime.mutation.parameter_mutation.proposal_creation_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.proposal_creation_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DT-02 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DT-01 `runtime.mutation.parameter_mutation.proposal_creation_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | proposal creation handler equivalence、explicit import pass、parent white-box input | 输入面冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.mutation.parameter_mutation.proposal_creation_import_pass` | proposal creation 白箱登记 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation_import_pass` | 建立单子叶基线 |

---

## 范围冻结

本批只冻结:

```text
src/runtime/mutation/parameter_mutation/proposal_creation.rs
proposal_creation_import_pass baseline_frozen
```

白箱节点:

```text
runtime_parameter_mutation_record_id
create_runtime_parameter_mutation
```

当前状态:

```text
current_parent_wildcard_import: use super::*
remaining_parent_import_bridge_21
remaining_mutation_import_bridge_19
remaining_parameter_mutation_import_bridge_9
old_three_leaf_pause_target_cancelled
```

---

## 真实输入面

`proposal_creation.rs` 当前通过 parent wildcard 隐式使用以下输入。

来自 crate/root support:

```text
auth
canonical_json_sha256_digest
current_time_ms
internal_error
io_error
json_bad_request
json_bad_request_with_details
load_run_record_from_state
normalize_actor_identity
persist_runtime_parameter_mutation_record
validate_runtime_capability_guard
AppState
CreateRuntimeParameterMutationRequest
RuntimeEvidenceSourceKind
RuntimeParameterMutationRecord
RuntimeParameterMutationStatus
```

来自 runtime mutation shared governance:

```text
append_parameter_mutation_events_to_run
build_runtime_parameter_mutation_event
canonical_runtime_parameter_version
governance_with_parameter_version
runtime_parameter_mutation_governance
validate_runtime_parameter_mutation_target
```

来自 `parameter_mutation` 父级白箱:

```text
validate_runtime_parameter_mutation_boundary
```

外部 crate 输入:

```text
axum::extract::State
axum::http::StatusCode
axum::Json
serde_json::json
```

不得把 `validate_runtime_parameter_mutation_boundary` 改成直接访问 lifecycle sibling 的横连；下一批应优先通过父级白箱输入显式指向。

---

## 等价冻结

必须保持:

1. `runtime_parameter_mutation_record_id` 的 digest 输入字段和 ID 前缀。
2. `create_runtime_parameter_mutation` 的 handler signature 与返回类型。
3. capability guard、source kind guard、target guard、activation boundary guard、actor/reason guard。
4. source run loading、old/new parameter version canonicalization、noop reject 语义。
5. governance snapshot、mutation event、run event append、record persistence、metrics 和 in-memory cache 更新顺序。
6. 所有错误码和中文错误文案。
7. 未启动 release transition，未新增 sibling horizontal link。

---

## 排除范围

本基线不处理:

```text
src/runtime/mutation/parameter_mutation.rs
src/runtime/mutation/parameter_mutation/record_query.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle.rs
src/runtime/mutation/parameter_mutation/transition_lifecycle/**
src/runtime/mutation/ai_proposal/**
src/runtime/mod.rs
src/runtime/run_guard.rs
```

不得宣称:

```text
parameter_mutation_import_pass complete
mutation_import_pass complete
parent_import_bridge complete
backend.runtime complete
Rust refactor complete
```

---

## 下一步

下一步只能进入:

```text
BE-001DT-02
runtime.mutation.parameter_mutation.proposal_creation_import_pass
抽离方案
```

BE-001DT-02 必须先确认:

1. 是否只改写 `proposal_creation.rs` 顶部 import。
2. 是否使用父级白箱路径承接 `validate_runtime_parameter_mutation_boundary`。
3. 是否需要调整任何 visibility。默认不调整函数体和 handler visibility。
4. 若显式 import 需要扩大到 lifecycle sibling 直接横连，必须暂停复核。

---

## 验证要求

本批提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DT-01 完成时，必须说明:

1. 本批是 `no code movement`。
2. 本批只冻结 `src/runtime/mutation/parameter_mutation/proposal_creation.rs`。
3. 当前 `proposal_creation.rs` 仍含 `use super::*`，尚未改 Rust。
4. 下一步只能进入 BE-001DT-02 抽离方案。
5. parent bridge 仍为 total 21 / mutation 19。
6. 旧三叶暂停目标保持取消。

---

## 验收标准

1. `364-runtime.mutation.parameter_mutation.proposal_creation_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 单子叶白箱节点和输入面已冻结。
3. 下一步固定为 BE-001DT-02 抽离方案。
4. 不恢复旧三叶暂停目标。
5. Rust / 治理 / 全量树门禁均通过。
