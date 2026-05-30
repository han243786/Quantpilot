# v4.16.0 runtime.report_ops.v1_report_endpoints 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CE-01  
> 基准: `260-runtime.report_ops父叶残余判断.md`、`259-runtime.report_ops.runtime_report单叶closeout.md`、`13-递归模块化全局根流程.md`  
> 判定: 建立 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线，冻结 `/api/v1/reports/*` 三个 report projection handler 的输入输出、状态读取面、父级出口和测试缺口。当前 `no code movement`，不得创建 `src/runtime/report_ops/v1_report_endpoints.rs`，不得迁移 handler。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线 | 建立下一抽离基线 |
| 规范矩阵 | v1 report endpoint contract、测试缺口、父级 re-export | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.v1_report_endpoints` | 新增 planned child 坐标 |
| 模块树 | `runtime.report_ops.v1_report_endpoints` | `stop_split: pending` |

---

## 当前真实边界

真实代码文件:

```text
src/runtime/report_ops.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/mod.rs
src/backend/runtime/routes/report_ops.rs
src/runtime/mutation.rs
src/frontend_api_types.rs
```

当前 planned child 文件尚未创建:

```text
src/runtime/report_ops/v1_report_endpoints.rs
```

父级保持:

```text
runtime.report_ops stop_split: false
runtime.report_ops.v1_report_endpoints stop_split: pending
```

本子叶只冻结三个 public handler:

- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

---

## Endpoint / handler 映射

| Endpoint | Handler | Query | Response |
| --- | --- | --- | --- |
| `/api/v1/reports/ops/daily` | `get_ops_daily_report` | `OpsDailyQuery` | `OpsDailyReport` |
| `/api/v1/reports/audit/weekly` | `get_audit_weekly_report` | `AuditWeeklyQuery` | `AuditWeeklyReport` |
| `/api/v1/reports/research/monthly` | `get_research_monthly_report` | `ResearchMonthlyQuery` | `ResearchMonthlyReport` |

route facade 保持在 `src/backend/runtime/routes/report_ops.rs`，通过 `crate::runtime as runtime_handlers` 调用 `src/runtime/mod.rs` 受控 re-export。BE-001CE-01 不改变 route path、method、order 或 auth extractor。

---

## 白箱输入输出

输入:

- `auth::UserId`
- `State<AppState>`
- `Query<OpsDailyQuery>`
- `Query<AuditWeeklyQuery>`
- `Query<ResearchMonthlyQuery>`

输出:

- `Json<OpsDailyReport>`
- `Json<AuditWeeklyReport>`
- `Json<ResearchMonthlyReport>`
- `(StatusCode, String)` error tuple

---

## 状态读取面

`get_ops_daily_report` 读取:

- `state.runs`
- `state.alert_firings`
- `state.evidence_metrics.compact_projection_source_event_count_total`
- `state.evidence_metrics.mutation_proposal_rejected_count`
- `state.evidence_metrics.mutation_proposal_created_count`
- `state.evidence_metrics.replay_page_count`
- `state.evidence_metrics.report_generation_failure_count`

`get_audit_weekly_report` 读取:

- `state.approval_records`
- `state.ai_proposals`
- `state.parameter_mutations`
- `state.hotswap_records`
- `state.evidence_metrics.mutation_rollback_applied_count`

`get_research_monthly_report` 读取:

- `state.backtests`
- `state.ai_proposals`

以上状态读取面在后续抽离时必须原样保留；不得借抽离机会迁移 `AppState`、state owner、schema owner、frontend caller 或 runtime persistence owner。

---

## 允许迁移清单

若 BE-001CE-02 方案允许进入实际抽离，BE-001CE-03 只可迁移:

- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

父级可新增:

```rust
mod v1_report_endpoints;

pub(crate) use v1_report_endpoints::{
    get_audit_weekly_report, get_ops_daily_report, get_research_monthly_report,
};
```

---

## 明确排除

以下内容不属于本子叶:

- `create_runtime_report`
- `list_runtime_reports`
- `get_runtime_report_detail`
- `export_runtime_report_artifact`
- `runtime.report_ops.runtime_report`
- `list_merge_records`
- `list_config_generations`
- `get_storage_health`
- `runtime.report_ops.merge_generation_health`
- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`
- `runtime.evidence_health`
- `AppState`
- schema owner
- frontend caller
- runtime persistence owner
- storage lifecycle owner
- release transition guard

---

## 测试缺口冻结

仓库当前未发现以下 endpoint 的专门测试:

- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`

BE-001CE-01 不补测试、不移动代码。BE-001CE-02 必须在方案中显式决定:

1. 先补最小 endpoint smoke，再做物理抽离。
2. 或先做纯物理抽离，并继续继承现有 broad regression 风险。

无论选哪条，不能把 `cargo check` 或现有 broad API test 误称为 v1 report endpoint 专门覆盖。

---

## 验证继承

BE-001CD-01 已确认父叶队列；BE-001CB-03 / BE-001CC-03 已覆盖 broad regression:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
```

本基线提交前必须执行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CE-02 runtime.report_ops.v1_report_endpoints 抽离方案
```

BE-001CE-02 只能在 `runtime.report_ops.v1_report_endpoints` 内做方案优化和测试策略取舍。不得创建 child 文件、不得迁移 handler、不得处理 merge/generation/storage health endpoints、不得处理 `runtime.evidence_health`，不得修改 schema、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CE-01 完成时，必须说明:

1. 本批次是 `no code movement` 的等价基线。
2. `src/runtime/report_ops/v1_report_endpoints.rs` 尚未创建。
3. 三个目标 handler 仍在 `src/runtime/report_ops.rs`。
4. v1 report endpoint 专门测试缺口已冻结，尚未补齐。
5. 下一步只能进入 BE-001CE-02 抽离方案。
6. `runtime.report_ops.merge_generation_health`、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.report_ops.v1_report_endpoints` planned child 坐标进入模块树。
3. 测试缺口被显式登记，不得误称已覆盖。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
5. 下一步固定为 BE-001CE-02 `runtime.report_ops.v1_report_endpoints` 抽离方案。
