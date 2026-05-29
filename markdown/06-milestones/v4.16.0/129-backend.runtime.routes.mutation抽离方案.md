# v4.16.0 backend.runtime.routes.mutation 抽离方案

> 判定: 建立 BE-001AE-02 `backend.runtime.routes.mutation` 抽离方案。当前仍为 `no code movement`；本批只规划 route facade 迁移，不移动 `src/runtime/mutation.rs` handler、不修改 `src/runtime/mod.rs` facade、不改变 `AppState`、`approval_records -> ai_proposals` 锁顺序、response schema、frontend caller 或发布过渡连接。下一步只能进入 BE-001AE-03 实际抽离记录。

---

## 提案

`backend.runtime.routes.mutation` 已在 BE-001AE-01 完成单子叶等价基线。BE-001AE-02 的目标是把下一批实际抽离限制为一个最小 route facade:

| 项 | 内容 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.mutation` |
| 父模块 | `backend.runtime.routes` |
| 当前 route aggregate | `src/backend/runtime/routes.rs` |
| planned route facade | `src/backend/runtime/routes/mutation.rs` |
| handler owner | `src/runtime/mutation.rs` |
| handler facade | `src/runtime/mod.rs` |
| 下一批次 | BE-001AE-03 实际抽离记录 |

1. 新建计划文件 `src/backend/runtime/routes/mutation.rs`。
2. 在 `src/backend/runtime/routes.rs` 中登记 `pub mod mutation`。
3. 在父级 `register_routes` 中通过 `mutation::register_routes(router)` 委托 mutation / AI proposal / approval route group。
4. 保持所有 handler 继续指向 `runtime_handlers::*`，不迁移 `src/runtime/mutation.rs` 内的任何函数。

---

## 适配性校验

| 坐标 | 当前 owner | BE-001AE-03 允许动作 | 禁止动作 |
| --- | --- | --- | --- |
| 父级 route aggregate | `src/backend/runtime/routes.rs` | 增加 `pub mod mutation` 并委托 route child | 不移动 event/evidence/report/experiment/ops |
| 子级 route facade | planned `src/backend/runtime/routes/mutation.rs` | 承接 11 条 mutation / AI proposal / approval route | 不增加新 route path |
| handler owner | `src/runtime/mutation.rs` | 保持原位 | 不拆分、不移动、不改签名 |
| runtime facade | `src/runtime/mod.rs` | 保持原 include/re-export 结构 | 不改变 public handler 出口 |
| state owner | `AppState` | 保持原字段与锁 | 不迁移 mutation ledger、approval records 或 AI proposals |
| lock order | `approval_records -> ai_proposals` | 保持顺序 | 不反转锁顺序 |
| frontend caller | frontend runtime mutation / approval caller | 保持 API contract | 不改 endpoint、schema、fixture |
| release transition guard | 开发者未声明发布过渡 | 保持禁止主动发布优化 | 不提横向连接或性能旁路 |

---

## 方案优化

首选方案: route facade 最小迁移。

```text
backend.runtime.routes
  -> backend.runtime.routes.run
  -> backend.runtime.routes.backtest
  -> inline event/evidence routes
  -> backend.runtime.routes.mutation
  -> inline report/experiment/ops routes

backend.runtime.routes.mutation
  -> runtime_handlers::{mutation, ai proposal, approval handlers}
  -> src/runtime/mutation.rs
  -> AppState / mutation ledger / approval_records / ai_proposals
```

不采用 handler 抽离方案。理由: handler 文件体量大且同时承载 mutation ledger、AI proposal、approval transition、static check、capability gate、audit 和 safe window 逻辑；如果在 route facade 批次同时拆 handler，会扩大缺口并降低等价检查清晰度。

不采用发布过渡优化方案。理由: 开发者未明确进入发布版本过渡，AI 不得主动提出横向连接、缓存旁路或性能优化。

---

## BE-001AE-03 允许清单

1. 新建 `src/backend/runtime/routes/mutation.rs`，结构仿照 `src/backend/runtime/routes/run.rs`。
2. 文件内只允许:
   - `use axum::{routing::{get, post}, Router};`
   - `use crate::{runtime as runtime_handlers, AppState};`
   - `pub const MODULE_ID: &str = "backend.runtime.routes.mutation";`
   - `pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>`
3. 只迁入下列 route registration:
   - `GET/POST /api/runtime/mutations`
   - `GET /api/runtime/mutations/:proposal_id`
   - `POST /api/runtime/mutations/:proposal_id/activate`
   - `POST /api/runtime/mutations/:proposal_id/rollback`
   - `GET/POST /api/runtime/ai-proposals`
   - `GET /api/runtime/ai-proposals/:ai_proposal_id`
   - `GET /api/v1/ai/approvals`
   - `GET /api/v1/ai/approvals/:approval_id`
   - `POST /api/v1/ai/proposals/:proposal_id/approve`
   - `POST /api/v1/ai/proposals/:proposal_id/reject`
   - `POST /api/v1/ai/proposals/:proposal_id/claim`
4. 父级 `src/backend/runtime/routes.rs` 只允许:
   - 增加 `pub mod mutation;`
   - 在 event/evidence routes 后、report/experiment/ops routes 前调用 `mutation::register_routes(router)`
   - 删除被迁出的重复 inline route registration

---

## 禁止清单

- 不迁移 `src/runtime/mutation.rs` 的 `create_runtime_parameter_mutation`、`list_runtime_parameter_mutations`、`get_runtime_parameter_mutation_detail`、`activate_runtime_parameter_mutation`、`rollback_runtime_parameter_mutation`。
- 不迁移 `create_runtime_ai_proposal`、`list_runtime_ai_proposals`、`get_runtime_ai_proposal_detail`。
- 不迁移 `list_runtime_approvals`、`get_runtime_approval_detail`、`approve_ai_proposal`、`reject_ai_proposal`、`claim_ai_proposal_review`。
- 不修改 `src/runtime/mod.rs` 的 `include!("mutation.rs")`。
- 不修改 `AppState`、mutation ledger、approval records、AI proposal storage、capability context 或 `approval_records -> ai_proposals` 锁顺序。
- 不改 response schema、frontend caller、fixtures、测试资产、发布过渡协议。
- 不把 report、evidence、experiment、ops/storage/config route 混入本子叶。

---

## 回退点

BE-001AE-03 如果失败，回退只需要删除 planned `src/backend/runtime/routes/mutation.rs`，移除 `pub mod mutation` 和 `mutation::register_routes(router)`，并把 11 条 route registration 放回 `src/backend/runtime/routes.rs` 原位置。不得通过回退改动 handler、schema、state 或测试资产。

---

## 验证清单

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 新 route facade 格式不漂移 |
| `cargo check -p quantpilot` | Rust route 类型 | 父级委托与 handler 引用可编译 |
| `cargo test --no-run` | 测试编译 | mutation / AI proposal / approval 测试仍可编译 |
| `cargo test -p quantpilot --test api_mutation` | mutation route group | path/method/handler 等价 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal route group | static check、capability gate、audit 等价 |
| `cargo test -p quantpilot --test api_evidence_contract` | 邻接 evidence route | evidence health 与 cleanup 不被误伤 |
| `cargo test -p quantpilot --test api_run` | 邻接 run/report route | route aggregate 邻接行为不漂移 |
| `tools\check-utf8.ps1` | 编码 | 新增/修改文档保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 129 号方案、模块树和全量树引用不缺失 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 文档引用不制造不存在的 active path |
| `git diff --check` | diff whitespace | 空白无错误 |

---

## 下一步

1. BE-001AE-03 只能执行上述 route facade 最小物理抽离。
2. BE-001AE-03 完成后必须先做单叶整理 / closeout，再判断 `backend.runtime.routes.mutation` 是否值得继续向 handler 内部细拆。
3. 若要细拆 handler，必须另起子叶等价基线，不能直接从 route facade 抽离批次跳入 handler 拆分。

---

## 幻觉检查点

AI 声称 BE-001AE-02 完成时，必须说明: 本批只是 `backend.runtime.routes.mutation` 抽离方案，仍为 `no code movement`；route facade 尚未创建，mutation route 尚未抽离，`src/runtime/mutation.rs`、`src/runtime/mod.rs`、`AppState`、锁顺序、schema、frontend caller 和发布过渡均未改变。不得宣称 BE-001AE-03 已完成、mutation handler 已拆分、`backend.runtime.routes` 父叶完成、整理或重构已经完成。

---

## 验收标准

1. `129-backend.runtime.routes.mutation抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树明确 BE-001AE-03 只能做 route facade 最小抽离。
3. 治理门禁能发现 BE-001AE-02、BE-001AE-03、planned route facade、父级委托、禁止迁移边界和测试证据缺失。
