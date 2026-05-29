# v4.16.0 runtime.mutation.ai_proposal.source_governance_identity 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BB-01  
> 基线: `189-runtime.mutation.ai_proposal父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`tests/api_ai_proposal.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线。当前只冻结 source context、governance projection 与 proposal record identity；本批 `no code movement`。下一步只能进入 BE-001BB-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BB-01 source_governance_identity 等价基线 | 新增基线 |
| 规范矩阵 | 父子通信、`pub(super)` visibility 预期、record id 稳定性、非目标边界 | 冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.source_governance_identity` | 新增白箱节点 |
| 模块树 | `runtime.mutation.ai_proposal.source_governance_identity` | 建立单子叶基线 |

---

## 基线范围

`runtime.mutation.ai_proposal.source_governance_identity` 是 `runtime.mutation.ai_proposal` 父叶在 static_check closeout 后的下一候选。它只冻结 create flow 中的 source / governance / identity 读写前置层:

- `RuntimeAiProposalSourceContext`
- `load_runtime_ai_proposal_source_context`
- `runtime_ai_proposal_governance`
- `runtime_ai_proposal_record_id`

本批不创建 child 文件，不移动 helper，不改变 public handler。

---

## 输入输出冻结

| 类型 | 内容 | 等价约束 |
| --- | --- | --- |
| 输入 | `AppState`、`auth::UserId`、`RuntimeEvidenceSourceKind`、`source_id` | `Run` 只读 run record，`Backtest` 只读 backtest record |
| 输入 | `RuntimeGovernanceSnapshot`、old/proposed parameter version | governance projection 不改变字段映射 |
| 输入 | `CreateRuntimeAiProposalRequest`、created timestamp、source event count、proposed parameter version | record id digest input 不改变 |
| 输出 | `RuntimeAiProposalSourceContext` | `graph_id`、`event_count`、`current_sequence_no`、`governance` 语义不变 |
| 输出 | `RuntimeAiProposalGovernance` | capability / deployment / strategy / permission boundary / ai policy 字段不变 |
| 输出 | `ai_proposal_{created_at_ms}_{digest[..12]}` | prefix、digest 输入、12 位截断不变 |

---

## helper 细节冻结

### `RuntimeAiProposalSourceContext`

字段必须保持:

- `graph_id: String`
- `event_count: usize`
- `current_sequence_no: u64`
- `governance: RuntimeGovernanceSnapshot`

若后续 BE-001BB-03 实际抽离，struct 与字段必须允许父级 `create_runtime_ai_proposal` 受控读取，可采用 `pub(super)` struct + `pub(super)` fields；不得把 source context 暴露给 sibling。

### `load_runtime_ai_proposal_source_context`

必须保持:

- `RuntimeEvidenceSourceKind::Run` 使用 `load_run_record_from_state`
- `RuntimeEvidenceSourceKind::Backtest` 使用 `load_backtest_record_from_state`
- `current_sequence_no` 优先取最后一个 event 的 `envelope.sequence_no`
- 无事件时 fallback 到 `events.len() as u64`
- `event_count` 保持 `events.len()`
- `governance` 保持源 record governance snapshot

### `runtime_ai_proposal_governance`

必须保持字段映射:

- `capability_hash`
- `deployment_revision`
- `strategy_version`
- `previous_parameter_version`
- `proposed_parameter_version`
- `permission_boundary_model_version`
- `ai_write_policy`

### `runtime_ai_proposal_record_id`

`canonical_json_sha256_digest` 输入必须保持:

- `created_at_ms`
- `source_event_count`
- `source_kind`
- `source_id`
- `target`
- `model`
- `prompt_hash`
- `evidence_hash`
- `proposed_parameter_version`

输出格式必须保持:

```text
ai_proposal_{created_at_ms}_{digest[..12]}
```

---

## 非目标边界

BE-001BB-01 不得移动或修改:

- `create_runtime_ai_proposal`
- `ai_proposal_static_check_result`
- `validate_ai_model_identity`
- `validate_hash_identity`
- `build_runtime_ai_proposal_event`
- `ai_proposal_lifecycle_entry`
- `persist_runtime_ai_proposal_transition`
- `load_runtime_ai_proposal_for_user`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得把本批解释为 event_lifecycle、record_query、approval_review、approval_persistence、sandbox_trigger 或 status_transition 已拆分。

---

## 验证计划

本批为 `no code movement`，只需文档治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 BE-001BB-02 抽离方案必须明确目标 child 文件、父级 child 声明、`pub(super)` visibility、迁移清单和回退点。BE-001BB-03 实际抽离必须补齐 Rust 编译与 API 回归测试。

---

## 幻觉检查点

AI 声称 BE-001BB-01 完成时，必须说明本批只建立 `runtime.mutation.ai_proposal.source_governance_identity` 单子叶等价基线，并且为 `no code movement`。不得宣称 source_governance_identity helper 已迁移、目标文件已创建、event lifecycle / approval review / record query 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.source_governance_identity` 白箱节点。
3. 本批只冻结 source context、governance projection、record identity 与非目标边界。
4. 下一步只能进入 BE-001BB-02 抽离方案。
