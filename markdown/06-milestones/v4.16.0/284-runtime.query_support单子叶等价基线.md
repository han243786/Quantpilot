# v4.16.0 runtime.query_support 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CN-01  
> 基准: `283-backend.runtime第四轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 目标子叶: `runtime.query_support`  
> 模块树坐标: `root.backend.runtime.runtime.query_support`  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CN-01 `runtime.query_support` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | 父子通信、query DTO visibility、禁止跳步、release transition guard | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.query_support` | 新增 planned 子叶坐标 |
| 模块树 | `runtime.query_support` | 白箱登记 |

---

## 当前真实结构

已 closeout sibling / 父级:

- `backend.runtime.routes stop_split: true`
- `runtime.report_ops stop_split: true`
- `runtime.evidence_health stop_split: true`
- `runtime.backtest stop_split: true`
- `runtime.mutation.parameter_mutation stop_split: true`
- `runtime.mutation.ai_proposal stop_split: true`
- `runtime.mutation.shared_governance stop_split: true`
- `backend.runtime stop_split: false`

本批冻结的父级残余仍在:

```text
src/runtime/mod.rs
src/runtime/run.rs
src/runtime/mutation.rs
```

planned child 文件尚未创建。BE-001CN-01 只建立等价基线，不创建 `src/runtime/query_support.rs`，不迁移 query DTO 或 helper。

---

## 白箱边界

| public / helper | 当前文件 | 输入 | 输出 / 调用意义 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `RuntimeReplayQuery` | `src/runtime/mod.rs` | replay query params: cursor / limit / checkpoint / filters / key_only | `normalized_replay_options` 的 typed input | 不得改变 query field name、default、cursor 兼容或 filter semantics |
| `RuntimeParameterMutationListQuery` | `src/runtime/mod.rs` | parameter mutation list query params | parameter mutation record list filtering / pagination | 不得改变 `source_kind`、`source_id`、`limit`、`offset` semantics |
| `RuntimeAiProposalListQuery` | `src/runtime/mod.rs` | AI proposal list query params | AI proposal record list filtering | 不得改变 `source_kind`、`source_id`、`status` semantics |
| `RuntimeApprovalListQuery` | `src/runtime/run.rs` | approval list query params | approval review list filtering | 不得改变 `review_state` default or filtering semantics |
| `OpsDailyQuery` | `src/runtime/mutation.rs` | `/api/v1/reports/ops/daily` query params | ops daily report date selection | 不得改变 optional date behavior |
| `AuditWeeklyQuery` | `src/runtime/mutation.rs` | `/api/v1/reports/audit/weekly` query params | audit weekly report start selection | 不得改变 optional week_start behavior |
| `ResearchMonthlyQuery` | `src/runtime/mutation.rs` | `/api/v1/reports/research/monthly` query params | research monthly report month selection | 不得改变 optional month behavior |
| `clean_optional_filter` | `src/runtime/mod.rs` | optional string query value | trimmed non-empty optional filter | 不得改变 trim / empty-filter behavior |
| `normalized_replay_options` | `src/runtime/mod.rs` | `RuntimeReplayQuery` | `RuntimeReplayOptions` | 不得改变 default page size, max page size, cursor precedence, sequence cursor or filter mapping |

---

## 调用方基线

| 调用方文件 | 当前依赖 | 禁止事项 |
| --- | --- | --- |
| `src/runtime/run/replay_status.rs` | `RuntimeReplayQuery`、`normalized_replay_options` | 不得改变 run replay/status endpoint contract |
| `src/runtime/backtest/replay.rs` | `RuntimeReplayQuery`、`normalized_replay_options` | 不得改变 backtest replay endpoint contract |
| `src/runtime/mutation/parameter_mutation/record_query.rs` | `RuntimeParameterMutationListQuery`、`clean_optional_filter` | 不得改变 mutation record filter / pagination |
| `src/runtime/mutation/ai_proposal/record_query.rs` | `RuntimeAiProposalListQuery`、`clean_optional_filter` | 不得改变 AI proposal record filter |
| `src/runtime/mutation/ai_proposal/approval_review.rs` | `RuntimeApprovalListQuery` | 不得改变 approval list review_state filter |
| `src/runtime/report_ops/v1_report_endpoints.rs` | `OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery` | 不得改变 v1 report endpoint query contract |

---

## 现有等价证据

当前已有自动化覆盖:

- `tests/api_run.rs::run_replay_endpoint_exposes_paginated_ordered_timeline`
- `tests/api_backtest.rs::backtest_replay_endpoint_exposes_paginated_ordered_timeline`
- `tests/api_mutation.rs::runtime_parameter_mutation_creates_persisted_proposal_and_key_event`
- `tests/api_ai_proposal.rs::runtime_ai_proposal_creates_static_checked_record_and_key_events`
- `tests/api_ai_proposal.rs::runtime_ai_proposal_static_check_failed_candidate_is_auditable`
- `tests/api_v1_reports.rs::v1_report_endpoints_return_minimal_contracts`

BE-001CN-02 抽离方案不需要先补 endpoint smoke，但必须把 `api_run`、`api_backtest`、`api_mutation`、`api_ai_proposal` 与 `api_v1_reports` 作为实际抽离前后的硬门禁。

---

## 父子通信规则

`runtime.query_support` 后续若实际抽离，只能作为 `backend.runtime` 下的 query DTO / normalization child。通信路径必须保持父级中介:

```text
runtime child callers
  -> src/runtime/mod.rs controlled query surface
  -> runtime.query_support
```

本叶白箱范围显式覆盖 filter normalization 与 replay option normalization；后续方案不得把二者拆到 sibling child 或 route facade。

迁入 child 后，query DTO fields 必须保持 sibling child 可访问，优先使用 `pub(super)` field visibility，不得为了方便升级为 public API。开发者未明确进入发布版本过渡前，不得让 route facade、frontend caller、schema owner、runtime persistence owner、storage lifecycle owner 或 `AppState` 横向直连该 planned child。

---

## 明确排除

- 不创建 `src/runtime/query_support.rs`。
- 不迁移 `RuntimeReplayQuery`、`RuntimeParameterMutationListQuery`、`RuntimeAiProposalListQuery`、`RuntimeApprovalListQuery`、`OpsDailyQuery`、`AuditWeeklyQuery`、`ResearchMonthlyQuery`、`clean_optional_filter` 或 `normalized_replay_options`。
- 不处理 `DiscardRuntimeArtifactResponse`、`MergeRecordsResponse`、`MergeRecordEntry`、`RunInProgressGuard`、`MAX_EXPERIMENT_VARIANTS`、`DEFAULT_REPLAY_PAGE_SIZE` 或 `MAX_REPLAY_PAGE_SIZE`。
- 不删除 `include!("run.rs")`、`include!("mutation.rs")` 或 drained `include!("backtest.rs")`。
- 不回改 `backend.runtime.routes`、`runtime.report_ops`、`runtime.evidence_health`、`runtime.backtest`、`runtime.mutation.parameter_mutation`、`runtime.mutation.ai_proposal` 或 `runtime.mutation.shared_governance` closed child。
- 不迁移 `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、lock order 或 release transition guard。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前仍需执行:

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
BE-001CN-02 runtime.query_support 抽离方案
```

BE-001CN-02 只能决定 planned child 文件、父级声明 / controlled import、允许迁移清单、field visibility、验证命令和回退点；不得直接宣称 query DTO/helper 已迁移。

---

## 幻觉检查点

AI 声称 BE-001CN-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. planned child 文件 `src/runtime/query_support.rs` 尚未创建。
3. query DTO 与 normalization helper 仍在 `src/runtime/mod.rs`、`src/runtime/run.rs`、`src/runtime/mutation.rs`。
4. 后续实际迁移必须处理 query DTO field visibility，优先评估 `pub(super)`，不得扩大为 public API。
5. 下一步只能进入 BE-001CN-02 抽离方案。
6. response support、run guard、experiment limit、drained parent include、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState`、lock order 和 release transition guard 均未迁移。

不得宣称 query support 已抽离、`backend.runtime` 已完成、parent support 已整体抽离、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `284-runtime.query_support单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `runtime.query_support` planned 子叶白箱坐标，但不登记不存在的真实文件路径。
3. 治理门禁能阻止跳过 BE-001CN-02 直接创建 child 文件或迁移 query DTO/helper。
4. 治理门禁、全量树覆盖、Rust 等价测试和 `git diff --check` 均通过。
