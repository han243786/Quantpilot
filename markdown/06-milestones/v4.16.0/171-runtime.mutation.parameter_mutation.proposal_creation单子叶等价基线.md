# v4.16.0 runtime.mutation.parameter_mutation.proposal_creation 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AU-01  
> 基线: `170-runtime.mutation.parameter_mutation父叶残余判断.md`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`、`tests/api_ai_proposal.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs`  
> 判定: `runtime.mutation.parameter_mutation.proposal_creation` 单子叶等价基线已建立；当前 `no code movement`，只冻结 create handler 与 proposal record id helper 的输入、输出、调用顺序和不变约束。下一步只能进入 BE-001AU-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AU-01 proposal_creation 单子叶等价基线 | 基线 |
| 规范矩阵 | 父子通信、public handler、proposal id helper、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.proposal_creation` | 新候选叶子 |
| 模块树 | `runtime.mutation.parameter_mutation.proposal_creation` | 建立白箱边界 |

---

## 白箱边界

| 项 | 基线 |
| --- | --- |
| 父级 owner | `src/runtime/mutation/parameter_mutation.rs` |
| 候选目标文件 | `src/runtime/mutation/parameter_mutation/proposal_creation.rs` |
| 候选方法 | `create_runtime_parameter_mutation` |
| 候选 helper | `runtime_parameter_mutation_record_id` |
| 输入 | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeParameterMutationRequest>` |
| 输出 | `Result<Json<RuntimeParameterMutationRecord>, (StatusCode, String)>` |
| 下一批次 | BE-001AU-02 抽离方案 |

本批不创建 `proposal_creation.rs`，不移动 `create_runtime_parameter_mutation`，不移动 `runtime_parameter_mutation_record_id`。

---

## 冻结调用顺序

`create_runtime_parameter_mutation` 的等价顺序必须保持:

1. `validate_runtime_capability_guard`
2. `RuntimeEvidenceSourceKind::Run` 检查
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
18. `state.evidence_metrics.record_mutation_proposal`
19. `state.parameter_mutations.write().await.insert`
20. `Ok(Json(record))`

---

## record id helper 等价

`runtime_parameter_mutation_record_id` 必须保持:

- 输入仍为 `CreateRuntimeParameterMutationRequest`、`created_at_ms`、`source_event_count`、`proposed_parameter_version`。
- 返回值仍为 `Result<String, (StatusCode, String)>`。
- digest input 仍包含 `created_at_ms`、`source_event_count`、`source_kind`、`source_id`、`target`、`proposed_parameter_version`。
- digest 仍通过 `canonical_json_sha256_digest` 与 `json!` 构造。
- error mapping 仍通过 `internal_error(anyhow::anyhow!(error))`。
- id prefix 仍为 `parameter_mutation_`。
- output segment 仍由 `created_at_ms` 和 `digest[..12]` 组成。

---

## record 字段等价

`RuntimeParameterMutationRecord` 构造必须保持:

- `proposal_id` 来自 `runtime_parameter_mutation_record_id`。
- `source_kind`、`source_id`、`target`、`old_value`、`new_value` 均来自 request。
- `graph_id` 来自 source run。
- `old_parameter_version` 与 `proposed_parameter_version` 来自 canonicalization。
- noop 时 `status` 为 `RuntimeParameterMutationStatus::Rejected`，否则为 `RuntimeParameterMutationStatus::Proposed`。
- noop rejection reason 仍为 `旧值和新值解析为相同的规范参数版本`。
- `activation_boundary` 来自 request。
- `activation_state`、`safe_window_state`、`rollback_of`、`rollback_target_parameter_version` 仍为 `None`。
- `actor` 来自 normalized actor。
- `reason` 来自 trimmed request reason。
- `governance` 来自 `runtime_parameter_mutation_governance`。
- `lifecycle` 仍为 `Vec::new()`。
- `created_at_ms` 与 `updated_at_ms` 仍为同一个 `now_ms`。

---

## 父子通信规则

后续若进入实际抽离，父级通信必须保持:

```text
src/runtime/mod.rs
  -> runtime.mutation.parameter_mutation public handlers
src/runtime/mutation/parameter_mutation.rs
  -> proposal_creation::create_runtime_parameter_mutation
src/runtime/mutation/parameter_mutation/proposal_creation.rs
  -> parent-owned imports / helpers via use super::*
```

`proposal_creation` 不得被 route facade、AI proposal、approval review、AppState owner、schema owner、frontend caller 或发布过渡连接直接依赖。发布过渡仍未启动，ASCII guard: `release transition guard`。

---

## 非目标

- 不移动 Rust 代码。
- 不创建 `proposal_creation.rs`。
- 不迁移 `list_runtime_parameter_mutations`。
- 不迁移 `get_runtime_parameter_mutation_detail`。
- 不回改已 closeout 的 `transition_lifecycle`。
- 不迁移 AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner 或 route facade。
- 不启动发布过渡，不提出横向连接或性能旁路。

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

AI 声称 BE-001AU-01 完成时，必须说明当前只是 `proposal_creation` 单子叶等价基线，仍为 `no code movement`；目标文件尚未创建，create handler 与 record id helper 尚未迁移。下一步只能进入 BE-001AU-02 抽离方案。不得宣称 proposal_creation 已抽离、list/detail 已迁移、transition_lifecycle 已回改、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变或发布过渡已启动。

---

## 验收标准

1. `171-runtime.mutation.parameter_mutation.proposal_creation单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.parameter_mutation.proposal_creation` 白箱候选节点。
3. 基线冻结 create handler、record id helper、record 字段、调用顺序和非目标。
4. 本批无代码移动。
5. 本批验证通过后，后续才能进入 BE-001AU-02。
