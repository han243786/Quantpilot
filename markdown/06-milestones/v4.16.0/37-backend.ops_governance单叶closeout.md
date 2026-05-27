# v4.16.0 backend.ops_governance 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-07。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.ops_governance` 当前完成 facade closeout，值得继续细分；本批不改 ops handler、审计或运行时状态。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | ops governance 叶子整理、下一轮 L3 候选 | 扩展 |
| 规范矩阵 | sandbox、alert、snapshot、runbook、chaos、hotswap 审计 | 固化 |
| 引导矩阵 | `backend.ops_governance`、ops route owner | 扩展 |
| 模块树 | `backend.ops_governance` | 单叶 closeout 与继续细分登记 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 运维系统 |
| 模块树节点 | `backend.ops_governance` |
| 真实文件 | `src/backend/ops_governance.rs`、`src/alert_engine.rs`、`src/sandbox_verification.rs`、`src/snapshot_service.rs`、`src/runbook.rs`、`src/chaos_experiment.rs`、`src/hotswap_api.rs`、`src/collaboration.rs`、`src/migration_sender.rs` |
| public 方法 | `register_alert_routes`、`register_sandbox_verification_routes`、`register_snapshot_routes`、`register_runbook_routes`、`register_chaos_routes`、`register_hotswap_routes` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_collaboration`、人工 route 审核 |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | ops API request、runtime evidence、snapshot/signature context、hotswap request |
| 输出 | alert firing、sandbox report、snapshot、runbook result、chaos report、hotswap status |
| owner | `backend.ops_governance` 拥有 ops route facade，不拥有 runtime 核心状态 |
| 保留实现 | alert/sandbox/snapshot/runbook/chaos/hotswap 等 handler 均保留原文件 |
| 兼容桥 | `backend.interface_boundary -> backend.ops_governance -> existing ops handler` |
| 回退点 | 回退到 `app_router` 内旧 wrapper 或直接 route registration |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 值得继续拆分 |
| 原因 | sandbox、alert、snapshot、runbook、chaos、hotswap 都是独立运维能力，审计证据也不同 |
| 建议 L3 子叶 | `backend.ops_governance.sandbox`、`backend.ops_governance.alerts`、`backend.ops_governance.snapshots`、`backend.ops_governance.runbook`、`backend.ops_governance.chaos`、`backend.ops_governance.hotswap` |
| 暂停点 | 改审计、签名、chaos 默认状态、hotswap 安全边界时必须重新提案 |

---

## closeout 结论

`backend.ops_governance` 已完成当前 facade 整理 closeout。下一轮可按 ops 能力逐个拆分，但不能和 runtime 状态迁移混在一批。
