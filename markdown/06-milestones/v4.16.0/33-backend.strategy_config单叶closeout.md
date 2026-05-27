# v4.16.0 backend.strategy_config 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-03。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.strategy_config` 当前完成 facade closeout，但值得继续细分；本批只登记下一层候选，不迁移 handler。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | strategy config 叶子整理、下一轮 L3 候选 | 扩展 |
| 规范矩阵 | preflight、artifact、diff、AI proposal 配置绑定 | 固化 |
| 引导矩阵 | `backend.strategy_config`、后端 API tests | 扩展 |
| 模块树 | `backend.strategy_config` | 单叶 closeout 与继续细分登记 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 strategy config API |
| 模块树节点 | `backend.strategy_config` |
| 真实文件 | `src/backend/strategy_config.rs`、`src/strategy_config_api.rs`、`src/frontend_api_types.rs`、`src/frontend_runtime_mapping.rs` |
| public 方法 | `register_strategy_config_routes`、`/api/v1/strategy-config/artifact`、`/api/v1/strategy-config/preflight`、`/api/v1/strategy-config/diff` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_ai_proposal` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | strategy input、capability snapshot、compile evidence、artifact draft |
| 输出 | strategy config artifact、preflight readiness、domain diff、AI proposal binding |
| owner | `backend.strategy_config` 拥有配置契约边界，不拥有 runtime state |
| 保留实现 | `src/strategy_config_api.rs` 仍保留全部 handler 和 schema |
| 兼容桥 | `backend.interface_boundary -> backend.strategy_config -> strategy_config_api::register_strategy_config_routes` |
| 回退点 | 回退到 `app_router` 直接调用 `register_strategy_config_routes` |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 值得继续拆分，但不在本批执行 |
| 原因 | `src/strategy_config_api.rs` 规模大，artifact、preflight、diff、AI proposal binding 有不同 owner 和验证证据 |
| 建议 L3 子叶 | `backend.strategy_config.artifact`、`backend.strategy_config.preflight`、`backend.strategy_config.diff`、`backend.strategy_config.ai_proposal_binding` |
| 暂停点 | 任一子叶迁移 response schema、capability 语义或 AI proposal contract 时必须重新提案 |

---

## closeout 结论

`backend.strategy_config` 已完成当前 facade 整理 closeout。下一步适合进入 L3 细分提案，但只能先做 artifact/preflight/diff/AI proposal binding 的等价基线。
