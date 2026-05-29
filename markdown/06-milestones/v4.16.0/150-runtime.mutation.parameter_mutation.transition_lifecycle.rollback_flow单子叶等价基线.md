# v4.16.0 runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001AL-01  
> 基准: `149-runtime.mutation.parameter_mutation.transition_lifecycle第二轮父叶残余判断.md`、`src/runtime/mutation/parameter_mutation/transition_lifecycle.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`、`tests/api_mutation.rs`  
> 判定: 建立 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 单子叶等价基线。当前只冻结 rollback public handler 的输入输出、ledger lookup、safe-window、event append、metrics、transition persistence 和状态机分支；本批 `no code movement`。下一步只能进入 BE-001AL-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AL-01 rollback_flow 单子叶等价基线 | 扩展 |
| 规范矩阵 | 父子通信、rollback 状态机、ledger lookup、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 新增白箱节点 |
| 模块树 | `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` | 建立单子叶基线 |

---

## 白箱边界

`runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 是 `transition_lifecycle` 下的 rollback transaction 编排子叶，只冻结一个 public handler:

- `rollback_runtime_parameter_mutation`

本节点拥有 rollback request capability guard、activated-only gate、rollback attempt metric、source run load、target parameter version fallback、ledger lookup、rollback no-op protection、boundary resolution、rollback record id、governance projection、safe-window denial、RollbackScheduled / RolledBack / RollbackFailed lifecycle、run event append、rollback metrics 和 transition persistence 的等价证据。

本节点不拥有 activation flow、boundary_safety helper 的内部策略、activation snapshot helper body、proposal create/list/detail、AI proposal、approval review、AppState owner、schema、frontend caller、route facade、runtime persistence owner 或发布过渡连接。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` |
| 父模块 | `runtime.mutation.parameter_mutation.transition_lifecycle` |
| 上层模块 | `runtime.mutation.parameter_mutation` |
| 路由入口 | `backend.runtime.routes.mutation` |
| 当前 owner 文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle.rs` |
| 已完成 sibling | `runtime.mutation.parameter_mutation.transition_lifecycle.boundary_safety`、`runtime.mutation.parameter_mutation.transition_lifecycle.activation_flow` |
| 已完成 sibling 文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/boundary_safety.rs`、`src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs` |
| 目标文件路径 | BE-001AL-02 决定，候选为 `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` |
| route facade | `src/backend/runtime/routes/mutation.rs` |
| runtime facade | `src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` |
| 测试证据 | `tests/api_mutation.rs`、`tests/api_evidence_contract.rs`、`tests/api_run.rs` |
| 下一批次 | BE-001AL-02 抽离方案 |

---

## 输入输出冻结

| public 方法 | 输入 | 输出 | 必须保持 |
| --- | --- | --- | --- |
| `rollback_runtime_parameter_mutation` | `UserId`、`State<AppState>`、`Path(proposal_id)`、`Json<RollbackRuntimeParameterMutationRequest>` | `Json<RuntimeParameterMutationRecord>` 或 `(StatusCode, String)` | route facade 调用面、response schema、status code、error code、event sequence 和 active parameter version 写入语义不变 |

| 依赖 | 当前来源 | rollback_flow 使用方式 |
| --- | --- | --- |
| `validate_runtime_capability_guard` | capability helper | capability context 缺失或不匹配时返回 `parameter_mutation_boundary_violation` |
| `load_runtime_parameter_mutation_record` | persistence helper | 读取原 proposal record，并要求状态为 `Activated` |
| `load_run_record_from_state` | runtime state helper | 读取 source run，用于 current sequence、governance 和 active parameter version |
| `list_runtime_parameter_mutation_records` | persistence helper | 扫描 ledger，按 source / target 找 rollback target value |
| `resolve_runtime_parameter_mutation_boundary` | `boundary_safety` | 解析 rollback boundary |
| `evaluate_runtime_parameter_mutation_safe_window` | `boundary_safety` | 计算 safe-window state |
| `runtime_parameter_mutation_rollback_record_id` | transition lifecycle helper | 生成 deterministic rollback proposal id |
| `runtime_parameter_mutation_governance` | shared mutation helper | 从 source governance 派生 rollback governance |
| `build_runtime_parameter_mutation_event` | shared mutation helper | 构造 SafeWindowDenied / RollbackScheduled / RolledBack / RollbackFailed event |
| `append_parameter_mutation_events_to_run` | shared event append helper | 追加 run event，并在 rolled_back 分支写 active parameter version |
| `persist_runtime_parameter_mutation_transition` | transition shared helper | 持久化 rollback record 并刷新 state cache |

---

## 状态机冻结

| 分支 | 触发条件 | 现有结果 | 必须保持 |
| --- | --- | --- | --- |
| capability denied | `capability_context` 无效 | `parameter_mutation_boundary_violation` | 不读取 / 不写入 mutation record |
| original not activated | 原 proposal status 不是 `Activated` | `bad_request` | 不产生 rollback record，不追加 event |
| rollback attempt | 原 proposal 可进入 rollback | `record_mutation_rollback_attempt` | unknown version 也会计入 attempt |
| target fallback | request 未传 `target_parameter_version` | 回退到 `original.old_parameter_version` | 不改变默认 rollback target |
| ledger target via old value | ledger item `old_parameter_version == target` | rollback new_value = `old_value` | 只匹配同 source / same target |
| ledger target via proposed value | ledger item `proposed_parameter_version == target` | rollback new_value = `new_value` | 只匹配同 source / same target |
| unknown target | ledger 无匹配版本 | `parameter_mutation_rollback_unknown_version` | 不生成 rollback record |
| no-op target | target 等于 current active parameter version | `parameter_mutation_rollback_noop` | 不追加 event |
| safe-window denied | safe-window state `allowed == false` | status=`SafeWindowDenied`，追加 denied lifecycle/event，记录 denied metric，持久化 record，返回 `parameter_mutation_safe_window_denied` | 不写 active parameter version |
| scheduled | safe-window allowed | status=`RollbackScheduled`，写 activation_state，追加 schedule lifecycle/event，调用 `record_mutation_rollback_scheduled` | schedule sequence = current sequence + 1 |
| immediate rollback by next cycle | resolved boundary requested 为 `next_cycle_start` | status=`RolledBack`，activated_at = now + 1，追加 rollback lifecycle/event，active parameter version = rollback target，调用 `record_mutation_rollback_applied` | rollback sequence = schedule sequence + 1 |
| failed boundary | resolved sequence no <= schedule sequence no | status=`RollbackFailed`，写 failure_reason，追加 failed lifecycle/event，调用 `record_mutation_rollback_failed` | 不写 active parameter version |
| final persistence | 所有 allowed 分支 | append events -> persist transition -> return record | 顺序不变 |

---

## 回归证据冻结

| 测试 | 覆盖范围 |
| --- | --- |
| `runtime_parameter_mutation_rolls_back_to_ledger_backed_prior_version` | activated-only rollback、unknown target、ledger-backed target、rolled_back event、active parameter version、rollback metrics |
| `runtime_parameter_mutation_contract_snapshot_matches_fixture` | rollback record fields、status contract、rollback response fixture |
| `runtime_parameter_mutation_activation_uses_explicit_boundary_versions_and_reports` | rollback 依赖的 activated proposal 和 parameter version 基线 |
| `api_evidence_contract` / `api_run` | run event append 和 evidence projection 邻接面 |

---

## 父子通信规则

```text
backend.runtime.routes.mutation
  -> crate::runtime::rollback_runtime_parameter_mutation
  -> runtime.mutation.parameter_mutation
  -> runtime.mutation.parameter_mutation.transition_lifecycle
  -> rollback_flow candidate
  -> boundary_safety helper only through transition_lifecycle-owned dependency
```

BE-001AL-01 只建立基线，不能创建 `rollback_flow.rs` 目标文件。BE-001AL-02 才能决定目标路径、path attribute、visibility、父级 wrapper / re-export 和回退点。

后续若创建 `rollback_flow`，它只能向父级 `transition_lifecycle` 暴露 rollback handler 或受控 helper；route facade、AI proposal、approval review、frontend caller 或发布过渡连接不得直接依赖该子叶。ASCII guard: `release transition guard`。

---

## 排除边界

- 不移动 Rust 代码。
- 不创建 `rollback_flow.rs` 目标文件。
- 不迁移 `activate_runtime_parameter_mutation`。
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

下一批进入 BE-001AL-02 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 抽离方案。该方案必须先决定:

| 决策点 | 候选 | 暂定约束 |
| --- | --- | --- |
| 目标文件 | `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs` | 不在 BE-001AL-01 创建 |
| 父级声明 | path-attributed child | 必须兼容当前 nested path 解析 |
| 父级出口 | `pub(crate) use rollback_flow::rollback_runtime_parameter_mutation` | route facade 调用面不变 |
| sibling dependency | `boundary_safety` helper | 只能经父级受控引用，不新增横向发布连接 |
| shared helper | rollback id、lifecycle entry、transition persistence | 是否迁移 helper body 留给 BE-001AL-02 / closeout 判断 |

---

## 幻觉检查点

AI 声称 BE-001AL-01 完成时，必须说明当前只是 `rollback_flow` 单子叶等价基线，代码未移动，目标文件未创建；activation_flow、boundary_safety、snapshot side-effect、AI proposal/approval、AppState/schema/frontend caller 和发布过渡均未改变。不得宣称 rollback handler 已抽离或 transition_lifecycle 父叶完成。

---

## 验收标准

1. `150-runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.mutation.parameter_mutation.transition_lifecycle.rollback_flow` 白箱节点，但不引用尚未创建的目标 Rust 文件。
3. 全量树记录 BE-001AL-01 并把下一步固定为 BE-001AL-02 抽离方案。
4. 本批无代码移动。
5. 本批验证通过后，后续才能进入 BE-001AL-02。
