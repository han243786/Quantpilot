# v4.16.0 runtime.mutation.ai_proposal 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AY-04  
> 基准: `181-runtime.mutation.ai_proposal单子叶等价基线.md`、`182-runtime.mutation.ai_proposal抽离方案.md`、`183-runtime.mutation.ai_proposal抽离记录.md`、`src/runtime/mutation/ai_proposal.rs`、`tests/api_ai_proposal.rs`  
> 判定: `runtime.mutation.ai_proposal` 第一轮实际抽离等价成立，但本叶不停止细拆，设置 `stop_split: false`。下一步进入 BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AY 从实际抽离进入单叶 closeout，下一轮进入 static_check 基线 | 收束 |
| 规范矩阵 | 父级 re-export、父子通信、stop_split 判定、approval 锁顺序、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 单叶 closeout |
| 模块树 | `runtime.mutation.ai_proposal` | 设置 `stop_split: false` 并登记下一候选 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.ai_proposal` |
| 父模块 | `backend.runtime` |
| 路由入口 | `backend.runtime.routes.mutation` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/runtime/mutation/ai_proposal.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.ai_proposal` |
| 真实文件 | `src/runtime/mutation/ai_proposal.rs`、`src/runtime/mutation.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/mutation.rs` |
| public 方法 | `create_runtime_ai_proposal`、`list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail`、`list_runtime_approvals`、`get_runtime_approval_detail`、`approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review` |
| 父级声明 | `#[path = "mutation/ai_proposal.rs"] mod mutation_ai_proposal;` |
| 父级出口 | `pub(crate) use mutation_ai_proposal::{approve_ai_proposal, claim_ai_proposal_review, create_runtime_ai_proposal, get_runtime_ai_proposal_detail, get_runtime_approval_detail, list_runtime_ai_proposals, list_runtime_approvals, reject_ai_proposal};` |
| closeout 判定 | `stop_split: false` |
| 下一递归点 | BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线 |

---

## 等价 closeout 结论

| 维度 | 结论 |
| --- | --- |
| route 入口 | 等价。`src/backend/runtime/routes/mutation.rs` 仍只通过 `crate::runtime::{...}` 调用同名 handler |
| 父级出口 | 等价。`src/runtime/mod.rs` 通过 `mutation_ai_proposal` re-export 保持调用面 |
| create flow | 等价。model identity、source context、capability、static check、approval record、sandbox trigger 与 event append 未改变 |
| list/detail | 等价。proposal list/detail 与 approval list/detail 的 filtering、sorting、scoped lookup 和 disk fallback 未改变 |
| approval action | 等价。approve/reject/claim 的 review state guard、reviewer lifecycle、proposal status transition 和 persistence 未改变 |
| sandbox gate | 等价。`ensure_ai_proposal_can_be_approved` 仍要求 config binding、static_check_passed、sandbox report existence 与 passed verdict |
| 状态与锁 | `approval_records -> ai_proposals` lock order 未改变 |
| AppState / schema / frontend caller | 未变更 |
| 发布过渡 | 未启动，不新增横向连接或性能旁路 |

---

## 细分价值判断

**最终判定**: `stop_split: false`。

理由: 本叶已经从 `src/runtime/mutation.rs` 中抽出，但 `src/runtime/mutation/ai_proposal.rs` 仍承接 8 个 public handler 与多组可独立验证的 helper，职责跨度超过单一白箱 owner。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.mutation.ai_proposal.static_check` | 值得拆，下一候选 | `validate_hash_identity`、`is_valid_hash_identity`、`validate_ai_model_identity`、`ai_proposal_static_check_result`、`is_v4_ai_proposal_target`、`expected_config_domain_for_target`、`validate_ai_proposal_config_domain_binding`、`analyze_v4_backtest_artifact_for_ai` 均为低副作用 validation / analysis helper，可先独立白箱化 |
| `runtime.mutation.ai_proposal.source_governance_identity` | 后续候选 | source context、governance snapshot 与 record id 形成稳定输入输出，但依赖 create flow，适合 static_check 后再判断 |
| `runtime.mutation.ai_proposal.event_lifecycle` | 后续候选 | event contract、event build、lifecycle entry 与 proposal transition persistence 形成状态机投影域 |
| `runtime.mutation.ai_proposal.record_query` | 后续候选 | proposal list/detail 与 approval list/detail 是 read model，可独立验证 filtering / sorting / scoped fallback |
| `runtime.mutation.ai_proposal.approval_review` | 后续候选 | approve/reject/claim、sandbox gate、approval persistence 与 status transition 强耦合，需要独立基线保护 `approval_records -> ai_proposals` 锁顺序 |
| `runtime.mutation.ai_proposal.sandbox_trigger` | 暂缓 | 当前仍嵌在 create path 中，涉及 background task、retry、JoinHandle monitoring 和 sandbox report URL 回写，应等 approval_review 边界稳定后再判断 |

下一轮优先拆 `static_check`，因为它是低副作用的策略校验节点，能先把 AI proposal 的 config domain binding、digest validation 和 v4 artifact analysis 从长事务 create flow 中分离出来，且不改变 approval review、state owner 或 route facade。

---

## 父子通信收口

```text
backend.runtime.routes.mutation
  -> crate::runtime::{create_runtime_ai_proposal, list_runtime_ai_proposals, ...}
  -> runtime.mutation.ai_proposal
  -> static_check (next baseline only, no code movement yet)
```

`runtime.mutation.ai_proposal` 只能经父级 `backend.runtime.routes.mutation` 暴露 HTTP route；下一候选 `static_check` 若后续创建，也只能被 `runtime.mutation.ai_proposal` 受控调用。不得横向接管 parameter mutation、report、evidence、experiment、ops、strategy_config、frontend caller、executor、AppState、schema owner、runtime persistence owner 或发布过渡连接。ASCII guard: `release transition guard`。

---

## 本批不做

- 不移动 Rust 代码。
- 不创建 `src/runtime/mutation/ai_proposal/static_check.rs`。
- 不拆 approval review、record query、source governance、event lifecycle、approval persistence 或 sandbox trigger。
- 不迁移 `AppState`、schema owner、frontend caller、route facade、runtime persistence owner 或测试资产。
- 不主动提出发布版本过渡或横向性能连接。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001AY-04 完成时，必须说明本批只完成 `runtime.mutation.ai_proposal` 单叶 closeout，`stop_split: false`，下一步只能进入 BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线。不得宣称 `static_check` 已创建、approval review 已继续拆分、AppState/schema/frontend caller 已改变、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `184-runtime.mutation.ai_proposal单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树标记 `runtime.mutation.ai_proposal` 已完成 closeout，且设置 `stop_split: false`。
3. closeout 明确下一候选为 BE-001AZ-01 `runtime.mutation.ai_proposal.static_check` 单子叶等价基线。
4. closeout 明确本批 `no code movement`，不得迁移 static_check、approval_review、record_query、AppState、schema、frontend caller 或发布过渡连接。
5. 本批验证通过后，后续才能进入 BE-001AZ-01。
