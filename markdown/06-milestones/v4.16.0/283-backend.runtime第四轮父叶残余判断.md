# v4.16.0 backend.runtime 第四轮父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CM-01  
> 基准: `282-runtime.mutation.shared_governance单叶closeout.md`、`278-backend.runtime第三轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime stop_split: false`  
> 下一步: BE-001CN-01 `runtime.query_support` 单子叶等价基线  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CM-01 `backend.runtime` 第四轮父叶残余判断 | 父叶判断 |
| 规范矩阵 | 父叶停止条件、下一候选选择、禁止跳步 | 冻结 |
| 引导矩阵 | `root.backend.runtime` | 父叶继续细拆 |
| 模块树 | `runtime.query_support` | 下一候选 |

---

## 当前真实结构

已经 closeout / 收口的直接子域:

- `backend.runtime.routes stop_split: true`
- `runtime.report_ops stop_split: true`
- `runtime.evidence_health stop_split: true`
- `runtime.backtest stop_split: true`
- `runtime.mutation.parameter_mutation stop_split: true`
- `runtime.mutation.ai_proposal stop_split: true`
- `runtime.mutation.shared_governance stop_split: true`

父级 `src/runtime/mod.rs` 和 parent include 文件仍直接持有跨子叶支撑残余:

- `src/runtime/mod.rs`: `DiscardRuntimeArtifactResponse`、`RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`clean_optional_filter`、`normalized_replay_options`、`RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`DEFAULT_REPLAY_PAGE_SIZE`、`MAX_REPLAY_PAGE_SIZE`。
- `src/runtime/run.rs`: `RuntimeApprovalListQuery`、`MergeRecordsResponse`、`MergeRecordEntry`。
- `src/runtime/mutation.rs`: `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`。
- `src/runtime/backtest.rs`: drained parent include only, retained until parent support residual can be removed safely.

这些残余不再是 handler owner，但仍让 `backend.runtime` 父叶不能设置 `stop_split: true`。

---

## 残余判断

`backend.runtime` 当前仍不满足停止细分条件:

1. `src/runtime/mod.rs`、`src/runtime/run.rs` 与 `src/runtime/mutation.rs` 仍直接持有多组 request query DTO。
2. 这些 DTO 同时服务 run replay/status、backtest replay、parameter mutation list、AI proposal list、approval list、v1 report endpoints 与 replay option normalization。
3. `clean_optional_filter` 与 `normalized_replay_options` 已形成跨 replay / mutation / proposal 查询路径共享的 filter normalization surface。
4. 相比先抽 `RunInProgressGuard` 或 response support，优先形成 `runtime.query_support` 能更快削薄 parent include 残余，并为后续移除 drained `mutation.rs` / `run.rs` 支撑文件提供明确白箱。
5. `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`RunInProgressGuard` 和 experiment limit 仍是父级支撑残余，但它们应排在 query support 之后另起父叶判断，不得混进本批。

因此:

```text
backend.runtime stop_split: false
next: BE-001CN-01 runtime.query_support 单子叶等价基线
```

---

## 下一候选白箱

候选节点:

```text
root.backend.runtime.runtime.query_support
```

候选 planned child 文件只允许在后续抽离方案明确后创建:

```text
src/runtime/query_support.rs
```

BE-001CN-01 只能先冻结等价基线，不得直接创建 child 文件，不得迁移 DTO 或 helper。

候选输入:

- query params from `/api/runtime/runs/:id/replay`
- query params from `/api/runtime/backtests/:id/replay`
- query params from runtime parameter mutation list
- query params from runtime AI proposal list
- query params from runtime approval list
- query params from v1 ops / audit / research reports

候选输出:

- `RuntimeReplayQuery`
- `RuntimeReplayOptions`
- `RuntimeParameterMutationListQuery`
- `RuntimeAiProposalListQuery`
- `RuntimeApprovalListQuery`
- `OpsDailyQuery`
- `AuditWeeklyQuery`
- `ResearchMonthlyQuery`
- cleaned optional filters
- normalized replay options

候选调用方:

- `src/runtime/run/replay_status.rs`
- `src/runtime/backtest/replay.rs`
- `src/runtime/mutation/parameter_mutation/record_query.rs`
- `src/runtime/mutation/ai_proposal/record_query.rs`
- `src/runtime/mutation/ai_proposal/approval_review.rs`
- `src/runtime/report_ops/v1_report_endpoints.rs`

候选落地时需特别冻结 visibility: 迁入 child 后，query DTO fields 必须保持调用方可访问，优先评估 `pub(super)` field visibility，不得扩大到 public API。

---

## 明确排除

- 不在本批创建 `src/runtime/query_support.rs`。
- 不在本批迁移 `RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`、`OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`clean_optional_filter` 或 `normalized_replay_options`。
- 不在本批处理 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`DEFAULT_REPLAY_PAGE_SIZE` 或 `MAX_REPLAY_PAGE_SIZE`。
- 不回改 `backend.runtime.routes`、`runtime.report_ops`、`runtime.evidence_health`、`runtime.backtest`、`runtime.mutation.parameter_mutation`、`runtime.mutation.ai_proposal` 或 `runtime.mutation.shared_governance` closed child。
- 不修改 route facade、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、锁顺序或 release transition guard。

---

## 验证要求

本批为 `no code movement` 父叶判断，提交前仍需执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
cargo test -p quantpilot --test api_ai_proposal
cargo test -p quantpilot --test api_v1_reports
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CN-01 runtime.query_support 单子叶等价基线
```

BE-001CN-01 只能冻结 query DTO、filter normalization、replay option normalization、调用方和 field visibility 边界。不得直接创建 `src/runtime/query_support.rs`，不得迁移 query DTO/helper，不能处理 response support、run guard、experiment limit、drained parent include 删除或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CM-01 完成时，必须说明:

1. 本批次是 `no code movement` 父叶残余判断。
2. `backend.runtime stop_split: false`。
3. `runtime.mutation.shared_governance stop_split: true` 已成立，但父级仍有 query DTO / run guard / response support / experiment limit 残余。
4. 下一步只能进入 BE-001CN-01 `runtime.query_support` 单子叶等价基线。
5. 本批没有创建 `src/runtime/query_support.rs`，没有迁移 DTO/helper，没有处理 response support、run guard、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

不得宣称 `backend.runtime` 已完成、parent support 已整体抽离、drained parent include 已删除、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `283-backend.runtime第四轮父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树保持 `backend.runtime stop_split: false`。
3. 下一候选固定为 BE-001CN-01 `runtime.query_support` 单子叶等价基线。
4. 本批保持 `no code movement`。
5. 治理门禁、Rust 相关等价测试、全量树覆盖和 `git diff --check` 均通过。
