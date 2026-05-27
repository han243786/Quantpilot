# v4.16.0 backend.interface_boundary 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-01。
> 基准: `29-backend.interface_boundary等价基线.md`、`30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.interface_boundary` 作为后端接口父级 facade 已完成本阶段整理；不继续拆分本叶，后续变化应落到它管理的 8 个子叶。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001C 九叶逐叶 closeout | 扩展 |
| 规范矩阵 | route owner 父入口、旧 handler 保留、状态所有权冻结 | 固化 |
| 引导矩阵 | `backend.interface_boundary` 白箱节点、全量树后端入口 | 固化 |
| 模块树 | `backend.interface_boundary` | 单叶 closeout |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 九叶模块壳 |
| 模块树节点 | `backend.interface_boundary` |
| 真实文件 | `src/backend/interface_boundary.rs`、`src/app_router.rs` |
| public 方法 | `build_app_router`、`get_capabilities`、`health`、`register_*_routes`、`attach_state` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_backtest`、`tools/check-matrix-governance.ps1` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | `AppState`、HTTP request、route registration |
| 输出 | Axum Router、API response、SPA fallback |
| owner | `backend.interface_boundary` 只拥有父级 route facade，不拥有 handler 行为 |
| 保留实现 | `src/app_router.rs` 仍定义 `build_app_router`；真实 handler 仍在各旧文件 |
| 兼容桥 | `build_app_router -> backend.interface_boundary -> backend leaf facade -> existing handler` |
| 回退点 | 可以直接回退到 `src/app_router.rs` 内原 route registration 调用链 |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 不继续拆分本叶 |
| 原因 | 本叶是父级分发节点，再拆只会把 route facade 拆成目录美化；实际业务边界已经下沉到 8 个叶子 |
| 后续动作 | 修改 route owner 时改本叶；业务 handler 迁移时改对应子叶 |

---

## closeout 结论

`backend.interface_boundary` 本阶段完成整理 closeout。它是父模块路由入口，不再向下拆；继续拆分应发生在 runtime、graph_compile、strategy_config、storage_security、ops_governance 等子叶。
