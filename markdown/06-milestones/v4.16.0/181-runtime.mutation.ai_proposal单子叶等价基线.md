# v4.16.0 runtime.mutation.ai_proposal 单子叶等价基线
> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AY-01  
> 基线: `180-runtime.mutation.parameter_mutation第三轮父叶残余判断.md`、`src/runtime/mutation.rs`、`src/backend/runtime/routes/mutation.rs`、`tests/api_ai_proposal.rs`  
> 判定: 建立 `runtime.mutation.ai_proposal` 单子叶等价基线。当前只冻结 AI proposal candidate、static check、proposal list/detail、approval list/detail、approve/reject/claim、sandbox gate、approval persistence、状态迁移和测试证据；本批 `no code movement`。下一步只能进入 BE-001AY-02 抽离方案。  
> 代码动作: `no code movement`

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AY-01 AI proposal handler 单子叶等价基线 | 扩展 |
| 规范矩阵 | AI proposal 静态检查、审批锁顺序、父子通信、状态 owner 保留 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.ai_proposal` | 新增白箱节点 |
| 模块树 | `runtime.mutation.ai_proposal` | 建立单子叶基线 |

---

## 选择理由

`runtime.mutation.ai_proposal` 是 `runtime.mutation.parameter_mutation` 关闭后的下一个稳定 sibling:

1. 它覆盖 `/api/runtime/ai-proposals` 与 `/api/v1/ai/approvals` 的候选生成、审计查询和审批动作。
2. 它拥有独立的 static check、config domain binding、sandbox report gate、proposal status transition 和 approval lifecycle 语义。
3. `tests/api_ai_proposal.rs` 已覆盖创建成功、缺 capability 拒绝、静态检查失败、缺字段拒绝和关键事件回归。
4. 该域当前仍和 approval review、AI proposal persistence helper 共处于 `src/runtime/mutation.rs`，适合先冻结单叶等价基线，再进入抽离方案。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.ai_proposal` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.ai_proposal` |
| 父模块 | `backend.runtime` |
| route 入口 | `backend.runtime.routes.mutation` |
| handler owner | `src/runtime/mutation.rs` |
| handler facade | `src/runtime/mod.rs` |
| route facade | `src/backend/runtime/routes/mutation.rs` |
| state owner | `AppState` |
| schema owner | `src/frontend_api_types.rs` |
| persistence owner | `src/runtime_persistence.rs` + local approval disk helper |
| 测试证据 | `tests/api_ai_proposal.rs` |
| 下一批次 | BE-001AY-02 抽离方案 |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `CreateRuntimeAiProposalRequest` | frontend、API caller、`tests/api_ai_proposal.rs` | 必须保留 source_kind/source_id、target、old/new value、model、prompt_hash、evidence_hash、reason、config_domain_binding |
| 输入 | `RuntimeAiProposalListQuery` | API caller | 必须保留 source_kind/source_id/status filtering 与 `created_at_ms`、`ai_proposal_id` 倒序 |
| 输入 | `ApprovalActionRequest` | approve/reject/claim routes | 必须保留 actor_id、comment、proposal id 和 approval state guard |
| 输入 | source evidence | run/backtest record owner | 必须保留 run/backtest source context、event_count、current_sequence_no、governance copy |
| 输入 | sandbox report | sandbox verification owner | approve 前必须保留 report existence 与 passed gate |
| 输出 | `RuntimeAiProposalRecord` | frontend、tests、runtime evidence | 不得改变 status、static_check、source_evidence、governance、config_domain_binding、lifecycle 或 denial_reason |
| 输出 | `RuntimeApprovalRecord` | frontend、tests、approval API | 不得改变 review_state、reviewers、sandbox_report_url、lifecycle 或 persistence file name |
| 输出 | `FrontendRuntimeEvent` | run/backtest evidence | 不得改变 AIProposalCreated、AIProposalStaticCheckPassed、AIProposalStaticCheckFailed、AIProposalDenied、AIProposalApproved event contract |

---

## route owner 基线

| route | method | route owner | handler |
| --- | --- | --- | --- |
| `/api/runtime/ai-proposals` | GET | `src/backend/runtime/routes/mutation.rs` | `list_runtime_ai_proposals` |
| `/api/runtime/ai-proposals` | POST | `src/backend/runtime/routes/mutation.rs` | `create_runtime_ai_proposal` |
| `/api/runtime/ai-proposals/:ai_proposal_id` | GET | `src/backend/runtime/routes/mutation.rs` | `get_runtime_ai_proposal_detail` |
| `/api/v1/ai/approvals` | GET | `src/backend/runtime/routes/mutation.rs` | `list_runtime_approvals` |
| `/api/v1/ai/approvals/:approval_id` | GET | `src/backend/runtime/routes/mutation.rs` | `get_runtime_approval_detail` |
| `/api/v1/ai/proposals/:proposal_id/approve` | POST | `src/backend/runtime/routes/mutation.rs` | `approve_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/reject` | POST | `src/backend/runtime/routes/mutation.rs` | `reject_ai_proposal` |
| `/api/v1/ai/proposals/:proposal_id/claim` | POST | `src/backend/runtime/routes/mutation.rs` | `claim_ai_proposal_review` |

---

## 关键 public 方法

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_ai_proposal` | user id、`AppState`、AI proposal request | AI proposal record | `backend.runtime.routes.mutation` | 不得绕过 model identity、source evidence、capability、static check、approval record 或 sandbox trigger |
| `list_runtime_ai_proposals` | list query | AI proposal records | `backend.runtime.routes.mutation` | 不得改变 filtering 或 sorting |
| `get_runtime_ai_proposal_detail` | user id、proposal id | AI proposal record | `backend.runtime.routes.mutation` | 不得绕过 scoped memory lookup 或 disk fallback |
| `list_runtime_approvals` | user id、approval query | approval records | `backend.runtime.routes.mutation` | 不得改变 scoped visibility 或 review_state filtering |
| `get_runtime_approval_detail` | user id、approval id | approval record | `backend.runtime.routes.mutation` | 不得绕过 memory-first lookup 或 disk fallback |
| `approve_ai_proposal` | proposal id、approval action request | approval record | `backend.runtime.routes.mutation` | 不得改变 sandbox gate、reviewer quorum、approval lifecycle 或 proposal approved transition |
| `reject_ai_proposal` | proposal id、approval action request | approval record | `backend.runtime.routes.mutation` | 不得丢失 rejection reason 或 proposal denied transition |
| `claim_ai_proposal_review` | proposal id、approval action request | approval record | `backend.runtime.routes.mutation` | 不得改变 pending-only claim guard 或 reviewer assignment |

---

## helper 边界基线

| 子域 | 当前函数 | 基线约束 |
| --- | --- | --- |
| model/static check | `validate_ai_model_identity`、`ai_proposal_static_check_result`、`is_v4_ai_proposal_target`、`expected_config_domain_for_target`、`validate_ai_proposal_config_domain_binding`、`analyze_v4_backtest_artifact_for_ai` | 不得改变 provider/model/version 必填、source evidence、noop、reason、v4 backtest source、config domain binding 和 digest 校验 |
| source/governance/id | `load_runtime_ai_proposal_source_context`、`runtime_ai_proposal_governance`、`runtime_ai_proposal_record_id` | 不得改变 run/backtest source lookup、event count、sequence cursor、governance copy 或 deterministic id digest |
| event/lifecycle | `ai_proposal_event_contract`、`build_runtime_ai_proposal_event`、`ai_proposal_lifecycle_entry`、`persist_runtime_ai_proposal_transition` | 不得改变 event_type、reason_code、payload、lifecycle sequence 或 memory/disk write order |
| proposal lookup | `load_runtime_ai_proposal_for_user` | 不得改变 scoped cache 优先和 persistence fallback |
| approval guard | `load_sandbox_report_for_proposal`、`ensure_ai_proposal_can_be_approved` | 不得改变 binding required、static_check_passed required、sandbox report required 和 sandbox passed required |
| approval transition | `ai_proposal_approved_status`、`is_valid_ai_proposal_transition`、`update_ai_proposal_status` | 不得改变 Submitted -> StaticCheckPassed/Failed、StaticCheckPassed -> Approved/Denied/Expired transition |
| approval persistence | `persist_approval`、`load_approval_from_disk` | 不得改变 approval file naming、atomic write 或 not_found/internal error mapping |

---

## 状态与锁

- `state.ai_proposals` 仍由 `AppState` 拥有，本批不迁移 state owner。
- `state.approval_records` 仍由 `AppState` 拥有，本批不迁移 approval state owner。
- approval action 路径必须保持既有锁顺序和写入语义；尤其是 approve/reject 对 approval record 与 AI proposal status 的更新顺序不得改变。
- `approval_records -> ai_proposals` lock order 是本基线的并发保护点，不得在无方案和等价门禁时重排。
- sandbox verification background task、JoinHandle 监控和 `sandbox_report_url` 回写语义不得改变。

---

## 父子通信规则

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.mutation
  -> runtime.mutation.ai_proposal
  -> AppState / runtime persistence / sandbox verification / run-backtest evidence
```

`runtime.mutation.ai_proposal` 只能经 `backend.runtime.routes.mutation` 暴露 HTTP route。它不得横向接管 parameter mutation、report、evidence、experiment、ops、strategy_config、frontend caller 或 executor。状态 owner 仍是 `AppState`，schema owner 仍是 `src/frontend_api_types.rs`，共享 persistence owner 仍是 `src/runtime_persistence.rs`。发布过渡前不得主动提出横向连接或性能旁路。ASCII guard: `release transition guard`。

---

## 本批次不做

- 不移动 `src/runtime/mutation.rs` 中任何 handler/helper。
- 不新建 `src/runtime/mutation/ai_proposal.rs` 或其他代码文件。
- 不修改 `src/runtime/mod.rs` include/re-export facade。
- 不修改 `src/backend/runtime/routes/mutation.rs` route facade。
- 不迁移 `AppState`、`ai_proposals`、`approval_records`、sandbox report、runtime persistence、schema、frontend caller 或 fixture。
- 不启动 release transition，不提出横向连接或性能旁路。
- 不把 parameter mutation、report/evidence/experiment/ops route 或 frontend runtime panel 混入本子叶。

---

## 未来抽离决策点

| 决策点 | 当前默认 | 原因 |
| --- | --- | --- |
| 目标文件路径 | BE-001AY-02 决定 | 需先确认是否一次性迁入 `src/runtime/mutation/ai_proposal.rs` 并保留父级 re-export |
| approval review 是否随 AI proposal 首轮抽离 | 倾向随本叶首轮迁移 | approve/reject/claim 与 proposal status transition、approval persistence 强耦合 |
| static check 是否立即再细拆 | BE-001AY closeout 后判断 | static check/helper 体量较大，但应先完成大叶抽离再按递归流程判断 |
| approval persistence helper 是否再细拆 | BE-001AY closeout 后判断 | 当前只服务 approval review，但涉及 disk owner 和 error mapping |
| sandbox trigger 是否独立成叶 | BE-001AY closeout 后判断 | 当前与 create path 强耦合，先保持等价优先 |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo fmt --check` | Rust 格式 | 本批没有制造格式漂移 |
| `cargo check -p quantpilot` | Rust 模块与 route handler 类型 | handler owner 与 route facade 类型不漂移 |
| `cargo test --no-run` | 测试编译 | mutation / proposal / approval 邻接 handler 仍可编译 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal API | create、static check、capability denial、audit、missing contract fields 不漂移 |
| `cargo test -p quantpilot --test api_mutation` | parameter mutation 邻接域 | 本基线不误伤已 closeout parameter mutation |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effects | proposal evidence/report 健康指标不漂移 |
| `cargo test -p quantpilot --test api_run` | run/backtest source 邻接域 | source context lookup 不漂移 |
| `tools\check-utf8.ps1` | 文档编码 | 新增基线保持 UTF-8 |
| `tools\check-matrix-governance.ps1` | 治理门禁 | 基线、模块树、全量树引用不缺失 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 | 新基线和真实文件可定位 |
| `git diff --check` | diff whitespace | 本批没有空白错误 |

---

## 下一步

下一批进入 BE-001AY-02 `runtime.mutation.ai_proposal` 抽离方案。该批仍应保持 `no code movement`，只允许规划目标文件路径、父级 child 声明、handler re-export、允许迁移函数清单、回退点和验证门禁。

不得直接移动 approval review、static check helper、sandbox trigger、AppState、schema、frontend caller、runtime persistence owner 或 release transition guard；如果 BE-001AY-02 决定把 approval review 随 AI proposal 一起首轮迁移，必须显式写入迁移清单和回退点。

---

## 幻觉检查点

AI 声称 BE-001AY-01 完成时，必须说明本批只建立 `runtime.mutation.ai_proposal` 单子叶等价基线，并且为 `no code movement`。不得宣称 AI proposal handler 已迁移、目标文件已创建、approval review 已拆分、AppState/schema/frontend caller 已改变、sandbox owner 已迁移、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `181-runtime.mutation.ai_proposal单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.ai_proposal` 白箱节点，包含 route、handler、helper、状态 owner、锁顺序、schema owner、persistence owner 和排除边界。
3. 治理门禁能发现本文档、`no code movement`、下一批 BE-001AY-02、关键 handler/helper、状态锁顺序和测试证据缺失。
4. 本批验证通过后，后续才能进入 BE-001AY-02 抽离方案。
