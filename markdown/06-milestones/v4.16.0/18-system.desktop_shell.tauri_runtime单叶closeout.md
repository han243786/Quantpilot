# v4.16.0 system.desktop_shell.tauri_runtime 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`17-system.desktop_shell.tauri_runtime-readiness等价检查.md`。
> 执行档位: 重型。
> 判定: S3 `system.desktop_shell.tauri_runtime` 完成单叶白箱 closeout；Tauri runtime 入口、readiness wait、窗口生命周期和关闭路径已验证；不改代码，不进入整理或重构。

---

## 目标

本文件完成 S3 `system.desktop_shell.tauri_runtime` 的 closeout，补齐上一份 readiness 等价检查中保留的桌面启动 smoke 和窗口生命周期核查。

本批次只登记与验证现有行为:

1. 后端 3000 readiness 仍先于 Tauri Builder 启动。
2. Tauri dev 能拉起前端 5173 和桌面主窗口。
3. 主窗口标题、进程、端口 steady-state 符合现有语义。
4. `CloseMainWindow` 能让 Tauri 窗口进程退出。
5. 本批次不修改 `src-tauri/src/main.rs`、Tauri config、capability 或启动脚本。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S3 完成判定 | 落地 |
| 规范矩阵 | `system.desktop_shell.tauri_runtime` owner、public/内部实现分类、生命周期证据 | 落地 |
| 引导矩阵 | 全量树、模块树、真实文件、桌面启动 smoke 坐标 | 扩展 |
| 模块树 | `system.desktop_shell.tauri_runtime` | 完成 S3 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.desktop_shell.tauri_runtime` |
| 真实文件 | `src-tauri/src/main.rs`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` |
| public 方法 | Tauri `main` |
| 关键内部实现 | `wait_for_backend`、`TcpStream::connect_timeout`、`BACKEND_PORT`、`MAX_WAIT_SECS`、`tauri::Builder::default`、`tauri_plugin_shell::init`、debug devtools setup、`tauri::generate_context` |
| 测试/门禁 | `cargo build --bin quantpilot`、`cargo tauri dev --no-watch`、`cargo check -p quantpilot-tauri`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 桌面启动 smoke 证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| 后端构建 | 通过 | `cargo build --bin quantpilot` 成功 |
| 后端 readiness | 通过 | `target\debug\quantpilot.exe` 启动后，`http://127.0.0.1:3000/api/health` 返回 200 |
| Tauri dev 启动 | 通过 | 在 `src-tauri` 执行 `cargo tauri dev --no-watch` |
| 前端 dev server | 通过 | 5173 端口 ready |
| Tauri readiness wait | 通过 | Tauri 日志出现 `[tauri] Waiting for backend (port 3000)...` 和 `[tauri] Backend ready after 0s` |
| 主窗口出现 | 通过 | 新进程 `quantpilot-tauri.exe` 出现可见窗口，标题为 `QuantPilot - 量化策略平台` |
| steady-state | 通过 | 主窗口存活期间，3000 和 5173 均保持可连接 |
| 关闭路径 | 通过 | `CloseMainWindow` 返回 true，Tauri 窗口进程在 20 秒内退出 |
| 清理状态 | 通过 | smoke 后无运行中的 `quantpilot`、`quantpilot-tauri`、`cargo` 或 `node` smoke 残留，仅有端口 `TimeWait` |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | Tauri `main` 是 S3 唯一 public 入口 |
| 兼容入口 | 完成 | 桌面应用启动入口、窗口标题、窗口生命周期不变 |
| readiness wait | 完成 | 3000 端口、30 秒最大等待、成功/超时路径已在 `17-system.desktop_shell.tauri_runtime-readiness等价检查.md` 登记 |
| runtime wiring | 完成 | shell plugin、debug devtools setup、`generate_context` 保持现状 |
| 外部边界 | 完成 | 不拥有后端 API、AppState、前端路由、capability 真源或发布态横向连接 |
| 等价证据 | 完成 | 编译检查、readiness 核查、桌面 smoke 和关闭路径均已有证据 |
| 继续细分 | 停止 | `wait_for_backend` 是 Tauri `main` 内部 helper，没有独立 public 入口或独立 owner |

---

## 父子通信规则

`system.desktop_shell.tauri_runtime` 只能经 `system.desktop_shell` 承载桌面 runtime、readiness wait 和窗口生命周期。它不得直接横向调用 `backend.interface_boundary`、`frontend.*`、runtime state、AppState 或 capability 真源。

后续若改变以下任一内容，必须重新提案并回到 S3:

1. 后端 readiness 目标、端口、等待时长或超时策略。
2. Tauri `main` 启动顺序。
3. shell plugin、Tauri command 或 capability 权限。
4. 窗口创建、关闭、标题、decorations、devtools 或 lifecycle 语义。

---

## 不继续细分理由

S3 当前只有一个真实入口 `src-tauri/src/main.rs`，内部 helper 都服务 Tauri `main`:

| 候选子叶 | 不继续拆的原因 |
| --- | --- |
| readiness wait | 无独立 public 入口；只能作为 Tauri main 的启动前置 |
| shell plugin 初始化 | 无独立 owner；权限真源在 S4 capability/config |
| debug devtools setup | debug-only 内部 setup；无独立验证价值 |
| `generate_context` | Tauri build/runtime glue；不应脱离 S3 |

继续拆会制造文档碎片和横向通信压力，因此 S3 停止细分，后续只按变更触发维护。

---

## 验收标准

1. S3 readiness wait、Tauri runtime、主窗口出现、steady-state 和关闭路径均已有证据。
2. `system.desktop_shell.tauri_runtime` 模块树节点标记为单叶 closeout 完成。
3. v4.16 里程碑索引、落地记录、全量树和治理门禁能发现本 closeout 缺失。
4. 本批次没有修改 `src-tauri/src/main.rs`、Tauri config、capability 或启动脚本。
5. S3 不继续细分，不进入整理或重构。
