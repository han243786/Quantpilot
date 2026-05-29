# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation 抽离方案
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AU-02  
> 基线: `171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`、`tests/api_ai_proposal.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs`  
> 判定: `runtime.mutation.parameter_mutation.proposal_creation` 抽离方案已建立；当前仍为 `no code movement`，只固定 BE-001AU-03 的目标文件、父级声明、child 出口、迁移清单、非目标和回退点。下一步只能进入 BE-001AU-03 实际抽离。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AU-02 proposal_creation 抽离方案 | 方案冻结 |
| 规范矩阵 | 父子通信、public handler re-export、record id helper、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.proposal_creation` | 子叶抽离路径 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation` | 白箱方案 |

---

## 目标文件与父级声明

BE-001AU-03 只允许创建一个目标文件:

```text
src/runtime/mutation/parameter_mutation/proposal_creation.rs
```

父级 `src/runtime/mutation/parameter_mutation.rs` 只允许新增以下 child 声明:

```rust
#[path = "parameter_mutation/proposal_creation.rs"]
mod proposal_creation;
```

父级只允许新增以下 public handler 出口:

```rust
pub(crate) use proposal_creation::create_runtime_parameter_mutation;
```

child 文件必须以父级白箱输入为唯一来源:

```rust
use super::*;
```

---

## BE-001AU-03 迁移清单

只允许迁移:

- `pub(crate) async fn create_runtime_parameter_mutation`
- `fn runtime_parameter_mutation_record_id`

迁移后 handler 签名必须保持:

```rust
pub(crate) async fn create_runtime_parameter_mutation(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateRuntimeParameterMutationRequest>,
) -> Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>
```

`runtime_parameter_mutation_record_id` 仍为 child 私有 helper，不对 sibling 或 route facade 暴露。

---

## 必须保持的调用顺序

`create_runtime_parameter_mutation` 的等价顺序必须保持:

1. `validate_runtime_capability_guard`
2. `RuntimeEvidenceSourceKind::Run`
3. `validate_runtime_parameter_mutation_target`
4. `validate_runtime_parameter_mutation_boundary`
5. actor presence check 与 `normalize_actor_identity`
6. reason trim / empty rejection
7. `load_run_record_from_state`
8. old/new `canonical_runtime_parameter_version`
9. noop 判定
10. `current_time_ms`
11. `runtime_parameter_mutation_record_id`
12. `runtime_parameter_mutation_governance`
13. 构造 `RuntimeParameterMutationRecord`
14. `build_runtime_parameter_mutation_event`
15. `governance_with_parameter_version`
16. `append_parameter_mutation_events_to_run`
17. `persist_runtime_parameter_mutation_record`
18. `record_mutation_proposal`
19. `state.parameter_mutations`
20. `Ok(Json(record))`

---

## record id helper 等价约束

`runtime_parameter_mutation_record_id` 必须保持:

- `canonical_json_sha256_digest`
- `json!`
- `internal_error`
- `parameter_mutation_`
- `digest[..12]`

digest input 必须继续包含 `created_at_ms`、`source_event_count`、`source_kind`、`source_id`、`target` 与 `proposed_parameter_version`。

---

## 非目标

本批不得迁移:

- `list_runtime_parameter_mutations`
- `get_runtime_parameter_mutation_detail`
- `AI proposal`
- `approval review`
- `AppState`
- `schema`
- `frontend caller`
- release transition guard

不得改变 route facade、runtime persistence owner、response mapping owner、`RuntimeParameterMutationRecord` schema、`CreateRuntimeParameterMutationRequest` schema、lock order 或 frontend route。

---

## 父子通信规则

```text
backend.runtime.routes.mutation
  -> src/runtime/mod.rs facade
  -> src/runtime/mutation/parameter_mutation.rs
  -> proposal_creation::create_runtime_parameter_mutation
  -> parent-owned helpers via use super::*
```

`proposal_creation` 不得被 route facade、AI proposal、approval review、AppState owner、schema owner、frontend caller 或发布过渡连接直接依赖。发布过渡未启动，且 AI 不得主动提出横向连接或性能旁路。

---

## 回退点

若 BE-001AU-03 编译或等价检查失败，只允许回退本批新增的:

- `#[path = "parameter_mutation/proposal_creation.rs"] mod proposal_creation;`
- `pub(crate) use proposal_creation::create_runtime_parameter_mutation;`
- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`

不得回改 `transition_lifecycle` closed child，不得改 route facade、schema、AppState、frontend caller 或 persistence owner。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001AU-02 完成时，必须说明当前只是 `proposal_creation` 抽离方案，仍为 `no code movement`；目标文件尚未创建，`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 尚未迁移。下一步只能进入 BE-001AU-03 实际抽离。不得宣称 list/detail 已迁移、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或 `runtime.mutation.parameter_mutation` 父叶已经完成。

---

## 验收标准

1. `172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 目标文件、父级 path attribute、handler re-export、`use super::*`、迁移清单与非目标已冻结。
3. 本批无 Rust 代码移动。
4. BE-001AU-03 只能移动 create handler 与 record id helper。
