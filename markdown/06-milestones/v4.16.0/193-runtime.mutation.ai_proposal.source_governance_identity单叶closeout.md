# v4.16.0 runtime.mutation.ai_proposal.source_governance_identity 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BB-04  
> 基线: `190-runtime.mutation.ai_proposal.source_governance_identity单子叶等价基线.md`、`191-runtime.mutation.ai_proposal.source_governance_identity抽离方案.md`、`192-runtime.mutation.ai_proposal.source_governance_identity抽离记录.md`、`src/runtime/mutation/ai_proposal/source_governance_identity.rs`  
> 判定: `runtime.mutation.ai_proposal.source_governance_identity` 第一轮抽离等价成立，设置 `stop_split: true`。本叶只承载 AI proposal create flow 的 source context、governance projection 与 proposal record identity helper，继续细拆会增加父子 helper 接线而不会产生新的稳定 owner。父级 `runtime.mutation.ai_proposal` 仍包含其他职责，后续必须进入 BE-001BC-01 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BB-04 source_governance_identity 单叶 closeout | 收口 |
| 规范矩阵 | 三档执行、父子通信、`pub(super)` struct / field、停止细拆判定 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.source_governance_identity` | closeout |
| 模块树 | `runtime.mutation.ai_proposal.source_governance_identity` | `stop_split: true` |

---

## 等价结论

`src/runtime/mutation/ai_proposal/source_governance_identity.rs` 已承接以下 helper:

- `RuntimeAiProposalSourceContext`
- `load_runtime_ai_proposal_source_context`
- `runtime_ai_proposal_governance`
- `runtime_ai_proposal_record_id`

父级 `src/runtime/mutation/ai_proposal.rs` 只通过以下受控连接调用 child:

```rust
#[path = "ai_proposal/source_governance_identity.rs"]
mod source_governance_identity;

use source_governance_identity::{
    load_runtime_ai_proposal_source_context, runtime_ai_proposal_governance,
    runtime_ai_proposal_record_id,
};
```

`RuntimeAiProposalSourceContext` 与字段均保持 `pub(super)`，只允许父级 create flow 受控读取；未形成 sibling 横向连接，也不是 release transition 连接。

---

## stop_split 判定

| 细分候选 | 判定 | 原因 |
| --- | --- | --- |
| source loader | 不继续拆 | run/backtest 分支只读取源 record 并投影相同 context，拆出 child 只会增加 import |
| governance projection | 不继续拆 | 只是将 source governance 映射为 proposal governance，无独立状态 owner |
| record identity | 不继续拆 | 只服务 proposal record id digest，输出仍绑定 create flow |
| context struct | 不继续拆 | 字段全部由父级 create flow 一次性消费，不形成独立 public owner |

因此本叶设置:

```text
stop_split: true
```

---

## 未完成的父级残余

`runtime.mutation.ai_proposal` 父级仍不能宣称完成。后续至少还需要在父叶残余判断中处理:

- event_lifecycle
- record_query
- approval_review
- approval_persistence
- sandbox_trigger
- status_transition

下一步只能进入:

```text
BE-001BC-01 runtime.mutation.ai_proposal 父叶残余判断
```

不得直接迁移 approval review、record query、event lifecycle、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 验证记录

BE-001BB-03 抽离后已验证:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test --no-run`
- `cargo test -p quantpilot --test api_ai_proposal`
- `cargo test -p quantpilot --test api_mutation`
- `cargo test -p quantpilot --test api_evidence_contract`
- `cargo test -p quantpilot --test api_run`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `git diff --check`

本 closeout 批次为 `no code movement`，只更新治理记录与模块树状态。

---

## 回退点

若本 closeout 记录导致治理门禁失败，只允许回退本批文档变更:

- `193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md`
- 里程碑索引 / 落地记录 / 递归流程中的 BE-001BB-04 状态行
- 模块树中本叶 `stop_split: true` closeout 状态
- 全量树 / overview / governance checker 中本批索引

不得回退 BE-001BB-03 已完成的 `source_governance_identity.rs` 代码抽离。

---

## 幻觉检查点

AI 声称 BE-001BB-04 完成时，必须说明只完成 `runtime.mutation.ai_proposal.source_governance_identity` 单叶 closeout，并设置 `stop_split: true`。不得宣称 `runtime.mutation.ai_proposal` 父级已完成、event lifecycle 已拆分、approval review 已拆分、record query 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `193-runtime.mutation.ai_proposal.source_governance_identity单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.source_governance_identity` 标记为 `stop_split: true`。
3. 后续路径明确回到 `runtime.mutation.ai_proposal` 父叶残余判断。
4. 本批不产生代码变更，不改变 BE-001BB-03 的抽离结果。
