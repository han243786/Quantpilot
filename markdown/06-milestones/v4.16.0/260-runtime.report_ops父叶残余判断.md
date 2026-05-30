# v4.16.0 runtime.report_ops 父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CD-01  
> 基准: `259-runtime.report_ops.runtime_report单叶closeout.md`、`255-runtime.report_ops单叶closeout.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops.runtime_report` 已完成 closeout 并设置 `stop_split: true`，但父级 `src/runtime/report_ops.rs` 仍保留六个 v1 ops/report handler。父级继续具备可拆价值，因此 `runtime.report_ops stop_split: false`；下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CD-01 `runtime.report_ops` 父叶残余判断 | 递归队列选择 |
| 规范矩阵 | v1 endpoint 测试缺口、父子通信、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops` | 保持 `stop_split: false` |
| 模块树 | `runtime.report_ops.v1_report_endpoints` | 下一候选 |

---

## 当前父级残余

真实代码文件:

```text
src/runtime/report_ops.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/mod.rs
src/backend/runtime/routes/report_ops.rs
```

已关闭子叶:

- `runtime.report_ops.runtime_report`
- `runtime.report_ops.runtime_report stop_split: true`
- 文件: `src/runtime/report_ops/runtime_report.rs`

父级残余 handler:

- `list_merge_records`
- `list_config_generations`
- `get_storage_health`
- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

---

## 残余分组判定

| 候选子叶 | 包含内容 | 判定 |
| --- | --- | --- |
| `runtime.report_ops.v1_report_endpoints` | `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report` | 先拆。三者同属 `/api/v1/reports/*` read projection，代码量和状态读取面最大，值得先建立等价基线 |
| `runtime.report_ops.merge_generation_health` | `list_merge_records`、`list_config_generations`、`get_storage_health` | 后拆。三者是 v1 support/health read endpoints，可在 report endpoints 后单独收束 |
| `runtime.evidence_health` | `get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts` | 不属于本父叶，仍作为 sibling 另起基线 |

结论:

```text
runtime.report_ops.runtime_report stop_split: true
runtime.report_ops stop_split: false
next: BE-001CE-01 runtime.report_ops.v1_report_endpoints 单子叶等价基线
```

---

## v1 report endpoint 边界

BE-001CE-01 只能冻结以下 public handler 的输入输出、调用顺序、状态读取面和测试缺口:

- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

它们对应的 endpoint:

- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`

必须在 BE-001CE-01 显式继承 v1 ops/report endpoints 专门测试缺口。BE-001CE-01 不新增测试、不迁移代码；后续 BE-001CE-02 才能决定是先补最小 endpoint smoke，还是先做纯物理抽离并继承现有 broad regression。

---

## 明确排除

以下内容不属于 BE-001CE-01:

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

## 验证继承

BE-001CC-04 提交已通过:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
```

BE-001CB-03 / BE-001CC-03 已覆盖目标 broad API 回归:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
```

本父叶残余判断提交前必须执行治理门禁:

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
BE-001CE-01 runtime.report_ops.v1_report_endpoints 单子叶等价基线
```

BE-001CE-01 不得创建 `src/runtime/report_ops/v1_report_endpoints.rs`，不得迁移 handler，不得处理 merge/generation/storage health endpoints，不得处理 `runtime.evidence_health`，不得修改 schema、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CD-01 完成时，必须说明:

1. 本批次是 `no code movement` 的父叶残余判断。
2. `runtime.report_ops.runtime_report stop_split: true` 已继承。
3. 父级仍保留 v1 report endpoints 与 merge/generation/storage health endpoints，因此 `runtime.report_ops stop_split: false`。
4. 下一步只能进入 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。
5. v1 ops/report endpoints 的专门测试缺口仍存在，且 BE-001CE-01 只能登记和冻结该缺口。
6. `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `260-runtime.report_ops父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.report_ops stop_split: false` 与 BE-001CE-01 下一候选进入模块树。
3. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
4. 下一步固定为 BE-001CE-01 `runtime.report_ops.v1_report_endpoints` 单子叶等价基线。
