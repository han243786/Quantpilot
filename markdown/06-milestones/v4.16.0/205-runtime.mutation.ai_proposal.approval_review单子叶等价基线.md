# v4.16.0 runtime.mutation.ai_proposal.approval_review 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BH-01  
> 父级: `runtime.mutation.ai_proposal`  
> 上游判定: `204-runtime.mutation.ai_proposal第四轮父叶残余判断.md`  
> 判定: 建立 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线，冻结 approval list/detail/approve/reject/claim 五个 route-facing handler 的输入输出、锁顺序、review_state guard、reviewer lifecycle、quorum、proposal status side effect 与非目标边界。当前 `no code movement`，下一步只能进入 BE-001BH-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BH-01 approval_review 单子叶等价基线 | 建基线 |
| 规范矩阵 | 父子通信、锁顺序、route-facing handler 等价、非目标边界 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.approval_review` | 新增白箱候选 |
| 模块树 | `runtime.mutation.ai_proposal.approval_review` | `stop_split: pending` |

---

## 真实文件基线

本批只冻结当前真实文件，不创建目标 child:

- `src/runtime/mutation/ai_proposal.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mod.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/frontend_api_types.rs`
- `tests/api_ai_proposal.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `tests/api_run.rs`

计划目标文件为 `src/runtime/mutation/ai_proposal/approval_review.rs`，只能在 BE-001BH-03 实际抽离时由抽离记录创建。

---

## 范围冻结

`approval_review` 本轮只覆盖下列当前父级函数:

| 函数 / 入口 | 当前职责 | 等价要求 |
| --- | --- | --- |
| `list_runtime_approvals` | 读取当前用户 scoped approval records，按 `review_state` 可选过滤并按 `created_at_ms` 倒序排序 | 不改变 scoped prefix、filtering、sorting 或 response shape |
| `get_runtime_approval_detail` | 先按 `auth::scoped_key` 从 memory lookup，miss 后按 `approval_id` 调用 disk fallback | 不改变 memory-first 与 disk fallback 语义 |
| `approve_ai_proposal` | 读取 proposal、执行 sandbox approval gate、更新 approval reviewer/quorum/lifecycle，必要时把 proposal 状态改为 Approved | 不改变 guard、quorum、lifecycle、status side effect 或锁顺序 |
| `reject_ai_proposal` | 将 pending/under_review approval 置为 Rejected，写入 rejection lifecycle，并把 proposal 状态改为 Denied | 不改变 rejection comment fallback、lifecycle 或 denied side effect |
| `claim_ai_proposal_review` | pending-only claim，将 reviewer 加入 assigned 并进入 UnderReview | 不改变 pending-only guard、assigned 去重或 claim lifecycle |

---

## HTTP route 基线

| Route | Method | Handler | 等价冻结 |
| --- | --- | --- | --- |
| `/api/v1/ai/approvals` | GET | `list_runtime_approvals` | 保持 user scoped list 与 `review_state` filtering |
| `/api/v1/ai/approvals/:approval_id` | GET | `get_runtime_approval_detail` | 保持 memory-first detail 与 disk fallback |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `approve_ai_proposal` | 保持 approve guard、quorum 与 status side effect |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `reject_ai_proposal` | 保持 reject guard、comment fallback 与 denied side effect |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `claim_ai_proposal_review` | 保持 pending-only claim 和 reviewer assignment |

Route facade `src/backend/runtime/routes/mutation.rs` 不在本批迁移范围内。

---

## 输入输出契约

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| approval list query | API caller | `RuntimeApprovalListQuery` | `review_state` 为可选字符串过滤，比较方式保持 `Debug` + lowercase |
| approval detail id | API caller | approval id path segment | 不改变 scoped memory key 或 fallback id |
| approval action | approve/reject/claim route | `ApprovalActionRequest` | 保持 `actor_id` 与可选 `comment` 语义 |
| proposal id | approve/reject/claim route | proposal id path segment | 不改变按 proposal_id 查找 approval 的现有方式 |
| proposal record | parent record_query child | `RuntimeAiProposalRecord` | approve path 继续通过 `load_runtime_ai_proposal_for_user` 读取 |

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| approval list | frontend / tests | `Vec<RuntimeApprovalRecord>` | 不新增 pagination，不改变排序 |
| approval detail | frontend / tests | `RuntimeApprovalRecord` | 不改变 not found / IO error mapping |
| approval action result | frontend / tests | `RuntimeApprovalRecord` | 不改变 reviewers、review_state、lifecycle、sandbox_report_url 或 timestamps |
| proposal status side effect | `AppState.ai_proposals` | `RuntimeAiProposalStatus` | approve 到 `Approved`，reject 到 `Denied`，仍经父级 status helper |

---

## 状态机与锁顺序

`approval_review` 必须冻结以下行为:

1. `approve_ai_proposal` 在读取 approval 前先调用 `load_runtime_ai_proposal_for_user` 与 `ensure_ai_proposal_can_be_approved`。
2. approve/reject/claim action 当前均持有 `state.approval_records.write()` 完成读改写。
3. approve/reject 的 proposal 状态副作用保持 `approval_records -> ai_proposals` 锁顺序。
4. approve 只在 actor 既未 approved 又未 rejected 时加入 `reviewers_approved`。
5. approve 达到 `reviewers_required` 后进入 `RuntimeApprovalReviewState::Approved` 并写入 `APPROVAL_APPROVED` lifecycle；未达到 quorum 时进入 `UnderReview` 并写入 `APPROVAL_PARTIAL` lifecycle。
6. reject 允许 Pending / UnderReview，进入 `Rejected` 并写入 `APPROVAL_REJECTED` lifecycle，message 使用 request comment 或 `"审批拒绝"` fallback。
7. claim 只允许 Pending，actor 未在 `reviewers_assigned` 时追加，进入 `UnderReview` 并写入 `APPROVAL_CLAIMED` lifecycle。

---

## 父级保留 helper

BE-001BH-01 不迁移以下 helper，后续 BE-001BH-02/03 也只能在方案明确允许后处理:

- `load_runtime_ai_proposal_for_user`，由 closed child `record_query` 提供，approval_review 只能经父级受控调用。
- `ensure_ai_proposal_can_be_approved` 与 `load_sandbox_report_for_proposal`，仍归 `sandbox_trigger` 残余。
- `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status`，仍归 `status_transition` 残余。
- `persist_approval` 与 `load_approval_from_disk`，仍归 `approval_persistence` 残余。

---

## 非目标边界

本批不创建 `src/runtime/mutation/ai_proposal/approval_review.rs`，不移动代码，不改测试。

不得迁移或修改:

- `create_runtime_ai_proposal`
- `load_sandbox_report_for_proposal`
- `ensure_ai_proposal_can_be_approved`
- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`
- `persist_approval`
- `load_approval_from_disk`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回收或重拆 `static_check`、`source_governance_identity`、`event_lifecycle` 或 `record_query` 已 closeout 子叶。

---

## 等价证据与测试缺口

现有证据:

- `tests/api_ai_proposal.rs` 覆盖 AI proposal create、proposal list/detail、static check event 与 sandbox gate helper 间接链路。
- `tests/api_mutation.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs` 覆盖 mutation/evidence/runtime 旁路回归。
- `cargo check -p quantpilot` 与 `cargo test --no-run` 能捕获 handler signature、route export 与 schema drift。

当前需明确的缺口:

- approval action endpoints 的行为断言目前主要靠 handler 编译和间接契约保护，BE-001BH-03 实际抽离如触碰 reviewer lifecycle 或锁顺序，应优先补一组 focused API equivalence tests，而不是扩大功能语义。

---

## 下一步

下一步只能进入:

```text
BE-001BH-02 runtime.mutation.ai_proposal.approval_review 抽离方案
```

该方案必须先固定目标文件、父级 `#[path]` 声明、handler re-export、helper import、允许迁移清单、回退点和验证门禁，不得直接移动代码。

---

## 验证计划

本批 `no code movement`，只需要治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际抽离必须扩展为:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 幻觉检查点

AI 声称 BE-001BH-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.approval_review` 单子叶等价基线，尚未创建 `approval_review.rs`，也尚未迁移 handler。不得宣称 approval persistence、sandbox trigger、status transition、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition 已改变。

---

## 验收标准

1. `205-runtime.mutation.ai_proposal.approval_review单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.approval_review` 白箱候选，状态为 `stop_split: pending`。
3. 下一步固定为 BE-001BH-02 `runtime.mutation.ai_proposal.approval_review` 抽离方案。
4. 本批不产生代码变更，不回收 closed child，不启动 release transition。
