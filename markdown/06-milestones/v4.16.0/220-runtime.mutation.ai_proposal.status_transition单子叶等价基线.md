# v4.16.0 runtime.mutation.ai_proposal.status_transition 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BN-01  
> 基线: `219-runtime.mutation.ai_proposal第七轮父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation/ai_proposal/approval_review.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线。当前 `no code movement`，只冻结 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 与 `update_ai_proposal_status` 的输入输出、状态机迁移矩阵、`state.ai_proposals` 写入副作用、非法转换日志和父级受控调用边界。下一步只能进入 BE-001BN-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BN-01 status_transition 单子叶等价基线 | 基线建立 |
| 规范矩阵 | 父子通信、状态机 guard、closed child 不横连、发布过渡禁止 | 约束冻结 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.status_transition` | 待抽离 child |
| 模块树 | `runtime.mutation.ai_proposal.status_transition` | 白箱基线 |

---

## 当前真实边界

当前代码仍在父级:

```text
src/runtime/mutation/ai_proposal.rs
```

后续候选目标文件仅可规划为:

```text
src/runtime/mutation/ai_proposal/status_transition.rs
```

当前禁止创建该文件；本批只冻结等价基线。

---

## 白箱职责

`runtime.mutation.ai_proposal.status_transition` 只拥有 AI proposal status transition helper:

- `ai_proposal_approved_status`
- `is_valid_ai_proposal_transition`
- `update_ai_proposal_status`

本节点不拥有 create proposal orchestration、approval review handler、sandbox trigger、approval persistence、record query、event lifecycle、static check、source governance identity、AppState owner、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 输入输出基线

| helper | 输入 | 输出 / 副作用 | 必须保持 |
| --- | --- | --- | --- |
| `ai_proposal_approved_status` | 无 | `RuntimeAiProposalStatus::Approved` | 不得把 approved projection 改回 `StaticCheckPassed` |
| `is_valid_ai_proposal_transition` | `current: RuntimeAiProposalStatus`、`next: RuntimeAiProposalStatus` | `bool` | 保持合法迁移矩阵 |
| `update_ai_proposal_status` | `&AppState`、`&auth::UserId`、proposal id、next status | 写入 `state.ai_proposals` 中匹配 scoped record | 保持 scoped lookup、非法转换阻断、`updated_at_ms` 更新 |

合法迁移矩阵固定为:

```rust
(Submitted, StaticCheckPassed | StaticCheckFailed)
(StaticCheckPassed, Approved | Denied | Expired)
```

`update_ai_proposal_status` 必须继续:

1. 通过 `state.ai_proposals.write().await` 获取写锁。
2. 通过 `auth::scoped_key(user_id, proposal_id)` 定位记录。
3. 未找到记录时保持 no-op。
4. 调用 `is_valid_ai_proposal_transition(record.status, status)`。
5. 非法转换时继续 `safe_eprintln!("[ai_proposal] 非法状态转换: {:?} → {:?} (proposal_id={})", ...)` 并 return。
6. 合法转换时设置 `record.status = status`。
7. 合法转换时设置 `record.updated_at_ms = current_time_ms()`。

---

## 调用方基线

| 调用方 | 当前调用 | 后续抽离后必须保持 |
| --- | --- | --- |
| `approval_review::approve_ai_proposal` | `ai_proposal_approved_status()` 与 `update_ai_proposal_status(...)` | 继续经父级 `use super::*` 受控访问 |
| `approval_review::reject_ai_proposal` | `update_ai_proposal_status(..., RuntimeAiProposalStatus::Denied)` | 继续经父级 `use super::*` 受控访问 |
| 父级 test module | 通过父级作用域访问状态 helper | 不得直接横向 import child |

---

## 父子通信规则

`runtime.mutation.ai_proposal.status_transition` 后续如物理抽离，只能被父级 `runtime.mutation.ai_proposal` 连接:

- child 固定 `use super::*`。
- `ai_proposal_approved_status` 与 `update_ai_proposal_status` 后续如迁移，只能以 `pub(super)` 暴露给父级。
- `is_valid_ai_proposal_transition` 必须保持 child private，除非后续方案明确证明父级需要直接调用。
- `approval_review` 必须继续经父级 `use super::*` 受控调用状态 helper，不得直接 import `status_transition` sibling。
- status_transition 不得横向 import `approval_review`、`sandbox_trigger`、`approval_persistence`、`record_query`、`event_lifecycle`、`static_check` 或 `source_governance_identity` sibling。
- 发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 非目标边界

BE-001BN-01 不移动代码，也不创建 `status_transition.rs`。

当前不得迁移或修改:

- `create_runtime_ai_proposal`
- proposal create orchestration
- `approval_review`
- `sandbox_trigger`
- `approval_persistence`
- `record_query`
- `event_lifecycle`
- `static_check`
- `source_governance_identity`
- `AppState`
- schema owner `src/frontend_api_types.rs`
- frontend caller
- route facade `src/backend/runtime/routes/mutation.rs`
- runtime persistence owner
- release transition guard

不得回收或重拆已 closeout 子叶。

---

## 回归保护

本基线批次只跑治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际抽离必须补跑:

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

## 下一步

下一步只能进入:

```text
BE-001BN-02 runtime.mutation.ai_proposal.status_transition 抽离方案
```

---

## 幻觉检查点

AI 声称 BE-001BN-01 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.status_transition` 单子叶等价基线，尚未创建 `status_transition.rs`，也尚未迁移 `ai_proposal_approved_status`、`is_valid_ai_proposal_transition` 或 `update_ai_proposal_status`。不得宣称 proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变。

---

## 验收标准

1. `220-runtime.mutation.ai_proposal.status_transition单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.mutation.ai_proposal.status_transition` 白箱节点记录输入、输出、状态机矩阵、调用方和父子通信规则。
3. 本批不产生代码变更，不创建 `status_transition.rs`。
4. 下一步固定为 BE-001BN-02 抽离方案。
