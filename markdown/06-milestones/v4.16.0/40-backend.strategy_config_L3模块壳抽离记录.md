# v4.16.0 backend.strategy_config L3 模块壳抽离记录

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001D。
> 基准: `33-backend.strategy_config单叶closeout.md`。
> 判定: 启动 `backend.strategy_config` 的 L3 递归抽离试水；只建立 artifact、preflight、diff、AI proposal binding 四个子叶 facade，不迁移 handler、schema、response contract 或 AI proposal 校验逻辑。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | backend R5 局部递归、BE-001D L3 模块壳抽离 | 扩展 |
| 规范矩阵 | strategy config 父子通信、route facade、handler 保留 | 固化 |
| 引导矩阵 | `backend.strategy_config.*` 四个 L3 子叶、全量树后端入口 | 扩展 |
| 模块树 | `backend.strategy_config.artifact`、`backend.strategy_config.preflight`、`backend.strategy_config.diff`、`backend.strategy_config.ai_proposal_binding` | 新增白箱节点 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 九叶模块壳 |
| 模块树节点 | `backend.strategy_config` 及四个 L3 子叶 |
| 真实文件 | `src/backend/strategy_config.rs`、`src/backend/strategy_config/artifact.rs`、`src/backend/strategy_config/preflight.rs`、`src/backend/strategy_config/diff.rs`、`src/backend/strategy_config/ai_proposal_binding.rs`、`src/strategy_config_api.rs` |
| public 方法 | `register_routes`、`register_strategy_config_artifact_route`、`register_strategy_config_preflight_route`、`register_strategy_config_diff_route`、`register_strategy_config_routes` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_ai_proposal`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 抽离内容

| 子叶 | 新 facade 文件 | 真实实现保留 | 本批输出 |
| --- | --- | --- | --- |
| `backend.strategy_config.artifact` | `src/backend/strategy_config/artifact.rs` | `src/strategy_config_api.rs` | `/api/v1/strategy-config/artifact` route facade |
| `backend.strategy_config.preflight` | `src/backend/strategy_config/preflight.rs` | `src/strategy_config_api.rs` | `/api/v1/strategy-config/preflight` route facade |
| `backend.strategy_config.diff` | `src/backend/strategy_config/diff.rs` | `src/strategy_config_api.rs` | `/api/v1/strategy-config/diff` route facade |
| `backend.strategy_config.ai_proposal_binding` | `src/backend/strategy_config/ai_proposal_binding.rs` | `src/strategy_config_api.rs`、`src/runtime/mutation.rs` | 当前为 no-op facade，登记 AI proposal 配置域绑定边界 |

---

## 等价边界

| 项 | 结论 |
| --- | --- |
| route 顺序 | 保持 artifact -> preflight -> diff |
| handler 位置 | handler 仍保留在 `src/strategy_config_api.rs` |
| schema 位置 | `StrategyConfigArtifact`、preflight、diff、evidence diff schema 均保留原位 |
| AI proposal binding | 只登记 L3 边界，不迁移 `runtime/mutation.rs` 校验逻辑 |
| 回退点 | 可回退到 `backend.strategy_config::register_routes -> strategy_config_api::register_strategy_config_routes` 单跳注册 |

---

## 禁止事项

1. 不宣称 `src/strategy_config_api.rs` 已拆完。
2. 不迁移 `StrategyConfigArtifactRequest`、`StrategyConfigArtifact`、`StrategyConfigPreflightReport`、`StrategyConfigDiffReport`。
3. 不改变 `/api/v1/strategy-config/*` response schema。
4. 不把 AI proposal binding 从 runtime mutation 迁出。
5. 不进入 strategy config handler 整理或旧实现退役。

---

## closeout 结论

BE-001D 完成 `backend.strategy_config` 的 L3 模块壳抽离。当前只完成父子 facade 拆分和白箱坐标建立，下一步若继续该叶，应该逐个给 artifact、preflight、diff、AI proposal binding 建立等价基线，再讨论是否迁移内部 handler。
