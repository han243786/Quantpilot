# v4.16.0 system 抽离完成记录

> 版本类型: MINOR architecture。
> 基准: `08-system大模块分层统计.md` 与 `09-system.entry首批抽离记录.md`。
> 执行档位: 重型。
> 判定: `system.entry.backend_process` 代码抽离完成；`system` 大模块的 3 层、10 叶子白箱归属已完成登记。整理和重构仍不启动。

---

## 完成范围

本批次把后端进程启动职责从 `src/lib.rs` 迁入 `src/system/entry/backend_process.rs`:

1. `run_server` 位于 `system.entry.backend_process`。
2. `run_api_server` 位于 `system.entry.backend_process`。
3. 启动目录创建、CORS、安全头、JSON rejection middleware、后台观察任务、优雅关闭和关闭刷盘均位于 `system.entry.backend_process`。
4. `src/lib.rs` 只保留 crate root 模块声明、兼容 re-export 和业务/接口实现，不再拥有后端进程启动实现。
5. `quantpilot::run_server` 兼容入口不变。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 抽离批次完成口径 | 落地 |
| 规范矩阵 | 旧入口兼容、无 handler 迁移、无状态所有权迁移 | 落地 |
| 引导矩阵 | `system.entry.backend_process` 真实文件、public 方法与关键内部启动实现 | 完成 |
| 模块树 | `system.entry.backend_process` | 完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、附录 E.2 |
| 模块树节点 | `system.entry.backend_process` |
| 真实文件 | `src/system/entry/backend_process.rs`、`src/system/entry/mod.rs`、`src/system/mod.rs`、`src/lib.rs`、`src/main.rs` |
| public 方法 | `run_server`、兼容入口 `quantpilot::run_server` |
| 关键内部启动实现 | `run_api_server` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 完成后的调用链

```text
src/main.rs
  -> quantpilot::run_server()
  -> pub use system::entry::backend_process::run_server
  -> system.entry.backend_process::run_server()
  -> system.entry.backend_process::run_api_server()
  -> backend.interface_boundary
  -> build_app_router()
```

`new_app_state` 仍归属 `src/app_runtime_helpers.rs`，因为它是 AppState 工厂，不是进程启动编排本体。`build_app_router` 仍归属 `backend.interface_boundary`，因为它是后端接口边界，不是 system 真源。

---

## 未迁移边界

| 边界 | 归属 | 原因 |
| --- | --- | --- |
| `new_app_state` | `app_runtime_helpers` | 状态工厂，后续是否拆成 state bootstrap 需单独讨论 |
| `build_app_router` | `backend.interface_boundary` | 后端接口边界真源，system 不得拥有 route owner |
| handler 和 response schema | 各后端模块 | 本批次只抽启动编排 |
| `start.bat` / `start.ps1` | `system.entry.launch_scripts` | 已是独立脚本叶子，不需要迁入 Rust 模块 |
| `src-tauri/*` | `system.desktop_shell` | 桌面壳是独立 crate/配置叶子，本批次只登记白箱归属 |
| CI / release / Docker | `system.build_delivery` | 已是独立交付叶子，后续只按变更触发 |

---

## 等价证据

| 维度 | 结果 |
| --- | --- |
| public 入口 | `quantpilot::run_server` 仍存在 |
| 二进制入口 | `src/main.rs` 未改 |
| API 边界 | `build_app_router` 未迁移，route owner 不变 |
| 状态所有权 | `new_app_state` 未迁移，AppState 字段和锁顺序不变 |
| 代码验证 | `cargo check -p quantpilot` 通过 |

---

## 禁止事项

1. 不把 system 抽离完成解释为整理或重构完成。
2. 不把 `system.entry.backend_process` 解释为 API route owner。
3. 不让 system 横向修改 handler、runtime state 或 executor state。
4. 不删除旧 public 入口。
5. 不主动提出发布版本过渡或横向连接。
