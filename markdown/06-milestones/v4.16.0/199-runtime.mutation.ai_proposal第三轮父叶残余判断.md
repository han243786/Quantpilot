# v4.16.0 runtime.mutation.ai_proposal 第三轮父叶残余判断
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BE-01  
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`188-runtime.mutation.ai_proposal.static_check单叶closeout.md`、`193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`、`198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/static_check.rs`、`src/runtime/mutation/ai_proposal/source_governance_identity.rs`、`src/runtime/mutation/ai_proposal/event_lifecycle.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check`、`runtime.mutation.ai_proposal.source_governance_identity` 与 `runtime.mutation.ai_proposal.event_lifecycle` 均已 closeout 并设置 `stop_split: true`，但父叶 `runtime.mutation.ai_proposal` 仍存在多个稳定残余职责，父叶继续保持 `stop_split: false`。下一步只能进入 BE-001BF-01 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BE-01 AI proposal 第三轮父叶残余判断 | 递归回到父叶 |
| 规范矩阵 | closed child 不回收、父级剩余职责排序、禁止发布过渡 | 约束收紧 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 残余职责排序 |
| 模块树 | `runtime.mutation.ai_proposal` | `stop_split: false` |

---

## 已 closeout 子叶

| 子叶 | 状态 | 判定 |
| --- | --- | --- |
| `runtime.mutation.ai_proposal.static_check` | BE-001AZ-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.source_governance_identity` | BE-001BB-04 closeout | `stop_split: true`，不继续细拆 |
| `runtime.mutation.ai_proposal.event_lifecycle` | BE-001BD-04 closeout | `stop_split: true`，不继续细拆 |

---

## 当前父叶残余

`src/runtime/mutation/ai_proposal.rs` 当前仍直接承接以下稳定职责:

| 残余职责 | 当前函数 / 入口 | 是否值得继续拆 | 原因 |
| --- | --- | --- | --- |
| proposal create orchestration | `create_runtime_ai_proposal` | 后续候选 | create flow 仍是事务编排 owner，已抽出 source/static/event 后可等待 sibling 排序 |
| record_query | `list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail`、`load_runtime_ai_proposal_for_user` | 是 | list/detail/read-through cache 形成稳定 read model，风险低，适合作为下一单子叶 |
| approval_review | `list_runtime_approvals`、`get_runtime_approval_detail`、`approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review` | 是 | 审批查询和动作包含 reviewer lifecycle、quorum、claim guard，需要独立基线 |
| approval_persistence | `persist_approval`、`load_approval_from_disk` | 是 | approval record disk/memory owner 与 action flow 相关，但可单独冻结 |
| sandbox_trigger | `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` 与 approve path background task | 是 | sandbox gate 与 background verification trigger 有独立外部证据 |
| status_transition | `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status` | 是 | 状态迁移 guard 与 proposal lifecycle update 是独立状态机残余 |

---

## 下一候选判定

下一候选固定为:

```text
BE-001BF-01 runtime.mutation.ai_proposal.record_query 单子叶等价基线
```

选择 `record_query` 的原因:

1. `list_runtime_ai_proposals` 与 `get_runtime_ai_proposal_detail` 是纯 read model handler，边界清晰。
2. `load_runtime_ai_proposal_for_user` 已同时服务 detail / approve / reject / claim，先冻结它的 memory-first + disk fallback 语义，可以降低后续 approval review 抽离风险。
3. 本候选不触碰 `approval_records -> ai_proposals` 锁顺序，不触碰 sandbox background task，不触碰 status transition guard。
4. 本候选能直接用 `tests/api_ai_proposal.rs` 和 `tests/api_mutation.rs` 做等价证据。

---

## 非目标边界

BE-001BE-01 不移动代码，也不创建 `record_query.rs`。后续 BE-001BF-01 也只能建立等价基线，不得直接创建目标文件。

当前不得迁移或修改:

- `create_runtime_ai_proposal`
- `approve_ai_proposal`
- `reject_ai_proposal`
- `claim_ai_proposal_review`
- `list_runtime_approvals`
- `get_runtime_approval_detail`
- `persist_approval`
- `load_approval_from_disk`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回收或重拆 `static_check`、`source_governance_identity` 或 `event_lifecycle` 已 closeout 子叶。

---

## 验证计划

本批 `no code movement`，只需要治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001BE-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal` 第三轮父叶残余判断，`record_query` 尚未建立基线也尚未抽离。不得宣称 `runtime.mutation.ai_proposal` 父级完成、record_query 已拆分、approval review 已拆分、sandbox trigger 已迁移、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `199-runtime.mutation.ai_proposal第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树继续将 `runtime.mutation.ai_proposal` 标记为 `stop_split: false`。
3. 下一候选固定为 BE-001BF-01 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线。
4. 本批不产生代码变更，也不回收已 closeout 子叶。
