# v4.16.0 runtime.mutation.ai_proposal.proposal_creation 单叶 closeout

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BP-04
> 基线: `225-runtime.mutation.ai_proposal.proposal_creation单子叶等价基线.md`、`226-runtime.mutation.ai_proposal.proposal_creation抽离方案.md`、`227-runtime.mutation.ai_proposal.proposal_creation抽离记录.md`、`src/runtime/mutation/ai_proposal/proposal_creation.rs`
> 判定: `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout 完成并设置 `stop_split: true`。本叶只承接 `create_runtime_ai_proposal` public handler；继续拆成 approval record construction、event append、transition persistence 或 sandbox trigger call 微叶不会形成稳定 owner，反而会扩大父子接线与锁顺序风险。下一步只能进入 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BP-04 AI proposal proposal_creation 单叶 closeout | 递归收口 |
| 规范矩阵 | 父子通信、public handler、锁顺序、发布过渡保护 | stop_split 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.proposal_creation` | child closeout |
| 模块树 | `runtime.mutation.ai_proposal.proposal_creation` | `stop_split: true` |

---

## 等价基线回放

本叶当前文件:

```text
src/runtime/mutation/ai_proposal/proposal_creation.rs
```

父级连接:

```rust
#[path = "ai_proposal/proposal_creation.rs"]
mod proposal_creation;
pub(crate) use proposal_creation::create_runtime_ai_proposal;
```

child 固定:

```rust
use super::*;
```

route facade、`src/runtime/mod.rs`、AppState、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。

---

## 白箱输入输出

| 项目 | 白箱边界 | closeout 结论 |
| --- | --- | --- |
| public handler | `create_runtime_ai_proposal` | 保持唯一 public 叶子入口 |
| 输入 | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeAiProposalRequest>` | 不改变 request contract |
| 输出 | `Result<Json<RuntimeAiProposalRecord>, (StatusCode, String)>` | 不改变 response / error shape |
| source evidence | `RuntimeAiProposalSourceEvidence` | 仍由 create flow 构造 |
| approval side effect | `RuntimeApprovalRecord`、`RuntimeApprovalLifecycleEntry`、`APPROVAL_COUNTER` | 仍在同一 transaction 内保持锁顺序 |
| proposal side effect | `state.ai_proposals` via `persist_runtime_ai_proposal_transition` | 不拆分 transition owner |
| scoped key | `auth::scoped_key` | 不改变 scoped memory key |
| lock order | `approval_records -> ai_proposals` | 不拆出独立锁 owner |

---

## 不继续细拆的判定

`proposal_creation` 内部看似包含多个动作，但它们共同服务一个 create transaction:

1. capability / target / identity validation。
2. source context 与 parameter version 解析。
3. static check 与 proposal record construction。
4. runtime event / lifecycle append。
5. StaticCheckPassed 时创建 approval record。
6. `approval_records -> ai_proposals` 锁顺序写入。
7. sandbox verification side effect trigger。

这些动作没有独立 route、独立 schema、独立 state owner 或独立 persistence owner。继续拆 `approval record construction`、`lifecycle event append`、`transition persistence`、`sandbox trigger call` 或 `record construction` 微叶，只会让 `proposal_creation` child 依赖更多父级受控 helper，增加接线面和 AI 幻觉风险，因此本叶设置 `stop_split: true`。

---

## 父子通信规则

closeout 后仍固定:

```text
src/runtime/mod.rs
  -> runtime.mutation.ai_proposal public handlers
src/runtime/mutation/ai_proposal.rs
  -> proposal_creation::create_runtime_ai_proposal
src/runtime/mutation/ai_proposal/proposal_creation.rs
  -> parent-owned imports / helpers via use super::*
```

禁止:

- `proposal_creation` 横向 import `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger` 或 `status_transition` sibling。
- sibling 回头 import `proposal_creation`。
- route facade 直接 import child。
- `src/runtime/mod.rs` 直接 import child。
- 迁移 AppState、schema owner、frontend caller、runtime persistence owner 或 route facade。
- 在 release transition guard 之外提出横向连接或性能旁路。

---

## 回归保护

本 closeout 为治理收口批次，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001BP-03 已完成实际抽离并跑过:

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
BE-001BQ-01 runtime.mutation.ai_proposal 父叶残余判断
```

该父叶判断只能确认 `static_check`、`source_governance_identity`、`event_lifecycle`、`record_query`、`approval_review`、`approval_persistence`、`sandbox_trigger`、`status_transition` 与 `proposal_creation` 均已 closeout 后，是否让 `runtime.mutation.ai_proposal` 父叶收口；不得混入 AppState/schema/frontend caller、route facade、runtime persistence owner 或发布过渡连接。

---

## 幻觉检查点

AI 声称 BE-001BP-04 完成时，必须说明当前只是 `runtime.mutation.ai_proposal.proposal_creation` 单叶 closeout，并设置 `stop_split: true`；`runtime.mutation.ai_proposal` 父叶尚未完成残余判断。不得宣称 AppState/schema/frontend caller、route facade、runtime persistence owner 或 release transition guard 已改变，也不得宣称 Rust backend 重构已完成。

---

## 验收标准

1. `runtime.mutation.ai_proposal.proposal_creation` 在模块树中设置 `stop_split: true`。
2. `228-runtime.mutation.ai_proposal.proposal_creation单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
3. closeout 明确不继续细拆 approval record construction、lifecycle append、transition persistence 或 sandbox trigger call。
4. 下一步固定为 BE-001BQ-01 `runtime.mutation.ai_proposal` 父叶残余判断。
5. 本批次保持 `no code movement`。
