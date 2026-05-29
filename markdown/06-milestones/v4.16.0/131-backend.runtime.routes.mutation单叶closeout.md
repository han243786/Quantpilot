# v4.16.0 backend.runtime.routes.mutation 单叶 closeout

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AE-04  
> 基准: `128-backend.runtime.routes.mutation单子叶等价基线.md`、`129-backend.runtime.routes.mutation抽离方案.md`、`130-backend.runtime.routes.mutation抽离记录.md`  
> 判定: `backend.runtime.routes.mutation` route facade 已完成等价 closeout，route facade 本身设置 `stop_split: true`；`src/runtime/mutation.rs` handler 域仍值得另起 `runtime.mutation.parameter_mutation` 单子叶等价基线继续递归。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AE 从实际抽离进入单叶 closeout，下一轮进入 handler 域基线 | 收束 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、handler owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.mutation` | closeout |
| 模块树 | `backend.runtime.routes.mutation` | 设置 `stop_split: true` 并登记下一候选 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.mutation` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes/mutation.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.mutation` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/backend/runtime/routes/mutation.rs`、`src/runtime/mutation.rs`、`src/runtime/mod.rs` |
| public 方法 | `backend.runtime.routes.mutation::register_routes`、`create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail`、`activate_runtime_parameter_mutation`、`rollback_runtime_parameter_mutation`、`create_runtime_ai_proposal`、`list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail`、`list_runtime_approvals`、`get_runtime_approval_detail`、`approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_mutation`、`cargo test -p quantpilot --test api_ai_proposal`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、`git diff --check` |

---

## 等价判定

| 检查项 | 结论 |
| --- | --- |
| route path | 等价。11 条 path registration 保持不变 |
| route method | 等价。13 个 method 绑定保持不变 |
| handler 调用 | 等价。继续调用 `runtime_handlers::*` |
| 父级委托 | 等价。`src/backend/runtime/routes.rs` 通过 `mutation::register_routes(router)` 接入 |
| response schema | 未变更 |
| error code | 未变更 |
| AppState / 锁顺序 | 未变更，`approval_records -> ai_proposals` 仍按原顺序 |
| frontend caller | 未变更 |
| 发布过渡 | 未启动，不新增横向连接或性能旁路 |

---

## 真实 route facade 结果

| route | method | 当前 owner | handler |
| --- | --- | --- | --- |
| `/api/runtime/mutations` | GET | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::list_runtime_parameter_mutations` |
| `/api/runtime/mutations` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::create_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id` | GET | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::get_runtime_parameter_mutation_detail` |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::activate_runtime_parameter_mutation` |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::rollback_runtime_parameter_mutation` |
| `/api/runtime/ai-proposals` | GET | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::list_runtime_ai_proposals` |
| `/api/runtime/ai-proposals` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::create_runtime_ai_proposal` |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::get_runtime_ai_proposal_detail` |
| `/api/v1/ai/approvals` | GET | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::list_runtime_approvals` |
| `/api/v1/ai/approvals/:approval_id` | GET | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::get_runtime_approval_detail` |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::approve_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::reject_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `src/backend/runtime/routes/mutation.rs` | `runtime_handlers::claim_ai_proposal_review` |

---

## 保留边界

- `src/runtime/mutation.rs` 仍拥有所有 mutation / AI proposal / approval handler 与 helper。
- `src/runtime/mod.rs` 仍通过 `include!("mutation.rs")` 暴露兼容 handler。
- `AppState`、mutation ledger、approval records、AI proposals、runtime run store 和持久化目录 owner 均不迁移。
- `approval_records -> ai_proposals` 锁顺序保持不变。
- response schema、frontend caller、fixture、测试资产和发布过渡协议均不迁移。
- report、evidence、experiment、ops/storage/config routes 不属于本子叶。
- 不主动提出发布版本过渡，不新增子模块横向连接。ASCII guard: `release transition guard`。

---

## 细分价值判断

`backend.runtime.routes.mutation` route facade 本身停止细分，`stop_split: true`。它只承担 route registration；继续把 58 行 facade 拆成 mutation routes、AI proposal routes 和 approval routes 会制造更碎的父级导入面，但不会形成新的稳定 owner。

`src/runtime/mutation.rs` handler 域值得继续细拆，原因如下:

| 候选 | 价值判断 | 理由 |
| --- | --- | --- |
| `runtime.mutation.parameter_mutation` | 值得拆 | create/list/detail/activate/rollback、safe window、activation/rollback event 和 parameter version owner 边界清楚，`api_mutation` 覆盖强 |
| `runtime.mutation.ai_proposal` | 值得拆 | AI proposal static check、config domain binding、sandbox/source context 和 candidate audit 独立，`api_ai_proposal` 覆盖强 |
| `runtime.mutation.approval_review` | 值得拆 | approval list/detail/approve/reject/claim 与 reviewer claim、approval transition 和锁顺序强相关 |
| `runtime.mutation.shared_persistence_governance` | 暂缓 | transition persistence、governance/lifecycle entry、disk load/save helper 被三组 handler 共用，应在前三个子叶边界稳定后再判断 |

默认下一候选为 `runtime.mutation.parameter_mutation`，因为它是 mutation handler 域的第一条 lifecycle 主线，已有最完整的 `api_mutation` 等价证据。

---

## 下一步

下一批进入 BE-001AF-01 `runtime.mutation.parameter_mutation` 单子叶等价基线。该批只允许建立基线，不直接迁移代码；必须先冻结 create/list/detail/activate/rollback、safe window、activation boundary、parameter version canonicalization、mutation event、rollback target、runtime run record append 和 `api_mutation` 证据。

不得直接移动 AI proposal、approval review、shared persistence/governance helper、AppState、锁顺序、schema、frontend caller、report/evidence/experiment/ops route 或发布过渡连接。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001AE-04 完成时，必须说明只完成 `backend.runtime.routes.mutation` route facade closeout，并设置 route facade `stop_split: true`。不得宣称 `src/runtime/mutation.rs` handler 已拆分、parameter mutation handler 已迁移、AI proposal/approval 状态 owner 已迁移、AppState 或锁顺序已改变、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `131-backend.runtime.routes.mutation单叶closeout.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 `backend.runtime.routes.mutation` route facade 设置 `stop_split: true`。
3. closeout 明确 `src/runtime/mutation.rs` handler 域仍值得继续递归，默认下一候选为 `runtime.mutation.parameter_mutation`。
4. closeout 明确 handler、AppState、锁顺序、schema、frontend caller、report/evidence/experiment/ops 和发布过渡均未迁移。
5. 本批验证通过后，后续才能进入 BE-001AF-01 等价基线。
