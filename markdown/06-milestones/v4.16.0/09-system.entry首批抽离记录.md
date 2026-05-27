# v4.16.0 system.entry 首批抽离记录

> 版本类型: MINOR architecture / governance。
> 基准: `08-system大模块分层统计.md`。
> 执行档位: 重型。
> 判定: 以 `system.entry.backend_process` 作为 system 试水抽离；只移动启动 public 入口，不迁移 router、handler、runtime state 或 executor state。

---

## 落地范围

本批次完成 `system.entry.backend_process` 的第一刀代码抽离:

1. 新增 `src/system/mod.rs` 作为 system 父模块入口。
2. 新增 `src/system/entry/mod.rs` 作为 system.entry 二级域入口。
3. 新增 `src/system/entry/backend_process.rs` 承载 public `run_server`。
4. `src/lib.rs` 保留 `pub use system::entry::backend_process::run_server`，确保旧调用 `quantpilot::run_server()` 不变。
5. `src/main.rs` 不改，仍只调用 `quantpilot::run_server().await`。

本批次不移动 `run_api_server`，不移动 `build_app_router`，不移动 `new_app_state`，不改 HTTP route、response schema、存储路径、锁顺序或后台任务语义。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 抽离状态机、试水批次、等价验证 | 落地 |
| 规范矩阵 | 父子通信、旧入口兼容、无状态迁移 | 落地 |
| 引导矩阵 | `system.entry.backend_process` 白箱节点 | 扩展 |
| 模块树 | `system.entry`、`system.entry.backend_process` | 扩展 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、附录 E.2 |
| 模块树节点 | `system.entry`、`system.entry.backend_process` |
| 真实文件 | `src/system/mod.rs`、`src/system/entry/mod.rs`、`src/system/entry/backend_process.rs`、`src/lib.rs`、`src/main.rs` |
| public 方法 | `run_server`、兼容入口 `quantpilot::run_server` |
| 测试/门禁 | `cargo check -p quantpilot`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 兼容桥

```text
src/main.rs
  -> quantpilot::run_server()
  -> pub use system::entry::backend_process::run_server
  -> system.entry.backend_process::run_server()
  -> run_api_server()
  -> backend.interface_boundary
  -> build_app_router()
```

旧 public 入口不变，调用方不需要改 import。`system.entry.backend_process` 只拥有进程启动与 CLI 分发，不拥有后端 API 语义。

---

## 等价证据

| 维度 | 结果 |
| --- | --- |
| public 入口 | `quantpilot::run_server` 仍存在 |
| 二进制入口 | `src/main.rs` 未改 |
| API 边界 | `run_api_server`、`build_app_router` 未迁移 |
| 状态所有权 | `new_app_state` 未迁移 |
| 编译验证 | `cargo check -p quantpilot` 通过 |

---

## 暂停线

后续如果要继续拆 `system.entry.backend_process` 内部，必须先讨论以下决策:

1. 是否把 `run_api_server` 从 `src/lib.rs` 迁入 system 模块。
2. 如果迁移 `run_api_server`，如何处理其对 `AppState`、后台任务、CORS、安全中间件和持久化预热的访问。
3. 是否允许把 `new_app_state` 继续留在 `app_runtime_helpers`，或另开 `system.entry.state_bootstrap`。

未决策前，不继续搬迁更深层启动实现。

---

## 禁止事项

1. 不绕过 `backend.interface_boundary` 或 `build_app_router`。
2. 不迁移 runtime state、executor state 或 AppState 所有权。
3. 不删除旧 public 入口。
4. 不把本次试水描述为 system 全量抽离完成。
5. 不主动提出发布版本过渡或横向连接。
