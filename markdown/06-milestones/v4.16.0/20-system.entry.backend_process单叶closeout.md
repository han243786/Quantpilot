# v4.16.0 system.entry.backend_process 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `10-system抽离完成记录.md`、`11-system抽离经验回填.md`、`12-system十叶模块等价基线.md`。
> 执行档位: 重型。
> 判定: S2 `system.entry.backend_process` 完成正式单叶白箱 closeout；既有代码抽离保持等价，不扩大到后端 API、AppState 或 handler owner。

---

## 目标

本文件把此前已经完成的 `system.entry.backend_process` 试水抽离收束为单叶 closeout。

本批次只确认启动进程边界:

1. `run_server` 仍是后端进程 public 启动入口。
2. `quantpilot::run_server` 兼容入口仍保留。
3. `run_api_server` 只作为启动期内部实现，不变成 API route owner。
4. `new_app_state`、`build_app_router`、handler、response schema、runtime state 和 executor state 均不迁移。
5. 本批次不改代码、不改 CLI、不改端口、不改锁顺序。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S2 完成判定 | 落地 |
| 规范矩阵 | 启动进程 owner、兼容 public 入口、未迁移边界 | 收口 |
| 引导矩阵 | 全量树、模块树、真实文件、后端启动门禁坐标 | 扩展 |
| 模块树 | `system.entry.backend_process` | 完成 S2 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.entry.backend_process` |
| 真实文件 | `src/system/entry/backend_process.rs`、`src/system/entry/mod.rs`、`src/system/mod.rs`、`src/lib.rs`、`src/main.rs`、`src/app_runtime_helpers.rs`、`src/app_router.rs` |
| public 方法 | `run_server`、`quantpilot::run_server` |
| 关键内部实现 | `run_api_server`、`initialize_process_environment`、`dispatch_process_command`、启动目录创建、CORS、安全头、JSON rejection middleware、后台观察任务、优雅关闭和关闭刷盘 |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 等价验证证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| crate 兼容入口 | 通过 | `src/lib.rs` 仍 `pub use system::entry::backend_process::run_server` |
| binary 入口 | 通过 | `src/main.rs` 仍调用 `quantpilot::run_server().await` |
| public 启动入口 | 通过 | `src/system/entry/backend_process.rs` 提供 `pub async fn run_server()` |
| API server 内部实现 | 通过 | `run_api_server` 仍为启动期内部实现，不是 public API |
| 默认无 CLI 参数行为 | 通过 | `cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided` 成功 |
| 保留外部边界 | 通过 | `new_app_state` 仍在 `src/app_runtime_helpers.rs`，`build_app_router` 仍在 `src/app_router.rs` |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | `run_server` 和旧 crate root 兼容入口已登记 |
| 兼容入口 | 完成 | `quantpilot::run_server` 调用方不需要改 import |
| 启动期内部实现 | 完成 | `run_api_server`、CLI 分发、环境初始化和 server 启动链已登记 |
| 外部边界 | 完成 | 不拥有 router、handler、response schema、AppState 字段或 runtime/executor state |
| 等价证据 | 完成 | 默认 server 行为测试和文件边界核查已有证据 |
| 继续细分 | 停止 | 再拆会进入启动 helper 级细节，缺少独立 public 入口和独立 owner |

---

## 父子通信规则

`system.entry.backend_process` 只能通过 `system.entry` 暴露后端进程启动能力，并通过 `run_api_server -> backend.interface_boundary -> build_app_router` 进入后端接口边界。

它不得直接横向拥有或修改以下边界:

1. `backend.interface_boundary` 的 route owner。
2. `src/app_runtime_helpers.rs` 的 `new_app_state` 状态工厂。
3. handler、response schema、OpenAPI、runtime state、executor state。
4. Tauri runtime、启动脚本、CI/release、container proxy。

---

## 不继续细分理由

| 候选子叶 | 不继续拆的原因 |
| --- | --- |
| `initialize_process_environment` | 只是启动前环境初始化 helper，没有独立 public 入口 |
| `dispatch_process_command` | 只服务 `run_server` CLI 分发，不拥有 CLI 语义真源 |
| `run_api_server` | 是启动期内部实现，继续拆会撞上 AppState 和 router owner |
| CORS/security/rejection middleware | 属于启动装配细节，独立拆分会制造横向依赖 |

因此 S2 在当前抽离阶段停止细分，后续只有在启动职责变化时重新打开。

---

## 禁止事项

- 不把 `system.entry.backend_process` 解释为 API route owner。
- 不把 `run_api_server` 升级成 public API。
- 不迁移 `new_app_state`、`build_app_router`、handler、response schema 或状态锁。
- 不借 S2 closeout 删除旧 crate root 兼容入口。
- 不把 system 入口抽离描述成 system 全量整理或重构完成。

---

## 验收标准

1. S2 的 public 入口、兼容入口、关键内部实现和保留外部边界已登记。
2. 默认无 CLI 参数行为测试通过。
3. `system.entry.backend_process` 模块树节点标记为单叶 closeout 完成。
4. v4.16 里程碑索引、落地记录、全量树和治理门禁能发现本 closeout 缺失。
5. 本批次不改代码，不继续细分，不进入整理或重构。
