# v4.16.0 backend.app_state_wiring 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001C-08。
> 基准: `30-backend九叶模块壳抽离记录.md`。
> 判定: `backend.app_state_wiring` 当前不继续拆分；涉及 AppState 字段或状态所有权的下一步必须决策暂停。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | app state wiring 叶子整理 | 扩展 |
| 规范矩阵 | AppState owner、health、state attach、启动链边界 | 固化 |
| 引导矩阵 | `backend.app_state_wiring`、system/backend 连接点 | 固化 |
| 模块树 | `backend.app_state_wiring` | 单叶 closeout |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1 后端启动链与根2 backend 九叶 |
| 模块树节点 | `backend.app_state_wiring` |
| 真实文件 | `src/backend/app_state_wiring.rs`、`src/app_runtime_helpers.rs`、`src/lib.rs`、`src/system/entry/backend_process.rs` |
| public 方法 | `new_app_state`、`health`、`attach_state` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`、`cargo test -p quantpilot --test api_auth` |

---

## 白箱整理

| 项 | 结论 |
| --- | --- |
| 输入 | storage dirs、AppState、Router |
| 输出 | AppState、health response、Router with state |
| owner | `backend.app_state_wiring` 只拥有连接点 facade，不拥有 AppState 字段迁移 |
| 保留实现 | `src/app_runtime_helpers.rs` 仍保留 `new_app_state` 与 health 实现 |
| 兼容桥 | `system.entry.backend_process -> new_app_state -> build_app_router -> attach_state` |
| 回退点 | 回退到 `app_router` 直接 `.with_state(state)` 和 `health` handler |

---

## 细分价值判断

| 判断 | 结论 |
| --- | --- |
| 是否继续拆分 | 当前不继续拆分 |
| 原因 | 当前只是启动链与 AppState 的连接 facade；继续拆会触碰 AppState 字段、锁和状态归属 |
| 触发再拆条件 | 只有在开发者明确进入 AppState owner 重整或状态域迁移时，才讨论 `state_factory`、`health`、`state_schema` 子叶 |
| 暂停点 | 任一 AppState 字段、锁顺序、状态目录或 health schema 变化都必须先讨论 |

---

## closeout 结论

`backend.app_state_wiring` 已完成当前整理 closeout。它暂时停止细分，作为 system 与 backend 的连接保护层。
