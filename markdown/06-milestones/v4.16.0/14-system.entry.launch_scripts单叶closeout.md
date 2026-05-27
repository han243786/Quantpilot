# v4.16.0 system.entry.launch_scripts 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`13-递归模块化全局根流程.md`。
> 执行档位: 重型。
> 判定: S1 `system.entry.launch_scripts` 完成单叶白箱 closeout；不改脚本行为，不进入整理或重构。

---

## 目标

本批次只确认 `system.entry.launch_scripts` 作为 system 第一个低风险叶子，是否能保持原启动功能等价。

本批次不修改 `start.bat` 或 `start.ps1`，只记录其真实职责、入口、内部步骤、保留外部边界、等价证据和后续停止条件。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 单叶 closeout、递归模块化 R2/R3 | 落地 |
| 规范矩阵 | 启动脚本 owner、旧入口兼容、端口/进程语义冻结 | 落地 |
| 引导矩阵 | `system.entry.launch_scripts` 真实文件、入口、门禁 | 扩展 |
| 模块树 | `system.entry.launch_scripts` | 完成 S1 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.entry.launch_scripts` |
| 真实文件 | `start.bat`、`start.ps1` |
| public 方法 | Windows CMD 启动入口 `start.bat`、PowerShell 启动入口 `start.ps1` |
| 关键内部实现 | 设置 `QUANTPILOT_DEV=true`、停止旧进程、构建后端、启动 `target\debug\quantpilot.exe`、等待 3000 端口、执行 `cargo tauri dev` |
| 测试/门禁 | `tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、脚本静态等价核查 |

---

## 真实行为基线

| 步骤 | `start.bat` | `start.ps1` | 等价口径 |
| --- | --- | --- | --- |
| 工作目录 | `cd /d "%~dp0"` | `Set-Location $root` | 都回到仓库根目录 |
| 编码/错误 | `chcp 65001`、`setlocal enabledelayedexpansion` | UTF-8 native、`$ErrorActionPreference = "Stop"` | 平台差异保留，不互相强行统一 |
| 开发模式 | `set QUANTPILOT_DEV=true` | `$env:QUANTPILOT_DEV = "true"` | 环境变量语义一致 |
| 停旧进程 | `taskkill` 关闭 `quantpilot.exe`、`quantpilot-tauri.exe` | `Stop-Process` 关闭 `quantpilot`、`quantpilot-tauri` | 旧进程清理语义一致 |
| 端口清理 | 清理 5173 LISTENING 进程 | 清理 5173 owner 进程 | 前端 dev port 清理语义一致 |
| 后端构建 | `cargo build --bin quantpilot` | `cargo build --bin quantpilot` | 构建命令一致 |
| 后端启动 | `start ... target\debug\quantpilot.exe` | `Start-Process target\debug\quantpilot.exe` | 启动目标一致 |
| readiness | `netstat` 检查 3000，最多 30 次 | TCP connect 检查 3000，最多 30 秒 | readiness 目标一致 |
| 桌面启动 | `cd src-tauri` 后 `cargo tauri dev` | `Set-Location "$root\src-tauri"` 后 `cargo tauri dev` | Tauri dev 入口一致 |

---

## public/内部实现分类

| 分类 | 内容 | 处理 |
| --- | --- | --- |
| public 入口 | `start.bat`、`start.ps1` | 文件名、调用方式、用户启动入口不变 |
| 兼容 public 入口 | 原脚本路径和默认命令行行为 | 抽离阶段不得删除或重命名 |
| 关键内部实现 | 环境变量、旧进程清理、5173 清理、后端构建、后端启动、3000 readiness、Tauri dev 启动 | 只登记，不改逻辑 |
| 保留外部边界 | 后端 API、`system.entry.backend_process`、Tauri runtime、前端 dev server、runtime state、capability 真源 | S1 不拥有这些 owner |

---

## 等价证据

| 证据 | 结果 |
| --- | --- |
| 文件存在 | `start.bat`、`start.ps1` 均存在 |
| 默认开发模式 | 两者均设置 `QUANTPILOT_DEV=true` |
| 后端构建入口 | 两者均执行 `cargo build --bin quantpilot` |
| 后端启动目标 | 两者均启动 `target\debug\quantpilot.exe` |
| readiness 目标 | 两者均等待 3000 端口 |
| 桌面入口 | 两者均进入 `src-tauri` 并执行 `cargo tauri dev` |
| 治理门禁 | `tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

本批次未执行实际桌面启动 smoke，避免启动交互式 Tauri 进程。后续如要改脚本语义，必须单独做启动 smoke 或人工验收。

---

## closeout 判定

| 项 | 判定 |
| --- | --- |
| 单叶抽离状态 | S1 完成白箱 closeout |
| 是否改代码/脚本 | 否 |
| 是否改变用户入口 | 否 |
| 是否继续细分 | 暂不值得继续细分 |
| 是否进入整理/重构 | 否 |
| 后续动作 | 仅在端口、进程拓扑、启动顺序或脚本命令变化时重新打开 |

---

## 不继续细分的理由

`system.entry.launch_scripts` 目前只有两个平台启动入口，内部差异主要来自 CMD 与 PowerShell 平台差异。继续拆 L3 只能得到“环境变量设置”“进程清理”“后端构建”“桌面启动”等 helper 级概念，但没有独立 owner、独立 public 入口或独立验证证据。

因此 S1 作为叶子停止细分。

---

## 禁止事项

- 不改默认端口、启动顺序、脚本文件名或用户调用方式。
- 不把脚本入口当作后端进程 owner。
- 不把脚本进程清理扩展到未登记端口或未登记进程。
- 不把 S1 closeout 描述成 system 全部整理完成。
- 不因脚本差异强行统一 CMD 与 PowerShell 平台实现。

---

## 验收标准

1. S1 的 public 入口和兼容入口已登记。
2. S1 的关键内部实现和保留外部边界已登记。
3. S1 的等价证据已登记。
4. S1 明确不继续细分。
5. 后续 system 推进可转向 S4、S10 或 S3 readiness 等价检查。
