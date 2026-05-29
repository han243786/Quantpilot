# v4.16.0 backend.runtime.routes.mutation 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 标准。  
> 批次: BE-001AE-01。  
> 基准: `127-backend.runtime.routes父叶残余判断.md`、`50-backend.runtime.routes单子叶等价基线.md`、`51-backend.runtime.routes抽离记录.md`、`13-递归模块化全局根流程.md`。  
> 判定: 建立 `backend.runtime.routes.mutation` 单子叶等价基线。当前只冻结 mutation / AI proposal / approval route group 的 path、method、handler owner、输入输出、锁顺序、测试证据和禁止迁移边界；本批 `no code movement`。下一步只能进入 BE-001AE-02 抽离方案。  
> 代码动作: `no code movement`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AE-01 mutation route group 单子叶等价基线 | 扩展 |
| 规范矩阵 | route owner、handler owner、approval lock order、父子通信 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.mutation` | 新增白箱节点 |
| 模块树 | `backend.runtime.routes.mutation` | 建立单子叶基线 |

---

## 选择理由

`backend.runtime.routes.mutation` 是 `backend.runtime.routes` 父叶残余中最高价值的下一候选:

1. `src/runtime/mutation.rs` 约 2490 行，是当前 runtime route 残余里体量最大、耦合最高的 handler owner。
2. 它集中 runtime parameter mutation、AI proposal、approval、approve/reject/claim 等治理动作，涉及审计、capability gate、sandbox/static check 和锁顺序。
3. 它已有 `api_mutation` 与 `api_ai_proposal` 强回归证据，适合先冻结等价基线再进入抽离方案。
4. 本批只建立基线，不移动 route 或 handler，能把后续实际抽离限制在清晰父子通信范围内。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.mutation` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `backend.runtime.routes.mutation` |
| 父模块 | `backend.runtime.routes` |
| route aggregate | `src/backend/runtime/routes.rs` |
| handler owner | `src/runtime/mutation.rs` |
| handler facade | `src/runtime/mod.rs` |
| app state owner | `AppState` |
| 下一批次 | BE-001AE-02 抽离方案 |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `RuntimeParameterMutationRequest` | frontend、API caller、tests | 不改变参数版本、capability context、safe window 或 no-op rejection 语义 |
| 输入 | AI proposal request | frontend、AI proposal caller、tests | 必须保留 static check、strategy config domain binding 和 capability gate |
| 输入 | approval id / proposal id | path param | 不改变 scoped lookup、approval detail 或 claim/approve/reject target |
| 输入 | `AppState` | backend runtime state | 不迁移 `approval_records`、`ai_proposals`、mutation ledger 或锁顺序 |
| 输出 | mutation proposal/detail/list | frontend、tests | 不改变 response schema、status、audit events 或 rollback metadata |
| 输出 | AI proposal record/detail/list | frontend、tests | 不改变 static check failure、candidate audit 或 key event |
| 输出 | approval list/detail/action response | frontend、tests | 不改变 approval state transition、claim owner 或 rejection reason |

---

## route owner 基线

| route | method | handler | 当前处理 |
| --- | --- | --- | --- |
| `/api/runtime/mutations` | GET | `list_runtime_parameter_mutations` | 冻结 path/method，不移动 handler |
| `/api/runtime/mutations` | POST | `create_runtime_parameter_mutation` | 冻结 request/response，不移动 handler |
| `/api/runtime/mutations/:proposal_id` | GET | `get_runtime_parameter_mutation_detail` | 冻结 path param，不移动 handler |
| `/api/runtime/mutations/:proposal_id/activate` | POST | `activate_runtime_parameter_mutation` | 冻结 activation/audit 语义 |
| `/api/runtime/mutations/:proposal_id/rollback` | POST | `rollback_runtime_parameter_mutation` | 冻结 rollback ledger 语义 |
| `/api/runtime/ai-proposals` | GET | `list_runtime_ai_proposals` | 冻结 list response |
| `/api/runtime/ai-proposals` | POST | `create_runtime_ai_proposal` | 冻结 static check / capability gate |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `get_runtime_ai_proposal_detail` | 冻结 detail response |
| `/api/v1/ai/approvals` | GET | `list_runtime_approvals` | 冻结 approval list |
| `/api/v1/ai/approvals/:approval_id` | GET | `get_runtime_approval_detail` | 冻结 approval detail |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `approve_ai_proposal` | 冻结 approve transition |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `reject_ai_proposal` | 冻结 reject transition |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `claim_ai_proposal_review` | 冻结 claim transition |

---

## 关键 public 方法

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `backend.runtime.routes::register_routes` | Axum Router | runtime routes | `backend.runtime` | 不得在本批新建 mutation route child |
| `create_runtime_parameter_mutation` | mutation request | mutation proposal response | route aggregate | 不得改变 capability / safe window / audit |
| `list_runtime_parameter_mutations` | query | mutation list | route aggregate | 不得改变排序或 filtering |
| `get_runtime_parameter_mutation_detail` | proposal id | mutation detail | route aggregate | 不得绕过 scoped lookup |
| `activate_runtime_parameter_mutation` | proposal id | activation response | route aggregate | 不得改变 ledger-backed activation |
| `rollback_runtime_parameter_mutation` | proposal id | rollback response | route aggregate | 不得改变 rollback target |
| `create_runtime_ai_proposal` | AI proposal request | AI proposal response | route aggregate | 不得绕过 static check 或 capability gate |
| `list_runtime_ai_proposals` | query | AI proposal list | route aggregate | 不得改变 audit projection |
| `get_runtime_ai_proposal_detail` | proposal id | AI proposal detail | route aggregate | 不得改变 candidate diagnostics |
| `list_runtime_approvals` | query | approval list | route aggregate | 不得改变 approval visibility |
| `get_runtime_approval_detail` | approval id | approval detail | route aggregate | 不得改变 approval state |
| `approve_ai_proposal` | proposal id | approval action response | route aggregate | 不得改变 approval lock order |
| `reject_ai_proposal` | proposal id | rejection response | route aggregate | 不得丢失 rejection reason |
| `claim_ai_proposal_review` | proposal id | claim response | route aggregate | 不得改变 reviewer claim semantics |

---

## 父子通信规则

```text
backend.runtime
  -> backend.runtime.routes
  -> planned backend.runtime.routes.mutation
  -> crate::runtime::{mutation, ai_proposal, approval handlers}
  -> AppState / mutation ledger / approval_records / ai_proposals
```

`backend.runtime.routes.mutation` 只能经父级 `backend.runtime.routes` 暴露 mutation / AI proposal / approval routes。handler owner 仍是 `src/runtime/mutation.rs`；状态 owner 仍是 `AppState`。不得让 mutation route 子叶横向接管 report、evidence、experiment、ops、strategy_config、frontend caller 或 executor。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 状态与锁

| 状态/锁 | 当前 owner | 基线约束 |
| --- | --- | --- |
| mutation ledger | `AppState` / runtime mutation owner | 不迁移、不改写入顺序 |
| approval records | `AppState` / runtime mutation owner | 保持 approval transition 和 audit |
| AI proposals | `AppState` / runtime mutation owner | 保持 static check、candidate audit 和 detail lookup |
| lock order | `approval_records -> ai_proposals` | 不得反转，避免并发死锁 |
| capability context | runtime capability owner | 不得绕过缺失 capability 的阻断 |

---

## 本批次不做

- 不移动 `src/backend/runtime/routes.rs` 中任何 route。
- 不新建 `src/backend/runtime/routes/mutation.rs`。
- 不迁移 `src/runtime/mutation.rs` 中任何 handler/helper。
- 不修改 `AppState`、锁顺序、mutation ledger、approval records 或 AI proposal storage。
- 不修改 frontend caller、response schema、fixture、测试资产或发布过渡协议。
- 不把 evidence、report、experiment、ops 路线混入本子叶。

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 本批没有制造格式漂移 |
| `cargo check -p quantpilot` | Rust 模块与 Axum route 类型 | route target 类型不漂移 |
| `cargo test --no-run` | 测试编译 | mutation / AI proposal handler 仍可编译 |
| `cargo test -p quantpilot --test api_mutation` | runtime parameter mutation | proposal、activation、rollback、contract snapshot 不漂移 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal | static check、capability gate、audit 不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effects | evidence health 与 cleanup 不被误伤 |
| `cargo test -p quantpilot --test api_run` | runtime report/run 邻接路线 | route aggregate 邻接行为不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增基线保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 基线、模块树、全量树引用不缺失 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新基线和真实文件可定位 |
| `git diff --check` | diff whitespace | 本批没有空白错误 |

---

## 下一步

1. BE-001AE-02 只能建立 `backend.runtime.routes.mutation` 抽离方案。
2. 抽离方案必须继续保持 `no code movement`，只允许规划 route facade 迁移，不迁移 `src/runtime/mutation.rs` handler。
3. 若后续进入实际抽离，必须保留父级 `backend.runtime.routes` 委托和全部 route path/method 等价。

---

## 幻觉检查点

AI 声称 BE-001AE-01 完成时，必须说明: 本批只建立 `backend.runtime.routes.mutation` 单子叶等价基线，且为 `no code movement`；mutation route 尚未抽离，`src/runtime/mutation.rs` 尚未拆分，`AppState`、approval lock order、response schema、frontend caller 和发布过渡均未改变。不得宣称 `backend.runtime.routes` 父叶完成、mutation handler 已迁移、approval/AI proposal 状态 owner 已迁移、整理或重构已经完成。

---

## 验收标准

1. `128-backend.runtime.routes.mutation单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `backend.runtime.routes.mutation` 白箱节点，包含 route、handler、锁顺序和排除边界。
3. 治理门禁能发现本文档、`no code movement`、下一批 BE-001AE-02、关键 route/handler、锁顺序和测试证据缺失。
