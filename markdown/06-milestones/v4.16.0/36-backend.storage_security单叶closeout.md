# v4.16.0 backend.storage_security 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-06。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.storage_security` 当前完成 facade closeout，值得继续细分，但所有细分都必须先过安全和状态归属决策。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | storage/security 叶子整理、下一轮 L3 候选 | 扩展 |
| 规范矩阵 | 凭证、存储生命周期、认证、日志清洗、quota | 固化 |
| 引导矩阵 | `backend.storage_security`、安全测试与人工核查 | 扩展 |
| 模块树 | `backend.storage_security` | 单叶 closeout 与继续细分登记 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 安全系统 |
| 模块树节点 | `backend.storage_security` |
| 真实文件 | `src/backend/storage_security.rs`、`src/credential_api.rs`、`src/credential_vault.rs`、`src/storage_lifecycle.rs`、`src/safe_log.rs`、`src/auth/mod.rs`、`src/auth_middleware.rs`、`src/rate_limiter.rs`、`src/backup.rs` |
| public 方法 | `register_credential_routes`、`CredentialVault::load_from_storage_root`、`CredentialVault::set_service`、`persist_with_ttl`、`ensure_storage_quota`、`sanitize_secrets` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_auth`、credential/storage 相关单测、人工安全核查 |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | credential fields、storage paths、auth request、log text、quota request |
| 输出 | encrypted credential record、storage write result、auth response、redacted log |
| owner | `backend.storage_security` 拥有安全存储和认证边界 facade |
| 保留实现 | credential/auth/storage/safe_log handler 和 helper 均保留原文件 |
| 兼容桥 | `backend.interface_boundary -> backend.storage_security -> credential_api::register_credential_routes` |
| 回退点 | 回退到 `app_router` 直接调用 `credential_api::register_credential_routes` |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 值得继续拆分，但需要安全决策暂停 |
| 原因 | credential vault、credential API、storage lifecycle、auth/rate limit、safe_log/backup 都有独立安全约束 |
| 建议 L3 子叶 | `backend.storage_security.credential_vault`、`backend.storage_security.credential_api`、`backend.storage_security.storage_lifecycle`、`backend.storage_security.auth_boundary`、`backend.storage_security.safe_log_backup` |
| 暂停点 | 任何密钥格式、加密参数、认证状态、原子写、quota、日志清洗语义变化都必须先讨论 |

---

## closeout 结论

`backend.storage_security` 已完成当前 facade 整理 closeout。它值得继续细分，但属于高风险叶子，下一轮必须先做安全等价基线。
