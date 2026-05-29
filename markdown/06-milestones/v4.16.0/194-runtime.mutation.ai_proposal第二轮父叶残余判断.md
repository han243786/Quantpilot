# v4.16.0 runtime.mutation.ai_proposal 第二轮父叶残余判断
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BC-01  
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`188-runtime.mutation.ai_proposal.static_check单叶closeout.md`、`193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`src/runtime/mutation/ai_proposal/source_governance_identity.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check` 与 `runtime.mutation.ai_proposal.source_governance_identity` 均已 closeout 并设置 `stop_split: true`，但父叶 `runtime.mutation.ai_proposal` 仍存在多个稳定残余职责，父叶继续保持 `stop_split: false`。下一步只能进入 BE-001BD-01 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BC-01 父叶残余判断 | 路径选择 |
| 规范矩阵 | 单叶 closeout 后回父级、低副作用候选优先、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 残余职责排序 |
| 模块树 | `runtime.mutation.ai_proposal` | `stop_split: false` |

---

## 已完成子叶

| 子叶 | 状态 | 结论 |
| --- | --- | --- |
| `runtime.mutation.ai_proposal.static_check` | BE-001AZ-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.source_governance_identity` | BE-001BB-04 closeout | `stop_split: true`，不继续细拆 |

父级仍通过受控 child import 调用已完成 child:

```rust
#[path = "ai_proposal/source_governance_identity.rs"]
mod source_governance_identity;
#[path = "ai_proposal/static_check.rs"]
mod static_check;
```

```rust
use source_governance_identity::{
    load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
    runtime_ai_proposal_record_id,
};
use static_check::{
    ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
};
```

---

## 父叶剩余职责清单

| 残余职责 | 现有函数 / 结构 | 判定 |
| --- | --- | --- |
| event_lifecycle | `ai_proposal_event_contract`、`build_runtime_ai_proposal_event`、`ai_proposal_lifecycle_entry`、`persist_runtime_ai_proposal_transition` | 下一候选 |
| record_query | `load_runtime_ai_proposal_for_user`、`list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail` | 后续候选 |
| approval_review | `approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review`、`ensure_ai_proposal_can_be_approved` | 后续候选 |
| approval_persistence | `persist_approval`、`load_approval_from_disk`、approval record lookup | 后续候选 |
| sandbox_trigger | `load_sandbox_report_for_proposal`、create flow 中 sandbox verification spawn / callback | 后续候选 |
| status_transition | `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status` | 后续候选 |

父叶还包含 public handler orchestration，因此不能宣称 `runtime.mutation.ai_proposal` 完成。

---

## 下一候选选择

下一步选择:

```text
BE-001BD-01 runtime.mutation.ai_proposal.event_lifecycle 单子叶等价基线
```

选择理由:

1. 该职责由 event contract、event payload、lifecycle entry 与 proposal transition persistence 组成，函数边界集中。
2. 它同时服务 create / approve / reject / claim 事务，但只负责 proposal event 与 lifecycle 写入，不接管 handler orchestration。
3. 它不触碰 record_query、approval review、sandbox trigger、AppState owner、schema owner、frontend caller 或 route facade。

---

## 非目标边界

BE-001BC-01 不得移动代码，不得创建 child 文件，不得改变:

- `create_runtime_ai_proposal`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `list_runtime_ai_proposals`
- `get_runtime_ai_proposal_detail`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
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

后续 BE-001BD-01 基线仍不得移动代码；BE-001BD-03 若进入实际抽离，必须补齐 Rust 编译与 API 回归测试。

---

## 幻觉检查点

AI 声称 BE-001BC-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal` 第二轮父叶残余判断，`event_lifecycle` 尚未建立基线也尚未抽离。不得宣称 `runtime.mutation.ai_proposal` 父级完成、event lifecycle 已拆分、approval review 已拆分、sandbox trigger 已迁移、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `194-runtime.mutation.ai_proposal第二轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 父叶保持 `stop_split: false`，并明确剩余职责清单。
3. 下一候选固定为 BE-001BD-01 `runtime.mutation.ai_proposal.event_lifecycle` 单子叶等价基线。
4. 本批不产生代码变更，不改变 static_check / source_governance_identity 的 closeout 结果。
