# v4.16.0 backend.runtime 父叶残余判断

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CA-01  
> 基准: `250-backend.runtime.routes第六轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes` 已设置 `stop_split: true`，但 `backend.runtime` 父叶仍持有 `src/runtime/mod.rs` 中的 runtime report / evidence health / ops report handler 与 helper 残余。因此 `backend.runtime` 当前保持 `stop_split: false`。下一步只能进入 BE-001CB-01 `runtime.report_ops` 单子叶等价基线。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CA-01 `backend.runtime` 父叶残余判断 | 父叶判断 |
| 规范矩阵 | route aggregate closeout、handler owner、父子通信、发布过渡保护 | 继续冻结 |
| 引导矩阵 | `root.backend.runtime` | 选择下一候选 |
| 模块树 | `backend.runtime`、`runtime.report_ops` | 父叶保持可拆 |

---

## 当前真实边界

`backend.runtime` 当前有两个层面:

1. `src/backend/runtime.rs`: 只保留 `MODULE_ID`、`pub mod routes` 与 `register_routes(router)` facade。
2. `src/runtime/mod.rs`: 仍保留 route-facing runtime handler re-export、runtime report / evidence health / ops report handler 与 shared helper。

`backend.runtime.routes` 已完成 route aggregate 收口:

- `src/backend/runtime/routes.rs`
- `src/backend/runtime/routes/run.rs`
- `src/backend/runtime/routes/backtest.rs`
- `src/backend/runtime/routes/event_stream.rs`
- `src/backend/runtime/routes/evidence.rs`
- `src/backend/runtime/routes/mutation.rs`
- `src/backend/runtime/routes/experiment.rs`
- `src/backend/runtime/routes/report_ops.rs`

这些 route facade 均只做父级委托，不代表 `src/runtime/mod.rs` 中的 handler owner 已迁移。

---

## 残余候选

| 候选 | 当前 owner | 代表入口 | 判断 |
| --- | --- | --- | --- |
| `runtime.report_ops` | `src/runtime/mod.rs` | `create_runtime_report`、`list_runtime_reports`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report` | 值得进入下一轮基线 |
| `runtime.evidence_health` | `src/runtime/mod.rs` | `get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts` | 保留为后续 sibling，不并入本批 |
| `runtime.shared_helpers` | `src/runtime/mod.rs` | `clean_optional_filter`、`normalized_replay_options`、`RunInProgressGuard` | 暂不直接抽离，需等待具体调用 owner 清晰 |
| `runtime.state_persistence_boundary` | `AppState`、`runtime_persistence`、`storage_lifecycle` | runs/backtests/reports/approvals/snapshots dirs 与 locks | 不在本批迁移 |

---

## 父叶判断

`backend.runtime` 当前不能设置 `stop_split: true`，理由:

1. `src/runtime/mod.rs` 仍有多个直接 route target handler。
2. `runtime.report_ops` 具有清晰的 public handler 集合、route facade 调用点、状态读取边界和测试证据。
3. `runtime.evidence_health`、shared helper、state/persistence owner 仍需要在后续 sibling 或父叶判断中处理。
4. 若直接关闭 `backend.runtime`，会把 route facade 已收口误宣称为 handler 层、schema 层、state 层同时完成，扩大 AI 幻觉缺口。

结论:

```text
backend.runtime stop_split: false
next: BE-001CB-01 runtime.report_ops 单子叶等价基线
```

---

## 父子通信规则

固定为:

```text
backend.interface_boundary
  -> backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.report_ops
  -> runtime.report_ops (planned child)
  -> AppState / runtime_persistence / storage_lifecycle / report_store_dir
```

BE-001CB-01 只能建立 `runtime.report_ops` 等价基线，不得直接创建 `src/runtime/report_ops.rs`，不得迁移 handler，不得改变 `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、route order 或 release transition guard。

---

## 回归证据

本父叶判断继承 BE-001BY-03 / BE-001BY-04 / BE-001BZ-01 已通过验证:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

本批自身是文档治理判断，代码动作保持 `no code movement`。

---

## 下一步

下一步只能进入:

```text
BE-001CB-01 runtime.report_ops 单子叶等价基线
```

BE-001CB-01 需要冻结 `runtime.report_ops` 的 route path、handler owner、输入输出、状态读取边界、persistence owner 保留边界、前端调用者和回归证据。不得绕过基线直接进入抽离方案或实际迁移。

---

## 幻觉检查点

AI 声称 BE-001CA-01 完成时，必须说明:

1. `backend.runtime.routes` 已关闭，但 `backend.runtime` 未关闭。
2. `backend.runtime stop_split: false`。
3. 下一步只能是 BE-001CB-01 `runtime.report_ops` 单子叶等价基线。
4. `src/runtime/mod.rs` 中 report/evidence/ops handler 仍未迁移。
5. `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均未迁移。

不得宣称 backend 顶层完成、runtime handler 整理完成、发布过渡已启动或 Rust 重构完成。

---

## 验收标准

1. `251-backend.runtime父叶残余判断.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `backend.runtime` 明确保持 `stop_split: false`。
3. 下一步固定为 BE-001CB-01 `runtime.report_ops` 单子叶等价基线。
4. 本批保持 `no code movement`。
