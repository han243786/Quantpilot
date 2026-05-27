# v4.16.0 backend.graph_compile 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-05。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.graph_compile` 当前完成 facade closeout，值得继续细分；本批不改 graph、QS 或 compile handler。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | graph/compile 叶子整理、下一轮 L3 候选 | 扩展 |
| 规范矩阵 | graph version、QS 安全、compile diagnostics | 固化 |
| 引导矩阵 | `backend.graph_compile`、graph/compile tests | 扩展 |
| 模块树 | `backend.graph_compile` | 单叶 closeout 与继续细分登记 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 编译系统与图存储系统 |
| 模块树节点 | `backend.graph_compile` |
| 真实文件 | `src/backend/graph_compile.rs`、`src/graph_api.rs`、`src/graph_quantscript_api.rs`、`src/graph_version_compare.rs`、`src/compile_api.rs`、`src/compile_artifact_builders.rs`、`src/compile_diagnostics.rs` |
| public 方法 | `register_graph_routes`、`register_graph_quantscript_routes`、`register_compile_routes`、`/api/runtime/compile` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_graph_versions`、`cargo test -p quantpilot --test quantscript_real_strategy_authoring` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | graph JSON、QuantScript source、compile request |
| 输出 | graph version、compile summary、diagnostics、artifact bundle |
| owner | `backend.graph_compile` 拥有 graph/compile route facade，不拥有 runtime state |
| 保留实现 | graph、QS graph、compile handler 均保留原文件 |
| 兼容桥 | `backend.interface_boundary -> backend.graph_compile -> graph/compile register routes` |
| 回退点 | 回退到 `app_router` 直接调用 graph/compile route registration |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 值得继续拆分 |
| 原因 | graph CRUD/version、QS graph、runtime compile、diagnostics/artifacts owner 不同且测试证据不同 |
| 建议 L3 子叶 | `backend.graph_compile.graph_api`、`backend.graph_compile.quantscript_graph`、`backend.graph_compile.compile_api`、`backend.graph_compile.diagnostics_artifacts` |
| 暂停点 | 改 compile summary、diagnostics code、QS 安全边界或 graph version schema 时必须重新提案 |

---

## closeout 结论

`backend.graph_compile` 已完成当前 facade 整理 closeout。后续应以 graph、QS graph、compile、diagnostics/artifacts 四个方向递归处理。
