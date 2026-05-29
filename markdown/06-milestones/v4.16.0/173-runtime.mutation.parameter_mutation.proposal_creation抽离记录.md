# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation 抽离记录
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AU-03  
> 基线: `172-runtime.mutation.parameter_mutation.proposal_creation抽离方案.md`、`src/runtime/mutation/parameter_mutation.rs`、`src/runtime/mutation/parameter_mutation/proposal_creation.rs`、`tests/api_mutation.rs`、`tests/api_ai_proposal.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs`  
> 判定: `runtime.mutation.parameter_mutation.proposal_creation` 实际抽离已完成；`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 child，父级只保留 path attribute、handler re-export、transition child import 和 list/detail handler。下一步只能进入 BE-001AU-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AU-03 proposal_creation 实际抽离 | 执行 |
| 规范矩阵 | 父子通信、public handler re-export、record id helper 私有化 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.proposal_creation` | 真实文件落位 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation` | 白箱抽离记录 |

---

## 实际改动

新增 child:

```text
src/runtime/mutation/parameter_mutation/proposal_creation.rs
```

父级新增声明:

```rust
#[path = "parameter_mutation/proposal_creation.rs"]
mod proposal_creation;
```

父级新增 re-export:

```rust
pub(crate) use proposal_creation::create_runtime_parameter_mutation;
```

child 通过父级白箱复用依赖:

```rust
use super::*;
```

---

## 已迁移成员

- `pub(crate) async fn create_runtime_parameter_mutation`
- `fn runtime_parameter_mutation_record_id`

`runtime_parameter_mutation_record_id` 仍为 child 私有 helper，只服务 `create_runtime_parameter_mutation`。

---

## 等价保持点

`create_runtime_parameter_mutation` 仍保持:

- `auth::UserId`
- `State<AppState>`
- `Json<CreateRuntimeParameterMutationRequest>`
- `Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>`
- `validate_runtime_capability_guard`
- `RuntimeEvidenceSourceKind::Run`
- `validate_runtime_parameter_mutation_target`
- `validate_runtime_parameter_mutation_boundary`
- `normalize_actor_identity`
- `load_run_record_from_state`
- `canonical_runtime_parameter_version`
- `runtime_parameter_mutation_governance`
- `build_runtime_parameter_mutation_event`
- `governance_with_parameter_version`
- `append_parameter_mutation_events_to_run`
- `persist_runtime_parameter_mutation_record`
- `record_mutation_proposal`
- `state.parameter_mutations`

`runtime_parameter_mutation_record_id` 仍保持 `canonical_json_sha256_digest`、`json!`、`internal_error`、`parameter_mutation_` 与 `digest[..12]`。

---

## 未迁移边界

本批未迁移:

- `list_runtime_parameter_mutations`
- `get_runtime_parameter_mutation_detail`
- `AI proposal`
- `approval review`
- `AppState`
- `schema`
- `frontend caller`
- release transition guard

未改变 route facade、runtime persistence owner、response mapping owner、frontend route、lock order 或发布过渡连接。

---

## 父子通信结果

```text
backend.runtime.routes.mutation
  -> src/runtime/mod.rs facade
  -> src/runtime/mutation/parameter_mutation.rs
  -> proposal_creation::create_runtime_parameter_mutation
  -> src/runtime/mutation/parameter_mutation/proposal_creation.rs
```

route facade 仍只看原 handler 名；`proposal_creation` 不被 route facade、AI proposal、approval review、AppState owner、schema owner、frontend caller 或发布过渡连接直接依赖。

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

AI 声称 BE-001AU-03 完成时，必须说明只完成 `proposal_creation` 实际抽离，`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 child；list/detail 仍留在父级，AI proposal/approval、AppState、schema、frontend caller 和发布过渡均未改变。下一步只能进入 BE-001AU-04 单叶 closeout，不得宣称 `runtime.mutation.parameter_mutation` 父叶已经完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/proposal_creation.rs` 已创建并进入模块树与全量树。
2. 父级 `src/runtime/mutation/parameter_mutation.rs` 只新增 child 声明和 handler re-export。
3. create handler 与 record id helper 迁移后保持签名、调用顺序和 record/id 语义。
4. list/detail、AI proposal、approval review、AppState、schema、frontend caller 和发布过渡连接均未迁移。
5. 验证通过后进入 BE-001AU-04 单叶 closeout。
