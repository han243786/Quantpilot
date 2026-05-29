# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AJ-01  
> 基准: `144-runtime.mutation.parameter_mutation.transition_lifecycle父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`、`src/runtime/mutation/parameter_mutation.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 单子叶等价基线。当前只冻结 activation handler 的输入输出、状态机分支、event append、metrics、transition persistence 和 snapshot trigger；本批 `no code movement`。下一步只能进入 BE-001AJ-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AJ-01 activation_flow 单子叶等价基线 | 扩展 |
| 规范矩阵 | 父子通信、activation 状态机、event append、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 新增白箱节点 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` | 建立单子叶基线 |

---

## 白箱边界

`runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 是 `transition_lifecycle` 下的 activation 事务编排子叶，只冻结一个 public handler:

- `activate_runtime_parameter_mutation`

本节点拥有 activation 请求校验、proposal record load、source run load、actor resolution、safe-window application、activation scheduling、activated / failed lifecycle、run event append、activation metrics、transition persistence 和 `auto_snapshot_on_activation` 调用时机。

本节点不拥有 boundary_safety helper 的内部策略，不拥有 rollback flow，不拥有 snapshot helper body，不拥有 proposal create/list/detail、AI proposal、approval review、schema、frontend caller、AppState owner 或发布过渡连接。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` |
| 父模块 | `runtime.mutation.parameter_mutation.transition_lifecycle` |
| 上层模块 | `runtime.mutation.parameter_mutation` |
| 路由入口 | `backend.runtime.routes.mutation` |
| 当前 owner 文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 已完成 sibling | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety` |
| 已完成 sibling 文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs` |
| 目标文件路径 | BE-001AJ-02 决定，候选为 src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs |
| route facade | `src/backend/runtime/routes/mutation.rs` |
| runtime facade | `src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` |
| 测试证据 | `tests/api_mutation.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs` |
| 下一批次 | BE-001AJ-02 抽离方案 |

---

## 输入输出冻结

| public 方法 | 输入 | 输出 | 必须保持 |
| --- | --- | --- | --- |
| `activate_runtime_parameter_mutation` | `UserId`、`State<AppState>`、`Path(proposal_id)`、`Json<ActivateRuntimeParameterMutationRequest>` | `Json<RuntimeParameterMutationRecord>` 或 `(StatusCode, String)` | route facade 调用面、response schema、status code 和 error code 不变 |

| 依赖 | 当前来源 | activation_flow 使用方式 |
| --- | --- | --- |
| `validate_runtime_capability_guard` | capability helper | capability context 缺失或不匹配时返回 `parameter_mutation_boundary_violation` |
| `load_runtime_parameter_mutation_record` | persistence helper | 读取 proposal record，不改变 store owner |
| `load_run_record_from_state` | runtime state helper | 读取 source run，不迁移 state owner |
| `resolve_runtime_parameter_mutation_boundary` | `boundary_safety` | 解析 activation boundary |
| `evaluate_runtime_parameter_mutation_safe_window` | `boundary_safety` | 计算 safe-window state |
| `build_runtime_parameter_mutation_event` | shared mutation helper | 构造 schedule / denied / activated / failed event |
| `append_parameter_mutation_events_to_run` | shared event append helper | 追加 run event 并可写 active parameter version |
| `persist_runtime_parameter_mutation_transition` | transition shared helper | 持久化 mutation record 并刷新 state cache |
| `auto_snapshot_on_activation` | transition lifecycle helper | activation 完成后触发，helper body 不属于本基线迁移范围 |

---

## 状态机冻结

| 分支 | 触发条件 | 现有结果 | 必须保持 |
| --- | --- | --- | --- |
| capability denied | `capability_context` 无效 | `parameter_mutation_boundary_violation` | 不读取 / 不写入 mutation record |
| invalid source status | record status 不是 `Proposed` / `SafeWindowDenied` | `bad_request` | 不追加 event，不持久化 transition |
| safe-window denied | `evaluate_runtime_parameter_mutation_safe_window` 返回 `allowed == false` | status=`SafeWindowDenied`，追加 denied lifecycle/event，记录 denied metric，持久化 record，返回 `parameter_mutation_safe_window_denied` | 不设置 active parameter version，不触发 snapshot |
| scheduled | safe-window allowed | status=`ActivationScheduled`，写 activation_state，追加 schedule lifecycle/event，记录 scheduled metric | schedule sequence = current sequence + 1 |
| immediate activation by next cycle | resolved boundary requested 为 `next_cycle_start` | status=`Activated`，activated_at = now + 1，追加 activation lifecycle/event，active parameter version = proposed version，记录 activation applied metric | activation sequence = schedule sequence + 1 |
| failed boundary | resolved sequence no <= schedule sequence no | status=`ActivationFailed`，写 failure_reason，追加 failed lifecycle/event，记录 activation failed metric | 不写 active parameter version |
| final persistence | 所有 allowed 分支 | append events -> persist transition -> `auto_snapshot_on_activation` -> return record | 顺序不变 |

---

## 回归证据冻结

| 测试 | 覆盖范围 |
| --- | --- |
| `runtime_parameter_mutation_activation_uses_explicit_boundary_versions_and_reports` | activation boundary、schedule / activated lifecycle、active parameter version、report/export lifecycle count |
| `runtime_parameter_mutation_manual_pause_stays_pending` | manual pause boundary 保持 scheduled / pending，不立即 activated |
| `runtime_parameter_mutation_safe_window_denial_is_audited_without_activation` | safe-window denied 分支、audit event、denied metric、不触发 activation |
| `runtime_parameter_mutation_rolls_back_to_ledger_backed_prior_version` | activation 成功后 rollback 依赖 activated proposal 和 active parameter version |
| `runtime_parameter_mutation_contract_snapshot_matches_fixture` | activation record fields、safe-window fields、mutation event types 契约 |
| `api_evidence_contract` / `api_run` | run event append 和 evidence projection 邻接面 |

---

## 父子通信规则

```text
backend.runtime.routes.mutation
  -> crate::runtime::activate_runtime_parameter_mutation
  -> runtime.mutation.parameter_mutation
  -> runtime.mutation.parameter_mutation.transition_lifecycle
  -> activation_flow candidate
  -> boundary_safety helper only through transition_lifecycle-owned dependency
```

BE-001AJ-01 只建立基线，不能创建 activation_flow 目标文件。BE-001AJ-02 才能决定目标路径、path attribute、visibility、父级 wrapper / re-export 和回退点。

后续若创建 `activation_flow`，它只能向父级 `transition_lifecycle` 暴露 activation handler 或受控 helper；route facade、AI proposal、approval review、frontend caller 或发布过渡连接不得直接依赖该子叶。ASCII guard: `release transition guard`。

---

## 排除边界

- 不移动 Rust 代码。
- 不创建 activation_flow 目标文件。
- 不迁移 `rollback_runtime_parameter_mutation`。
- 不迁移 `auto_snapshot_on_activation` 的 helper body。
- 不迁移 boundary validation / resolution / safe-window evaluation 的已 closeout child。
- 不迁移 proposal create/list/detail、AI proposal、approval review、AppState、schema、frontend caller、route facade、测试 fixture 或发布过渡连接。
- 不主动提出发布版本过渡或横向性能连接。

---

## 回归保护

| 证据 | 覆盖范围 |
| --- | --- |
| `cargo fmt --check` | Rust 格式不漂移 |
| `cargo check -p quantpilot` | type / visibility 不漂移 |
| `cargo test --no-run` | 测试编译不漂移 |
| `cargo test -p quantpilot --test api_mutation` | activation / safe-window / rollback 主证据 |
| `cargo test -p quantpilot --test api_ai_proposal` | AI proposal 邻接 shared helper 不漂移 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence side effect 不漂移 |
| `cargo test -p quantpilot --test api_run` | run record append 不漂移 |
| `tools\check-utf8.ps1` | UTF-8 |
| `tools\check-matrix-governance.ps1` | 三矩阵登记 |
| `tools\check-full-feature-tree.ps1` | 全量树覆盖 |
| `git diff --check` | whitespace |

---

## 下一步

下一批进入 BE-001AJ-02 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 抽离方案。该方案必须先决定:

| 决策点 | 候选 | 暂定约束 |
| --- | --- | --- |
| 目标文件 | src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs | 不在 BE-001AJ-01 创建 |
| 父级声明 | path-attributed child | 必须兼容当前 nested path 解析 |
| 父级出口 | wrapper 或 re-export | route facade 调用面不变 |
| sibling dependency | boundary_safety helper | 只能经父级受控引用，不新增横向发布连接 |
| side effect helper | `auto_snapshot_on_activation` | 本轮只冻结调用时机，是否迁移 helper body 留给后续 closeout 判断 |

---

## 幻觉检查点

AI 声称 BE-001AJ-01 完成时，必须说明当前只是 `activation_flow` 单子叶等价基线，代码未移动，目标文件未创建；rollback_flow、snapshot side-effect、AI proposal/approval、AppState/schema/frontend caller 和发布过渡均未改变。不得宣称 activation handler 已抽离或 transition_lifecycle 父叶完成。

---

## 验收标准

1. `145-runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` 白箱节点，但不引用尚未创建的目标 Rust 文件。
3. 全量树记录 BE-001AJ-01 并把下一步固定为 BE-001AJ-02 抽离方案。
4. 本批无代码移动。
5. 本批验证通过后，后续才能进入 BE-001AJ-02。
