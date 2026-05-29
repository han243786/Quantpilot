# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AR-03  
> 前置方案: `166-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity抽离方案.md`  
> 判定: `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` 已完成实际抽离。`runtime_parameter_mutation_rollback_record_id` 已从父级迁入 child，父级只保留 path-attributed child 声明、helper import、handler re-export 与 boundary wrapper。下一步只能进入 BE-001AR-04 单叶 closeout。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AR-03 rollback_record_identity actual extraction | 实际抽离 |
| 规范矩阵 | 父子通信、visibility、rollback id helper 迁移、回退点 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | 新增实际叶子文件 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_record_identity` | child 落地 |

---

## 实际变更

| 项 | 结果 |
| --- | --- |
| 父级 owner | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 调用方 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` |
| 新 child | `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` |
| 父级声明 | `#[path = "transition_lifecycle/rollback_record_identity.rs"] mod rollback_record_identity;` |
| 父级导入 | `use rollback_record_identity::runtime_parameter_mutation_rollback_record_id;` |
| child prelude | `use super::*;` |
| helper visibility | `pub(super) fn runtime_parameter_mutation_rollback_record_id(...)` |
| 下一批次 | BE-001AR-04 单叶 closeout |

本批只迁移 `runtime_parameter_mutation_rollback_record_id`。`src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` 仍通过父级受控 helper 名称调用，不直接依赖 `rollback_record_identity.rs`。

---

## 等价边界

`runtime_parameter_mutation_rollback_record_id` 的以下契约保持不变:

- 输入仍为 `source_id`、`rollback_of`、`RuntimeParameterMutationTarget`、`created_at_ms`、`source_event_count`、`proposed_parameter_version`。
- 返回值仍为 `Result<String, (StatusCode, String)>`。
- digest 仍通过 `canonical_json_sha256_digest` 与 `json!` 构造。
- error mapping 仍通过 `internal_error(anyhow::anyhow!(error))`。
- id prefix 仍为 `parameter_rollback_`。
- output segment 仍由 `created_at_ms` 和 `digest[..12]` 组成。

---

## 父子通信规则

```text
rollback_flow.rs
  -> transition_lifecycle::runtime_parameter_mutation_rollback_record_id
transition_lifecycle.rs
  -> rollback_record_identity::runtime_parameter_mutation_rollback_record_id
rollback_record_identity.rs
  -> parent-owned imports / helpers via use super::*
```

`rollback_record_identity` 只能作为 `transition_lifecycle` child 被父级管理。route facade、AI proposal、approval review、frontend caller、AppState owner、schema owner 和发布过渡连接不得直接依赖本叶。ASCII guard: `release transition guard`。

---

## 本批不做

- 不回改 `rollback_flow` handler 结构。
- 不迁移 `activation_flow`、`boundary_safety`、`activation_snapshot_side_effect` 或 `transition_record_persistence`。
- 不改变 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、runtime persistence owner 或测试 fixture。
- 不主动提出发布版本过渡或横向性能连接。

---

## 回退点

若 BE-001AR-03 出现 visibility、path attribute、borrow checker 或 import 问题，只回退:

- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` 中的 `mod rollback_record_identity` 与 `use rollback_record_identity::runtime_parameter_mutation_rollback_record_id`

不得回退已 closeout 的 `boundary_safety`、`activation_flow`、`rollback_flow`、`activation_snapshot_side_effect` 或 `transition_record_persistence`。

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

AI 声称 BE-001AR-03 完成时，必须说明 `rollback_record_identity` 只是实际抽离完成，尚未 closeout；`transition_lifecycle` 父叶仍保持 `stop_split: false`，下一步只能进入 BE-001AR-04 单叶 closeout。不得宣称 rollback_flow 已回改、transition_lifecycle 父叶完成、parameter_mutation 父叶完成、AI proposal/approval 已拆分、AppState/schema/frontend caller 已改变、发布过渡已启动或整理/重构已经完成。

---

## 验收标准

1. `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_record_identity.rs` 已创建并承接 `runtime_parameter_mutation_rollback_record_id`。
2. 父级 `transition_lifecycle.rs` 通过 path-attributed child 与 helper import 保持 sibling 调用面。
3. `rollback_flow.rs` 无结构性回改，仍经父级 helper 名称调用。
4. 三矩阵、模块树、全量树和治理 gate 均登记 BE-001AR-03。
5. 验证通过后，后续只能进入 BE-001AR-04 单叶 closeout。
