# v4.16.0 runtime.mutation.ai_proposal.record_query 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BF-01  
> 基线: `199-runtime.mutation.ai_proposal第三轮父叶残余判断.md`、`src/runtime/mutation/ai_proposal.rs`、`src/runtime_persistence.rs`、`src/runtime/mod.rs`、`tests/api_ai_proposal.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线。当前只冻结 proposal list/detail/read-through loader 的输入输出、filtering、sorting、memory-first lookup、disk fallback、persistence helper 与非目标边界；本批 `no code movement`。下一步只能进入 BE-001BF-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BF-01 record_query 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | read model owner、memory-first + disk fallback、父子通信硬规则、非目标边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.record_query` | 新增白箱节点 |
| 模块树 | `runtime.mutation.ai_proposal.record_query` | 建立单子叶基线 |

---

## 叶子定义

`runtime.mutation.ai_proposal.record_query` 是 `runtime.mutation.ai_proposal` 父叶在 event_lifecycle closeout 后的下一候选。它只冻结 AI proposal record read model:

- proposal list handler
- proposal detail handler
- proposal memory-first loader
- persistence list/load helper 的调用语义

本叶不拥有 create transaction、approval review、approval persistence、sandbox trigger、status transition、AppState、schema owner、frontend caller 或 release transition 连接。

ASCII non-target markers: `approval_review`、`approval_persistence`、`sandbox_trigger`、`status_transition`。

---

## 输入

| 输入 | 来源 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| list query | `/api/runtime/ai-proposals` | `RuntimeAiProposalListQuery` | `source_kind`、`source_id`、`status` 均为可选过滤条件 |
| detail request | `/api/runtime/ai-proposals/:ai_proposal_id` | `auth::UserId` + proposal id | 必须先查 scoped in-memory record，再 fallback disk |
| loader request | approval / detail / future parent caller | `AppState`、`auth::UserId`、proposal id | 必须保持 `auth::scoped_key` 语义 |
| proposal store dir | `AppState.ai_proposal_store_dir` | filesystem path | 不迁移 storage owner |

---

## 输出

| 输出 | 去向 | 格式/类型 | 约束 |
| --- | --- | --- | --- |
| proposal list | frontend / tests | `Vec<RuntimeAiProposalRecord>` | 不改变 response shape，不引入 pagination |
| proposal detail | frontend / tests | `RuntimeAiProposalRecord` | memory record 优先，disk fallback 语义不变 |
| loader result | parent approval/status/sandbox flow | `RuntimeAiProposalRecord` | 返回同一 record type，不改变 error mapping |

---

## 当前函数边界

| 函数 | 当前职责 | 等价约束 |
| --- | --- | --- |
| `list_runtime_ai_proposals` | 从 `list_runtime_ai_proposal_records` 读取 disk records，再按 query 过滤并倒序排序 | 保持 `source_kind`、trim 后 `source_id`、`status` filtering；保持 `created_at_ms desc` + `ai_proposal_id desc` 排序 |
| `get_runtime_ai_proposal_detail` | 按 user/proposal id 查 detail | 保持 `state.ai_proposals` scoped memory 优先，再 `load_runtime_ai_proposal_record` disk fallback |
| `load_runtime_ai_proposal_for_user` | 供 detail/approve/reject/claim 复用的 read-through loader | 保持 `auth::scoped_key(user_id, proposal_id)`、memory-first、disk fallback |
| `clean_optional_filter` | 清洗 list query 中的 `source_id` | 保持 trim + empty string drop 语义 |
| `list_runtime_ai_proposal_records` | disk list helper | 不迁移 persistence owner |
| `load_runtime_ai_proposal_record` | disk load helper | 不迁移 persistence owner |

---

## route / handler 等价约束

BE-001BF-01 只冻结以下 route-facing 行为:

| Route | Method | Handler | 约束 |
| --- | --- | --- | --- |
| `/api/runtime/ai-proposals` | GET | `list_runtime_ai_proposals` | 不改变 query、filter、sort 或 response shape |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `get_runtime_ai_proposal_detail` | 不绕过 scoped in-memory lookup 或 disk fallback |

`src/backend/runtime/routes/mutation.rs` 与 `src/runtime/mod.rs` 的 route-facing re-export 不得改变。

---

## 非目标

BE-001BF-01 不移动代码，也不创建 `record_query.rs`。后续 BE-001BF-02 只能建立抽离方案，不得直接迁移 handler。

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

后续实际抽离时必须补齐 Rust 编译与 API 回归测试。本基线批次为 `no code movement`，先执行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001BF-03 实际抽离前后需要重点回归:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
```

---

## 幻觉检查点

AI 声称 BE-001BF-01 完成时，必须说明本批只建立 `runtime.mutation.ai_proposal.record_query` 单子叶等价基线，并且为 `no code movement`。不得宣称 record_query helper 已迁移、目标文件已创建、approval review 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `200-runtime.mutation.ai_proposal.record_query单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal.record_query` 白箱节点。
3. 基线冻结 list/detail/loader 的输入输出、filtering、sorting、memory-first lookup、disk fallback 和非目标边界。
4. 下一步只能进入 BE-001BF-02 抽离方案。
