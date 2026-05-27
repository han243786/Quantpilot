# v4.16.0 system.desktop_shell.tauri_config 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`13-递归模块化全局根流程.md`。
> 执行档位: 重型。
> 判定: S4 `system.desktop_shell.tauri_config` 完成单叶白箱 closeout；不改 Tauri 配置、CSP、窗口或权限语义，不进入整理或重构。

---

## 目标

本批次确认 `system.desktop_shell.tauri_config` 作为桌面壳配置叶子，是否能稳定承载现有 Tauri 配置、CSP、窗口配置、bundle 配置和 capability allowlist。

本批次不修改 `src-tauri/tauri.conf.json` 或 `src-tauri/capabilities/default.json`，只记录真实职责、入口、内部配置、保留外部边界、等价证据和停止条件。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 单叶 closeout、递归模块化 R2/R3 | 落地 |
| 规范矩阵 | Tauri 配置 owner、CSP/权限冻结、窗口配置兼容 | 落地 |
| 引导矩阵 | `system.desktop_shell.tauri_config` 真实文件、入口、门禁 | 扩展 |
| 模块树 | `system.desktop_shell.tauri_config` | 完成 S4 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.desktop_shell.tauri_config` |
| 真实文件 | `src-tauri/tauri.conf.json`、`src-tauri/capabilities/default.json` |
| public 方法 | Tauri config、CSP、capability allowlist、窗口和 bundle 配置 |
| 关键内部实现 | `productName`、`version`、`identifier`、`devUrl`、`beforeDevCommand`、`beforeBuildCommand`、window config、CSP、bundle target、NSIS install mode、capability permissions |
| 测试/门禁 | JSON parse、`cargo check -p quantpilot-tauri`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 真实行为基线

| 维度 | 当前值 | 等价口径 |
| --- | --- | --- |
| 产品名 | `QuantPilot` | 不改应用产品标识 |
| 版本 | `4.7.0` | 不在抽离批次中改版本语义 |
| identifier | `com.quantpilot.app` | 不改桌面应用标识 |
| 前端 dev URL | `http://localhost:5173` | 与启动脚本 5173 清理保持一致 |
| before dev | `src-tauri\dev.bat` | 不绕过 desktop build/dev scripts 叶子 |
| before build | `src-tauri\build.bat` | 不绕过 desktop build/dev scripts 叶子 |
| 主窗口 | 标题 `QuantPilot - 量化策略平台`，`1400x900`，最小 `960x600`，无 decorations | 不改窗口语义 |
| CSP | 允许本地 5173 dev、127.0.0.1:3000 API、Vite websocket、`https:` connect | 不放宽或重写 CSP |
| bundle | `active=true`、`targets=all`、icons、NSIS currentUser | 不改打包目标 |
| capability | `core:default`、`shell:allow-open`、窗口控制权限 | 不新增权限 |

---

## public/内部实现分类

| 分类 | 内容 | 处理 |
| --- | --- | --- |
| public 入口 | `src-tauri/tauri.conf.json`、`src-tauri/capabilities/default.json` | 文件路径、Tauri 消费方式和配置语义不变 |
| 兼容 public 入口 | Tauri CLI 默认读取配置和 capability 文件 | 抽离阶段不得删除、重命名或搬迁 |
| 关键内部实现 | app metadata、build commands、window config、CSP、bundle config、capability permissions | 只登记，不改逻辑 |
| 保留外部边界 | 前端 capability projection、后端 API 权限语义、业务 supported/unsupported 声明、runtime state | S4 不拥有这些 owner |

---

## 等价证据

| 证据 | 结果 |
| --- | --- |
| `src-tauri/tauri.conf.json` JSON parse | 通过 |
| `src-tauri/capabilities/default.json` JSON parse | 通过 |
| public config 文件存在 | 两个文件均存在 |
| dev URL 与脚本端口 | `5173` 与 S1 启动脚本前端端口清理一致 |
| 后端 API connect | CSP 保留 `http://127.0.0.1:3000` |
| capability allowlist | 未新增权限 |
| 治理门禁 | `tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

本批次未执行 Tauri 桌面启动 smoke，避免启动交互式桌面进程。后续如要改窗口、CSP、权限或 bundle 行为，必须单独做 Tauri 启动 smoke 或人工验收。

---

## closeout 判定

| 项 | 判定 |
| --- | --- |
| 单叶抽离状态 | S4 完成白箱 closeout |
| 是否改配置 | 否 |
| 是否改变桌面入口 | 否 |
| 是否继续细分 | 暂不值得继续细分 |
| 是否进入整理/重构 | 否 |
| 后续动作 | 仅在 CSP、窗口、权限、bundle 或 Tauri build command 变化时重新打开 |

---

## 不继续细分的理由

`system.desktop_shell.tauri_config` 当前只有两个 Tauri 配置文件。CSP、窗口、bundle 和 capability allowlist 虽然语义重要，但都没有独立 public 入口；继续拆 L3 会变成配置字段级文档，缺少独立 owner 和独立验证证据。

因此 S4 作为叶子停止细分。

---

## 禁止事项

- 不放宽 CSP。
- 不新增 Tauri capability permission。
- 不改变窗口默认尺寸、最小尺寸、decorations 或 app identifier。
- 不把 Tauri config 当作前端 capability 真源。
- 不把 S4 closeout 描述成 desktop shell 全部整理完成。

---

## 验收标准

1. S4 的 public 配置入口和兼容入口已登记。
2. S4 的关键内部配置和保留外部边界已登记。
3. S4 的 JSON parse 等价证据已登记。
4. S4 明确不继续细分。
5. 后续 system 推进可转向 S7 desktop build scripts。
