# v4.16.0 runtime.mutation.ai_proposal.static_check 单叶 closeout
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AZ-04  
> 基线: `185-runtime.mutation.ai_proposal.static_check单子叶等价基线.md`、`186-runtime.mutation.ai_proposal.static_check抽离方案.md`、`187-runtime.mutation.ai_proposal.static_check抽离记录.md`、`src/runtime/mutation/ai_proposal/static_check.rs`  
> 判定: `runtime.mutation.ai_proposal.static_check` 第一轮抽离等价成立，设置 `stop_split: true`。本叶只承载 AI proposal candidate 的静态校验 / config binding / v4 artifact analysis helper，继续细拆会增加父子 helper 接线而不会产生新的稳定 owner。父级 `runtime.mutation.ai_proposal` 仍包含其他职责，后续必须进入 BE-001BA-01 父叶残余判断。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AZ-04 static_check 单叶 closeout | 收口 |
| 规范矩阵 | 三档执行、父子通信、`pub(super)` helper、停止细拆判定 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal.static_check` | closeout |
| 模块树 | `runtime.mutation.ai_proposal.static_check` | `stop_split: true` |

---

## 等价结论

`src/runtime/mutation/ai_proposal/static_check.rs` 已承接以下 helper:

- `validate_hash_identity`
- `is_valid_hash_identity`
- `validate_ai_model_identity`
- `ai_proposal_static_check_result`
- `is_v4_ai_proposal_target`
- `expected_config_domain_for_target`
- `validate_ai_proposal_config_domain_binding`
- `analyze_v4_backtest_artifact_for_ai`

父级 `src/runtime/mutation/ai_proposal.rs` 只通过以下受控连接调用 child:

```rust
#[path = "ai_proposal/static_check.rs"]
mod static_check;

use static_check::{
    ai_proposal_static_check_result, validate_ai_model_identity, validate_hash_identity,
};
```

`validate_hash_identity` 在 BE-001AZ-03 编译期真实边界校验中确认仍被 create flow 直接调用，因此为 `pub(super)`；这不是横向连接，也不是发布过渡连接。

---

## stop_split 判定

| 细分候选 | 判定 | 原因 |
| --- | --- | --- |
| hash identity | 不继续拆 | 只服务 static check / create admission，拆出单独 child 只会增加 import |
| model identity | 不继续拆 | 只有 provider/model/model_version 必填校验，无独立 owner |
| config domain binding | 不继续拆 | 与 static check aggregate 强耦合，输出仍是 `RuntimeAiProposalStaticCheckDetail` |
| v4 artifact analysis | 不继续拆 | 当前只由 child 内部测试覆盖，尚未成为 runtime evidence owner |

因此本叶设置:

```text
stop_split: true
```

---

## 未完成的父级残余

`runtime.mutation.ai_proposal` 父级仍不能宣称完成。后续至少还需要在父叶残余判断中处理:

- source_governance_identity
- event_lifecycle
- record_query
- approval_review
- approval_persistence
- sandbox_trigger

下一步只能进入:

```text
BE-001BA-01 runtime.mutation.ai_proposal 父叶残余判断
```

不得直接迁移 approval review、record query、AppState、schema owner、frontend caller、route facade、runtime persistence owner 或 release transition guard。

---

## 验证记录

BE-001AZ-03 抽离后已验证:

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

- `188-runtime.mutation.ai_proposal.static_check单叶closeout.md`
- 里程碑索引 / 落地记录 / 递归流程中的 BE-001AZ-04 状态行
- 模块树中本叶 `stop_split: true` closeout 状态
- 全量树 / overview / governance checker 中本批索引

不得回退 BE-001AZ-03 已完成的 `static_check.rs` 代码抽离。

---

## 幻觉检查点

AI 声称 BE-001AZ-04 完成时，必须说明只完成 `runtime.mutation.ai_proposal.static_check` 单叶 closeout，并设置 `stop_split: true`。不得宣称 `runtime.mutation.ai_proposal` 父级已完成、approval review 已拆分、record query 已拆分、AppState/schema/frontend caller 已改变、release transition 已启动或 Rust backend 重构已完成。

---

## 验收标准

1. `188-runtime.mutation.ai_proposal.static_check单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树将 `runtime.mutation.ai_proposal.static_check` 标记为 `stop_split: true`。
3. 后续路径明确回到 `runtime.mutation.ai_proposal` 父叶残余判断。
4. 本批不产生代码变更，不改变 BE-001AZ-03 的抽离结果。
