# v4.16.0 backend.runtime 第三轮父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CK-01  
> 基准: `277-runtime.evidence_health单叶closeout.md`、`273-backend.runtime第二轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime stop_split: false`  
> 下一步: BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CK-01 `backend.runtime` 第三轮父叶残余判断 | 父叶判断 |
| 规范矩阵 | 父叶停止条件、下一候选选择、禁止跳步 | 冻结 |
| 引导矩阵 | `root.backend.runtime` | 父叶继续细拆 |
| 模块树 | `runtime.mutation.shared_governance` | 下一候选 |

---

## 当前真实结构

已经 closeout / 收口的直接子域:

- `backend.runtime.routes stop_split: true`
- `runtime.report_ops stop_split: true`
- `runtime.evidence_health stop_split: true`
- `runtime.backtest stop_split: true`
- `runtime.mutation.parameter_mutation stop_split: true`
- `runtime.mutation.ai_proposal stop_split: true`

父级 `src/runtime/mod.rs` 和 parent include 文件仍直接持有跨子叶支撑残余:

- `src/runtime/mod.rs`: `DiscardRuntimeArtifactResponse`、`RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`clean_optional_filter`、`normalized_replay_options`、`RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`DEFAULT_REPLAY_PAGE_SIZE`、`MAX_REPLAY_PAGE_SIZE`。
- `src/runtime/run.rs`: `RuntimeApprovalListQuery`、`MergeRecordsResponse`、`MergeRecordEntry`。
- `src/runtime/mutation.rs`: `canonical_runtime_parameter_version`、`validate_runtime_parameter_mutation_target`、`runtime_mode_from_events`、`status_contract_value`、`mutation_event_contract`、`build_runtime_parameter_mutation_event`、`append_parameter_mutation_events_to_run`、`runtime_parameter_mutation_governance`、`governance_with_parameter_version`、`OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`。
- `src/runtime/backtest.rs`: drained parent include only, retained until parent support residual can be removed safely.

这些残余不再是 handler owner，但它们仍让 `backend.runtime` 父叶不能设置 `stop_split: true`。

---

## 残余判断

`backend.runtime` 当前仍不满足停止细分条件:

1. `src/runtime/mutation.rs` 仍有一组跨 `runtime.mutation.parameter_mutation` 与 `runtime.mutation.ai_proposal` 共享的治理 helper。
2. 这组 helper 同时服务 proposal creation、activation / rollback lifecycle 和 AI proposal creation，具备稳定白箱边界。
3. 这组 helper 的状态含义比泛化的 parent support 更清晰，优先形成 `runtime.mutation.shared_governance` 能降低父级耦合。
4. `RuntimeReplayQuery`、`RunInProgressGuard`、`DiscardRuntimeArtifactResponse`、report query / response DTO 和 experiment limit 仍是父级支撑残余，但它们应排在 mutation shared governance 之后另起父叶判断，不得混进本批。

因此:

```text
backend.runtime stop_split: false
next: BE-001CL-01 runtime.mutation.shared_governance 单子叶等价基线
```

---

## 下一候选白箱

候选节点:

```text
root.backend.runtime.runtime.mutation.shared_governance
```

候选 planned child 文件只允许在后续抽离方案明确后创建:

```text
src/runtime/mutation/shared_governance.rs
```

BE-001CL-01 只能先冻结等价基线，不得直接创建 child 文件，不得迁移 helper。

候选输入:

- `RuntimeParameterMutationTarget`
- `RuntimeParameterMutationRecord`
- `RuntimeParameterMutationStatus`
- `RuntimeGovernanceSnapshot`
- `FrontendRuntimeEvent`
- `AppState`
- `auth::UserId`
- source run id / source governance / active parameter version

候选输出:

- canonical parameter version
- target validation result
- mutation event
- governed run event append
- runtime parameter mutation governance
- governance snapshot with parameter version

候选调用方:

- `src/runtime/mutation/parameter_mutation/proposal_creation.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/activation_flow.rs`
- `src/runtime/mutation/parameter_mutation/transition_lifecycle/rollback_flow.rs`
- `src/runtime/mutation/ai_proposal/proposal_creation.rs`

---

## 明确排除

- 不在本批创建 `src/runtime/mutation/shared_governance.rs`。
- 不在本批迁移 `canonical_runtime_parameter_version`、`validate_runtime_parameter_mutation_target`、`runtime_mode_from_events`、`status_contract_value`、`mutation_event_contract`、`build_runtime_parameter_mutation_event`、`append_parameter_mutation_events_to_run`、`runtime_parameter_mutation_governance` 或 `governance_with_parameter_version`。
- 不在本批处理 `RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`、`OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`MergeRecordsResponse`、`MergeRecordEntry`、`DiscardRuntimeArtifactResponse`、`RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`DEFAULT_REPLAY_PAGE_SIZE` 或 `MAX_REPLAY_PAGE_SIZE`。
- 不回改 `runtime.report_ops`、`runtime.evidence_health`、`runtime.backtest`、`runtime.mutation.parameter_mutation` 或 `runtime.mutation.ai_proposal` closed child。
- 不修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、锁顺序或 release transition guard。

---

## 验证要求

本批为 `no code movement` 父叶判断，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CL-01 runtime.mutation.shared_governance 单子叶等价基线
```

BE-001CL-01 只能冻结 mutation shared governance helper 的输入、输出、调用方、父级 re-export 预期、回归测试和排除项。不得直接创建 child 文件、迁移 helper、移动 query DTO、处理 run guard、删除 drained parent include 或启动 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CK-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `backend.runtime stop_split: false`。
3. `backend.runtime.routes`、`runtime.report_ops`、`runtime.evidence_health`、`runtime.backtest`、`runtime.mutation.parameter_mutation` 与 `runtime.mutation.ai_proposal` 均已 closeout，但父级仍有共享支撑残余。
4. 下一步只能进入 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线。
5. 本批没有创建 `src/runtime/mutation/shared_governance.rs`，没有迁移 helper，没有处理 query DTO、run guard、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

不得宣称 `backend.runtime` 已完成、Rust 重构完成、parent support 已整体抽离、发布过渡已启动或可以跳过等价基线直接迁移 helper。

---

## 验收标准

1. `278-backend.runtime第三轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树保持 `backend.runtime stop_split: false`。
3. 下一候选固定为 BE-001CL-01 `runtime.mutation.shared_governance` 单子叶等价基线。
4. 治理门禁、Rust 相关等价测试、全量树覆盖和 `git diff --check` 均通过。
