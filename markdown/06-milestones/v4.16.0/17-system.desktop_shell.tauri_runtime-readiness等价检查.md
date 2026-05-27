# v4.16.0 system.desktop_shell.tauri_runtime readiness 等价检查

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`14-system.entry.launch_scripts单叶closeout.md`、`15-system.desktop_shell.tauri_config单叶closeout.md`。
> 执行档位: 重型。
> 判定: S3 `system.desktop_shell.tauri_runtime` 的后端 readiness wait 等价检查完成；不改代码，不宣告 S3 完整 closeout，不进入整理或重构。

---

## 目标

本文件只回答一个问题: `src-tauri/src/main.rs` 中的 Tauri runtime readiness wait 是否能作为 `system.desktop_shell.tauri_runtime` 的稳定等价基线。

本批次不做以下动作:

1. 不修改 `src-tauri/src/main.rs`。
2. 不迁移 Tauri runtime 入口。
3. 不新增 Tauri command。
4. 不改变后端启动关系、窗口生命周期、CSP 或 capability 权限。
5. 不把 readiness 等价检查解释为 S3 完整 closeout。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 readiness 检查、S3 后续 closeout 准入 | 落地 |
| 规范矩阵 | `system.desktop_shell.tauri_runtime` owner、public/内部实现分类、父子通信边界 | 扩展 |
| 引导矩阵 | 全量树、模块树、真实文件、Tauri runtime 门禁坐标 | 扩展 |
| 模块树 | `system.desktop_shell.tauri_runtime` | 补白箱 readiness 节点 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.desktop_shell.tauri_runtime` |
| 真实文件 | `src-tauri/src/main.rs`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` |
| public 方法 | Tauri `main` |
| 关键内部实现 | `wait_for_backend`、`TcpStream::connect_timeout`、`BACKEND_PORT`、`MAX_WAIT_SECS`、`tauri::Builder::default`、`tauri_plugin_shell::init` |
| 测试/门禁 | `cargo check -p quantpilot-tauri`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、readiness wait 人工核查 |

---

## 白箱边界

| 项 | 当前事实 | 等价要求 |
| --- | --- | --- |
| 父模块 | `system.desktop_shell` | S3 只服务桌面壳 runtime，不拥有后端 API 或前端路由 |
| 真实入口 | `src-tauri/src/main.rs` | 桌面应用启动入口仍由 Tauri `main` 承载 |
| readiness 目标 | `127.0.0.1:3000` | 与 S1 启动脚本等待的后端端口保持一致 |
| readiness 方式 | `TcpStream::connect_timeout` | 只检查 TCP 可连接，不读取业务 API 响应 |
| 单次连接超时 | 1 秒 | 不扩大启动阻塞时间 |
| 最大等待 | `MAX_WAIT_SECS = 30` | 超时后记录日志并继续进入 Tauri runtime |
| 成功路径 | 连接成功后返回 | 先确认后端端口，再进入 `tauri::Builder::default` |
| 超时路径 | 30 秒后 proceeding anyway | 不因为后端未 ready 永久阻断桌面壳 |
| Tauri runtime | shell plugin、debug devtools、`generate_context` | 不新增 command，不改变窗口生命周期 |

---

## 等价检查结果

| 检查项 | 结果 | 说明 |
| --- | :--: | --- |
| `BACKEND_PORT` | 通过 | 固定为 `3000`，与启动脚本 readiness 目标一致 |
| `wait_for_backend` 调用位置 | 通过 | 在 `tauri::Builder::default` 之前执行 |
| 成功路径 | 通过 | TCP 连接成功后立即返回，不改后端状态 |
| 超时路径 | 通过 | 超过 30 秒后继续启动，保留桌面壳容错语义 |
| Tauri shell plugin | 通过 | 仍只初始化 `tauri_plugin_shell::init` |
| debug devtools | 通过 | 仍只在 debug 下打开第一个 webview devtools |
| capability/CSP | 通过 | 本批次未修改 `src-tauri/tauri.conf.json` 或 capability 文件 |
| 编译门禁 | 通过 | `cargo check -p quantpilot-tauri` 已通过 |

---

## 父子通信规则

`system.desktop_shell.tauri_runtime` 只能通过 `system.desktop_shell` 进入后端 readiness 检查和 Tauri runtime 启动，不得直接拥有 `backend.interface_boundary`、AppState、前端 route、业务 capability 真源或 release 横向连接。

如果后续要改变 readiness 目标、等待策略、后端启动关系或 Tauri command wiring，必须回到 S3 单叶提案，不得混入 S1 启动脚本、S4 配置、S6 manifest 或 S7 build scripts 批次。

---

## 后续 closeout 准入

S3 还不能因为本文件直接宣告完整 closeout。进入 S3 完整 closeout 前，至少需要补齐以下确认:

1. 桌面启动 smoke 或人工窗口生命周期核查。
2. Tauri `main`、shell plugin、debug devtools 和 `generate_context` 的白箱边界复核。
3. 确认没有新增 Tauri command、capability 权限或 CSP 变更。
4. 确认不进入发布版本过渡，不引入子模块横向连接。

---

## 验收标准

1. S3 readiness wait 的端口、等待时长、成功路径、超时路径和启动顺序已登记。
2. `system.desktop_shell.tauri_runtime` 进入模块树白箱节点。
3. v4.16 里程碑索引、落地记录、全量树和治理门禁能发现本文件缺失。
4. `cargo check -p quantpilot-tauri` 通过。
5. 本批次只完成 readiness 等价检查，不宣告 S3 完整 closeout。
