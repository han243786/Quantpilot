# v4.16.0 backend.ops_governance 子叶抽离完成记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001E-06。
> 基准: `37-backend.ops_governance单叶closeout.md`、`41-backend其余八叶模块壳抽离记录.md`。
> 判定: `backend.ops_governance` 子叶抽离完成；只建立 sandbox、alerts、snapshots、runbook、chaos、hotswap route facade，不迁移 ops handler。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001E 逐叶完成 | 固化 |
| 规范矩阵 | ops route facade、runtime/executor 横向禁止 | 固化 |
| 引导矩阵 | `backend.ops_governance.*` 子叶 | 扩展 |
| 模块树 | `backend.ops_governance` | 子叶抽离完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 ops governance |
| 模块树节点 | `backend.ops_governance.alerts`、`backend.ops_governance.chaos`、`backend.ops_governance.hotswap`、`backend.ops_governance.runbook`、`backend.ops_governance.sandbox`、`backend.ops_governance.snapshots` |
| 真实文件 | `src/backend/ops_governance.rs`、`src/backend/ops_governance/alerts.rs`、`src/backend/ops_governance/chaos.rs`、`src/backend/ops_governance/hotswap.rs`、`src/backend/ops_governance/runbook.rs`、`src/backend/ops_governance/sandbox.rs`、`src/backend/ops_governance/snapshots.rs`、`src/alert_engine.rs`、`src/chaos_experiment.rs`、`src/hotswap_api.rs`、`src/runbook.rs`、`src/sandbox_verification.rs`、`src/snapshot_service.rs` |
| public 方法 | `register_alert_routes`、`register_chaos_routes`、`register_hotswap_routes`、`register_runbook_routes`、`register_sandbox_verification_routes`、`register_snapshot_routes` |
| 测试/门禁 | `cargo check -p quantpilot`、ops API tests、`tools/check-matrix-governance.ps1` |

---

## 子叶抽离结果

| 子叶 | 职责 | 保留实现 |
| --- | --- | --- |
| `backend.ops_governance.alerts` | alerts route facade | `src/alert_engine.rs` |
| `backend.ops_governance.chaos` | chaos route facade | `src/chaos_experiment.rs` |
| `backend.ops_governance.hotswap` | hotswap route facade | `src/hotswap_api.rs` |
| `backend.ops_governance.runbook` | runbook route facade | `src/runbook.rs` |
| `backend.ops_governance.sandbox` | sandbox verification route facade | `src/sandbox_verification.rs` |
| `backend.ops_governance.snapshots` | snapshot route facade | `src/snapshot_service.rs` |

## 等价结论

ops handler、audit、sandbox evidence、snapshot signing、runbook semantics、chaos default-off 和 hotswap audit 均保持原位。本批不改 runtime/executor/release transition。
