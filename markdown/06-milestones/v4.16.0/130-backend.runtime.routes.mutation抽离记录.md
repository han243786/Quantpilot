# v4.16.0 backend.runtime.routes.mutation 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 重型  
> 批次: BE-001AE-03  
> 基准: `129-backend.runtime.routes.mutation抽离方案.md`、`128-backend.runtime.routes.mutation单子叶等价基线.md`、`127-backend.runtime.routes父叶残余判断.md`  
> 判定: 按方案完成 `backend.runtime.routes.mutation` 第一轮实际抽离；只将 mutation / AI proposal / approval route registration 迁入 `src/backend/runtime/routes/mutation.rs`，不迁移 `src/runtime/mutation.rs` handler、`src/runtime/mod.rs` facade、`AppState`、`approval_records -> ai_proposals` 锁顺序、schema、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AE 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级委托、子 route facade、父子通信、禁止横向连接 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.mutation` | 物理抽离 |
| 模块树 | `backend.runtime.routes.mutation` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.mutation` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes/mutation.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.mutation` |
| 父模块 | `backend.runtime.routes` |
| 新真实文件 | `src/backend/runtime/routes/mutation.rs` |
| 父级真实文件 | `src/backend/runtime/routes.rs` |
| handler owner | `src/runtime/mutation.rs` |
| handler facade | `src/runtime/mod.rs` |
| 状态 owner | `AppState` |
| 锁顺序 | `approval_records -> ai_proposals` |
| public 方法 | `backend.runtime.routes.mutation::register_routes` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_mutation`、`cargo test -p quantpilot --test api_ai_proposal`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 mutation route facade | `src/backend/runtime/routes/mutation.rs` | 承载 mutation / AI proposal / approval route group |
| 父级声明 route child | `src/backend/runtime/routes.rs` | 增加 `pub mod mutation;` |
| 父级委托 route child | `src/backend/runtime/routes.rs` | 增加 `mutation::register_routes(router)` |
| 移除父级 inline registration | `src/backend/runtime/routes.rs` | 11 条 path registration / 13 个 method 绑定从父级 inline 链迁出 |
| 保留 handler owner | `src/runtime/mutation.rs` | 所有 handler 仍在原文件 |
| 保留 handler facade | `src/runtime/mod.rs` | `include!("mutation.rs")` 不变 |

父级形态:

```rust
pub mod backtest;
pub mod mutation;
pub mod run;

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    let router = backtest::register_routes(router);
    let router = run::register_routes(router);
    let router = router
        .route("/api/runtime/runs/:run_id/events", get(runtime_handlers::stream_run_events))
        .route("/api/runtime/evidence/health", get(runtime_handlers::get_runtime_evidence_health))
        .route("/api/runtime/evidence/cleanup", post(runtime_handlers::cleanup_runtime_evidence));
    let router = mutation::register_routes(router);
    router
        .route("/api/runtime/reports", get(runtime_handlers::list_runtime_reports).post(runtime_handlers::create_runtime_report))
}
```

子 facade 形态:

```rust
pub const MODULE_ID: &str = "backend.runtime.routes.mutation";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/runtime/mutations", get(runtime_handlers::list_runtime_parameter_mutations).post(runtime_handlers::create_runtime_parameter_mutation))
        .route("/api/v1/ai/proposals/:proposal_id/claim", post(runtime_handlers::claim_ai_proposal_review))
}
```

---

## 迁入 route group

| route | method | handler | 等价要求 |
| --- | --- | --- | --- |
| `/api/runtime/mutations` | GET | `list_runtime_parameter_mutations` | list/filter 语义不变 |
| `/api/runtime/mutations` | POST | `create_runtime_parameter_mutation` | proposal、safe window、audit 不变 |
| `/api/runtime/mutations/:proposal_id` | GET | `get_runtime_parameter_mutation_detail` | scoped lookup 不变 |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` | activation ledger 不变 |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` | rollback target 不变 |
| `/api/runtime/ai-proposals` | GET | `list_runtime_ai_proposals` | audit projection 不变 |
| `/api/runtime/ai-proposals` | POST | `create_runtime_ai_proposal` | static check 与 capability gate 不变 |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `get_runtime_ai_proposal_detail` | candidate diagnostics 不变 |
| `/api/v1/ai/approvals` | GET | `list_runtime_approvals` | approval visibility 不变 |
| `/api/v1/ai/approvals/:approval_id` | GET | `get_runtime_approval_detail` | approval detail 不变 |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `approve_ai_proposal` | approval lock order 不变 |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `reject_ai_proposal` | rejection reason 不变 |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `claim_ai_proposal_review` | reviewer claim semantics 不变 |

---

## 明确未迁移

- 不迁移 `src/runtime/mutation.rs` 中任何 handler/helper。
- 不修改 `src/runtime/mod.rs` 的 `include!("mutation.rs")`。
- 不修改 `AppState`、mutation ledger、approval records、AI proposal storage 或 `approval_records -> ai_proposals` 锁顺序。
- 不修改 response schema、frontend caller、fixtures、测试资产或发布过渡协议。
- 不把 report、evidence、experiment、ops/storage/config route 混入本子叶。
- 不主动提出发布过渡、横向连接、缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 回退点

若发现 route 行为回归，可删除 `src/backend/runtime/routes/mutation.rs`，移除 `src/backend/runtime/routes.rs` 中的 `pub mod mutation;` 与 `mutation::register_routes(router)`，并把 11 条 path registration / 13 个 method 绑定放回父级原位置。回退不需要修改 handler、schema、state、frontend 或测试资产。

---

## 验证计划

本批收口必须运行:

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

## 下一步

下一批应进入 BE-001AE-04 `backend.runtime.routes.mutation` 单叶 closeout，确认 route facade 等价并判断该 facade 是否停止细分。当前不能直接拆 `src/runtime/mutation.rs` handler；若 closeout 后认为 handler 内部值得继续细拆，必须另起子叶等价基线。

---

## 幻觉检查点

AI 声称 BE-001AE-03 完成时，必须说明: 本批只完成 route facade 抽离，`src/runtime/mutation.rs` handler、`src/runtime/mod.rs` facade、`AppState`、`approval_records -> ai_proposals` 锁顺序、schema、frontend caller 和发布过渡均未改变。不得宣称 mutation handler 已拆分、`backend.runtime.routes` 父叶完成、单叶 closeout 已完成、整理或重构已经完成。

---

## 验收标准

1. `130-backend.runtime.routes.mutation抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/backend/runtime/routes/mutation.rs` 进入全量树和模块树真实文件。
3. `src/backend/runtime/routes.rs` 只保留父级委托，不再 inline 注册 mutation / AI proposal / approval route group。
4. `src/runtime/mutation.rs` 和 `src/runtime/mod.rs` 不发生 handler 迁移。
5. 治理门禁能发现本抽离记录、真实文件、父级委托、关键 route/handler、禁止迁移边界和回归证据缺失。
