# v4.16.0 runtime.report_ops.v1_report_endpoints 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CE-02  
> 基准: `261-runtime.report_ops.v1_report_endpoints单子叶等价基线.md`、`260-runtime.report_ops父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops.v1_report_endpoints` 可以继续抽离，但不能在专门测试缺口未处理时直接移动 handler。本方案采用 test-first: 下一批 BE-001CE-03 只补最小 endpoint smoke，不迁移 handler；BE-001CE-04 才允许创建 child module 并迁移三个 v1 report handler。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CE-02 `runtime.report_ops.v1_report_endpoints` 抽离方案 | 方案优化 |
| 规范矩阵 | test-first、父级 re-export、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.v1_report_endpoints` | 方案登记 |
| 模块树 | `runtime.report_ops.v1_report_endpoints` | 下一批先补 smoke |

---

## 方案决策

由于 BE-001CE-01 已确认 `/api/v1/reports/*` 三个 endpoint 没有专门测试，且 handler 内部读取 `runs`、`alert_firings`、`approval_records`、`ai_proposals`、`parameter_mutations`、`hotswap_records`、`backtests` 与多组 `evidence_metrics`，直接抽离会让行为缺口继续扩大。

因此本叶采用两步执行:

1. BE-001CE-03: 只新增最小 endpoint smoke，确认三个 v1 report endpoint 可返回基础 JSON contract。
2. BE-001CE-04: 在 smoke 通过后创建 child module，并迁移三个 handler。

本方案本身不移动代码、不创建测试文件。

---

## BE-001CE-03 允许动作

BE-001CE-03 只允许新增一个最小 API smoke 测试文件:

```text
tests/api_v1_reports.rs
```

允许覆盖的 endpoint:

- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`

允许断言:

- HTTP status 为 `200 OK`。
- `report_type` 分别为 `ops`、`audit`、`research`。
- `generated_at` 存在且为字符串。
- 每个响应包含对应核心字段，如 `summary`、`total_approvals`、`strategy_performance`。

禁止在 BE-001CE-03 中迁移 handler、重构 schema、改 route order、写入 mock persistence 或变更 frontend caller。

---

## BE-001CE-04 允许动作

BE-001CE-04 只有在 BE-001CE-03 smoke 通过并提交后，才允许:

新增 child module:

```text
src/runtime/report_ops/v1_report_endpoints.rs
```

父级 `src/runtime/report_ops.rs` 增加:

```rust
mod v1_report_endpoints;

pub(crate) use v1_report_endpoints::{
    get_audit_weekly_report, get_ops_daily_report, get_research_monthly_report,
};
```

迁移清单只限:

- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

---

## 保持不变

- `src/backend/runtime/routes/report_ops.rs` route facade。
- `src/runtime/mod.rs` 既有 `report_ops` re-export 调用面。
- `src/runtime/report_ops/runtime_report.rs` closed child。
- `/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health`。
- `list_merge_records`、`list_config_generations`、`get_storage_health`。
- `runtime.report_ops.merge_generation_health` 后续候选。
- `runtime.evidence_health` sibling。
- `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner。
- release transition guard。

---

## 回退点

BE-001CE-03 若 smoke 不稳定，停止在测试批次，不进入 handler 迁移。

BE-001CE-04 若抽离后编译或 smoke 失败，回退点只有:

- 删除 `mod v1_report_endpoints`。
- 删除 `pub(crate) use v1_report_endpoints::{...}`。
- 将三个 handler 恢复到 `src/runtime/report_ops.rs`。

不得回改 BE-001CE-03 smoke 测试来掩盖抽离失败。

---

## 验证命令

BE-001CE-03 必须执行:

```powershell
cargo test -p quantpilot --test api_v1_reports
cargo fmt --check
cargo check -p quantpilot
```

BE-001CE-04 必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
```

本方案提交前必须执行治理门禁:

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
BE-001CE-03 runtime.report_ops.v1_report_endpoints endpoint smoke 补测
```

BE-001CE-03 不得创建 `src/runtime/report_ops/v1_report_endpoints.rs`，不得迁移 handler，不得处理 merge/generation/storage health endpoints，不得处理 `runtime.evidence_health`，不得修改 schema、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CE-02 完成时，必须说明:

1. 本批次是 `no code movement` 的抽离方案。
2. 方案选择 test-first，BE-001CE-03 只补 `api_v1_reports` smoke。
3. child module 尚未创建，三个 handler 仍在 `src/runtime/report_ops.rs`。
4. BE-001CE-04 才允许迁移 handler。
5. merge/generation/storage health endpoints、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `262-runtime.report_ops.v1_report_endpoints抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. BE-001CE-03 test-first 下一步进入模块树。
3. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
4. 下一步固定为 BE-001CE-03 endpoint smoke 补测。
