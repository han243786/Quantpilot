# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AU-04  
> 基线: `173-runtime.mutation.parameter_mutation.proposal_creation抽离记录.md`、`src/runtime/mutation/parameter_mutation.rs`、`src/runtime/mutation/parameter_mutation/proposal_creation.rs`、`tests/api_mutation.rs`、`tests/api_ai_proposal.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs`  
> 判定: `runtime.mutation.parameter_mutation.proposal_creation` 单叶 closeout 已完成；实际抽离等价成立，本叶设置 `stop_split: true`。下一步只能进入 BE-001AV-01 `runtime.mutation.parameter_mutation` 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AU-04 proposal_creation 单叶 closeout | 收口 |
| 规范矩阵 | stop_split、父子通信、public handler 边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.proposal_creation` | 叶子完成 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation` | closeout |

---

## 等价结论

`create_runtime_parameter_mutation` 与 `runtime_parameter_mutation_record_id` 已迁入 `src/runtime/mutation/parameter_mutation/proposal_creation.rs`，父级 `src/runtime/mutation/parameter_mutation.rs` 通过:

```rust
#[path = "parameter_mutation/proposal_creation.rs"]
mod proposal_creation;
pub(crate) use proposal_creation::create_runtime_parameter_mutation;
```

维持 route facade 调用面。child 仍通过 `use super::*` 复用父级白箱输入，未新增横向依赖。

---

## 保持不变

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
- `canonical_json_sha256_digest`
- `json!`
- `internal_error`
- `parameter_mutation_`
- `digest[..12]`

---

## 停止细分判定

`proposal_creation` 设置 `stop_split: true`。

理由:

1. 本叶只有一个 public handler: `create_runtime_parameter_mutation`。
2. `runtime_parameter_mutation_record_id` 只服务 proposal creation，不形成可复用 sibling owner。
3. handler 内的 capability guard、source run load、record build、event append、persistence、metrics 与 in-memory index insert 属于同一 proposal transaction。
4. 继续拆 record builder、event append 或 persistence wrapper 会增加父级 import、visibility 和回退成本，但不会形成新的稳定模块边界。

---

## 未迁移边界

仍未迁移:

- `list_runtime_parameter_mutations`
- `get_runtime_parameter_mutation_detail`
- `AI proposal`
- `approval review`
- `AppState`
- `schema`
- `frontend caller`
- release transition guard

route facade、runtime persistence owner、response mapping owner、frontend route、lock order 和发布过渡连接均未改变。

---

## 下一步

BE-001AU-04 完成后，只能进入 BE-001AV-01 `runtime.mutation.parameter_mutation` 父叶残余判断。

父叶残余判断必须确认:

- `proposal_creation` 已 closeout 并设置 `stop_split: true`
- `transition_lifecycle` 已 closeout 并设置 `stop_split: true`
- `list_runtime_parameter_mutations` 与 `get_runtime_parameter_mutation_detail` 是否形成下一候选
- 不得直接迁移 AI proposal、approval review、AppState、schema、frontend caller 或发布过渡连接

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

AI 声称 BE-001AU-04 完成时，必须说明 `proposal_creation` 已完成单叶 closeout 并设置 `stop_split: true`，但 `runtime.mutation.parameter_mutation` 父叶尚未完成；list/detail 仍留在父级，AI proposal/approval、AppState、schema、frontend caller 和发布过渡均未改变。下一步只能进入 BE-001AV-01 父叶残余判断。

---

## 验收标准

1. `proposal_creation` closeout 文档进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `proposal_creation` `stop_split: true`。
3. 明确下一步为 BE-001AV-01 父叶残余判断。
4. 本批无代码移动。
