# v4.16.0 runtime.mutation.ai_proposal 第八轮父叶残余判断
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BO-01  
> 基线: `184-runtime.mutation.ai_proposal单叶closeout.md`、`218-runtime.mutation.ai_proposal.sandbox_trigger单叶closeout.md`、`223-runtime.mutation.ai_proposal.status_transition单叶closeout.md`、`src/runtime/mutation/ai_proposal.rs`  
> 判定: `runtime.mutation.ai_proposal` 父叶第八轮残余判断完成。`static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 与 `status_transition` 均已 closeout 并设置 `stop_split: true`；父叶仍保留 proposal create orchestration，因此父叶保持 `stop_split: false`。下一候选固定为 BE-001BP-01 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BO-01 AI proposal 父叶残余判断 | 候选选择 |
| 规范矩阵 | closed child 不回改、父叶残余不直接迁移、发布过渡保护 | 约束固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 下一候选定位 |
| 模块树 | `runtime.mutation.ai_proposal` | 父叶继续 `stop_split: false` |

---

## 当前已 closeout 子叶

已 closeout 并设置 `stop_split: true`:

- `runtime.mutation.ai_proposal.static_check`
- `runtime.mutation.ai_proposal.source_governance_identity`
- `runtime.mutation.ai_proposal.event_lifecycle`
- `runtime.mutation.ai_proposal.record_query`
- `runtime.mutation.ai_proposal.approval_review`
- `runtime.mutation.ai_proposal.approval_persistence`
- `runtime.mutation.ai_proposal.sandbox_trigger`
- `runtime.mutation.ai_proposal.status_transition`

这些子叶不得在本轮回收、重拆或横向连接。

---

## 父叶残余清单

`src/runtime/mutation/ai_proposal.rs` 当前仍保留 `create_runtime_ai_proposal` 创建编排。
该残余包含:

1. `validate_runtime_capability_guard` 与 `proposal_only` policy 校验。
2. `validate_runtime_parameter_mutation_target`、old/new value、model、prompt/evidence hash 与 `normalize_actor_identity` 校验。
3. `load_runtime_ai_proposal_source_context` source context 读取。
4. `canonical_runtime_parameter_version` old/proposed parameter version 构建。
5. `ai_proposal_static_check_result` 静态检查调用。
6. `runtime_ai_proposal_record_id` 与 `runtime_ai_proposal_governance` 构建。
7. `RuntimeAiProposalRecord` 与 source evidence 字段组装。
8. created/static lifecycle event 构建与 run event append。
9. StaticCheckPassed 分支下 `RuntimeApprovalRecord` 自动创建、`APPROVAL_CREATED` lifecycle 与 `persist_approval`。
10. `approval_records -> ai_proposals` 锁顺序保护下的 `persist_runtime_ai_proposal_transition`。
11. `spawn_ai_proposal_sandbox_verification` sandbox trigger 串联。
12. static check failed 分支的 transition persistence。

---

## 候选价值判断

`runtime.mutation.ai_proposal.proposal_creation` 值得进入下一轮单子叶等价基线。

原因:

- 它承接唯一 remaining public create handler `create_runtime_ai_proposal`，是明确 route-facing owner。
- 它串联 source context、static check、governance identity、event lifecycle、approval persistence、sandbox trigger 和 status transition helper，是父级剩余的主要 orchestration owner。
- 它包含独立输入输出、状态副作用、lock-order 注释、approval record side effect 和测试证据，适合单独冻结等价边界。
- 抽离时可以继续保持父级 path-attributed child 与 handler re-export，不需要横向连接 sibling。
- 它不会改变 AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 本轮不执行

BE-001BO-01 不创建目标文件，不迁移代码，不修改 handler 调用面。

不得在本轮执行:

- 创建 `src/runtime/mutation/ai_proposal/proposal_creation.rs`
- 移动 `create_runtime_ai_proposal`
- 拆分 approval record construction
- 拆分 lifecycle event append
- 拆分 sandbox trigger call
- 改变 `approval_records -> ai_proposals` lock order
- 改变 `AppState`
- 改变 schema owner
- 改变 frontend caller
- 改变 route facade
- 启动 release transition guard 之外的发布过渡

---

## 回归保护

本批为 `no code movement`，提交前只需执行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续 BE-001BP 实际抽离时必须补跑:

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
BE-001BP-01 runtime.mutation.ai_proposal.proposal_creation 单子叶等价基线
```

该基线只能冻结 `create_runtime_ai_proposal` 的输入、输出、调用顺序、状态副作用、锁顺序和非目标边界，不得直接创建目标文件或迁移代码。

---

## 幻觉检查点

AI 声称 BE-001BO-01 完成时，必须说明当前只是父叶残余判断，`runtime.mutation.ai_proposal.proposal_creation` 尚未建立等价基线，也尚未创建目标文件。不得宣称 proposal create orchestration、AppState/schema/frontend caller、route facade、runtime persistence owner、release transition 或 Rust backend 重构已完成。

---

## 验收标准

1. `224-runtime.mutation.ai_proposal第八轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树显示 `runtime.mutation.ai_proposal` 父叶保持 `stop_split: false`。
3. 下一候选固定为 BE-001BP-01 `runtime.mutation.ai_proposal.proposal_creation` 单子叶等价基线。
4. 本轮无 Rust 代码移动，不回改任何已 closeout child。
