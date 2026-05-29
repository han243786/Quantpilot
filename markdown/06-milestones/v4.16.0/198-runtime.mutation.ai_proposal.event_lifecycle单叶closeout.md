# v4.16.0 runtime.mutation.ai_proposal.event_lifecycle 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BD-04  
> 基线: `195-runtime.mutation.ai_proposal.event_lifecycle单子叶等价基线.md`、`196-runtime.mutation.ai_proposal.event_lifecycle抽离方案.md`、`197-runtime.mutation.ai_proposal.event_lifecycle抽离记录.md`、`src/runtime/mutation/ai_proposal/event_lifecycle.rs`  
> 判定: `runtime.mutation.ai_proposal.event_lifecycle` 第一轮抽离等价成立，设置 `stop_split: true`。本叶只承载 AI proposal 状态投影链路中的 event contract、runtime event builder、lifecycle entry 与 proposal transition persistence helper；继续细拆会增加父子 helper 接线而不会产生新的稳定 owner。父级 `runtime.mutation.ai_proposal` 仍包含其他职责，后续必须进入 BE-001BE-01 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明
| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BD-04 event_lifecycle 单叶 closeout | 收口 |
| 规范矩阵 | 三档执行、父子通信、`pub(super)` helper、停止细拆判定 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.event_lifecycle` | closeout |
| 模块树 | `runtime.mutation.ai_proposal.event_lifecycle` | `stop_split: true` |

---

## 等价结论

`src/runtime/mutation/ai_proposal/event_lifecycle.rs` 已承接以下 helper:

- `ai_proposal_event_contract`
- `build_runtime_ai_proposal_event`
- `ai_proposal_lifecycle_entry`
- `persist_runtime_ai_proposal_transition`

父级 `src/runtime/mutation/ai_proposal.rs` 只通过以下受控连接调用 child:

```rust
#[path = "ai_proposal/event_lifecycle.rs"]
mod event_lifecycle;

use event_lifecycle::{
    ai_proposal_lifecycle_entry, build_runtime_ai_proposal_event,
    persist_runtime_ai_proposal_transition,
};
```

`ai_proposal_event_contract` 保持 child private；三个父级可调用 helper 保持 `pub(super)`，未形成 sibling 横向连接，也不是 release transition 连接。

---

## stop_split 判定

| 细分候选 | 判定 | 原因 |
| --- | --- | --- |
| event contract | 不继续拆 | 只服务 event builder 与 lifecycle entry，独立成文件不会产生新 owner |
| runtime event builder | 不继续拆 | payload、severity、summary、envelope 共同描述同一 proposal event 投影 |
| lifecycle entry | 不继续拆 | 与 event contract 共享 reason_code，拆开只会增加 helper 接线 |
| proposal transition persistence | 不继续拆 | 写 disk 与写 `state.ai_proposals` 是同一次 proposal transition 的收尾动作，storage owner 仍保留在既有 persistence helper |

因此本叶设置:

```text
stop_split: true
```

---

## 未完成的父级残余

`runtime.mutation.ai_proposal` 父级仍不能宣称完成。后续至少还需要在父叶残余判断中处理:

- record_query
- approval_review
- approval_persistence
- sandbox_trigger
- status_transition

下一步只能进入:

```text
BE-001BE-01 runtime.mutation.ai_proposal 父叶残余判断
```

不得直接迁移 record query、approval review、approval persistence、sandbox trigger、status transition、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 验证记录

BE-001BD-03 抽离后已验证:

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

- `198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md`
- 里程碑索引 / 落地记录 / 递归流程中的 BE-001BD-04 状态行
- 模块树中本叶 `stop_split: true` closeout 状态
- 全量树 / overview / governance checker 中本批索引

不得回退 BE-001BD-03 已完成的 `event_lifecycle.rs` 代码抽离。

---

## 幻觉检查点

AI 声称 BE-001BD-04 完成时，必须说明只完成 `runtime.mutation.ai_proposal.event_lifecycle` 单叶 closeout，并设置 `stop_split: true`。不得宣称 `runtime.mutation.ai_proposal` 父级已完成、record query 已拆分、approval review 已拆分、approval persistence 已拆分、sandbox trigger 已迁移、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `198-runtime.mutation.ai_proposal.event_lifecycle单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.event_lifecycle` 标记为 `stop_split: true`。
3. 后续路径明确回到 `runtime.mutation.ai_proposal` 父叶残余判断。
4. 本批不产生代码变更，不改变 BE-001BD-03 的抽离结果。
