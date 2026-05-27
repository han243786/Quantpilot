# v4.16.0 system.build_delivery.desktop_build_scripts 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`18-system.desktop_shell.tauri_runtime单叶closeout.md`。
> 执行档位: 重型。
> 判定: S7 `system.build_delivery.desktop_build_scripts` 完成单叶白箱 closeout；desktop build/dev scripts 已验证，不改脚本，不进入整理或重构。

---

## 目标

本文件完成 S7 `system.build_delivery.desktop_build_scripts` 的 closeout，确认 Tauri build/dev 脚本作为独立交付叶子的当前行为。

本批次只登记与验证现有行为:

1. `src-tauri/build.rs` 仍只调用 `tauri_build::build()`。
2. `src-tauri/build.bat` 仍切到 `frontend` 后执行 `npm run build`。
3. `src-tauri/dev.bat` 仍切到 `frontend` 后执行 `npm run dev -- --strictPort`。
4. `src-tauri/tauri.conf.json` 仍通过 `beforeBuildCommand` / `beforeDevCommand` 引用这两个脚本。
5. 本批次不修改 build/dev scripts，不改变产物路径、端口、Tauri bundling 或根启动脚本语义。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S7 完成判定 | 落地 |
| 规范矩阵 | `system.build_delivery.desktop_build_scripts` owner、public/内部实现分类、交付边界 | 扩展 |
| 引导矩阵 | 全量树、模块树、真实文件、desktop build/dev 门禁坐标 | 扩展 |
| 模块树 | `system.build_delivery.desktop_build_scripts` | 完成 S7 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.build_delivery.desktop_build_scripts` |
| 真实文件 | `src-tauri/build.rs`、`src-tauri/build.bat`、`src-tauri/dev.bat`、`src-tauri/tauri.conf.json`、`frontend/package.json` |
| public 方法 | `src-tauri/build.rs` build script、`src-tauri/build.bat`、`src-tauri/dev.bat`、Tauri `beforeBuildCommand`、Tauri `beforeDevCommand` |
| 关键内部实现 | `tauri_build::build()`、`npm run build`、`npm run dev -- --strictPort`、5173 dev server |
| 测试/门禁 | `cargo check -p quantpilot-tauri`、`cmd /c src-tauri\build.bat`、受控 `src-tauri\dev.bat` 5173 smoke、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 等价验证证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| Rust build script | 通过 | `cargo check -p quantpilot-tauri` 成功，`src-tauri/build.rs` 仍只调用 `tauri_build::build()` |
| frontend build script | 通过 | `cmd /c src-tauri\build.bat` 成功，等价执行 `npm run build` |
| frontend build output | 通过 | Vite production build 完成，991 modules transformed |
| dev script 启动 | 通过 | 受控运行 `cmd /c src-tauri\dev.bat` |
| dev server 端口 | 通过 | 5173 端口 ready，`http://127.0.0.1:5173` 返回 200 |
| strictPort 语义 | 通过 | stdout 显示 `vite --strictPort` |
| 清理状态 | 通过 | smoke 结束后无本轮启动的 `node` / `npm` / dev server 残留 |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | `build.rs`、`build.bat`、`dev.bat` 和 Tauri before-command 配置已登记 |
| 兼容入口 | 完成 | Tauri build/dev 命令入口不变 |
| build 行为 | 完成 | 前端 production build 仍由 `npm run build` 执行 |
| dev 行为 | 完成 | 前端 dev server 仍由 `npm run dev -- --strictPort` 执行 |
| 外部边界 | 完成 | 不拥有根启动脚本、Tauri runtime、CI/release、容器代理或业务构建产物语义 |
| 等价证据 | 完成 | Tauri check、build.bat build、dev.bat 5173 smoke 均已有证据 |
| 继续细分 | 停止 | 当前只有 3 个小脚本，拆成 L3 会变成命令行字段级文档 |

---

## 父子通信规则

`system.build_delivery.desktop_build_scripts` 只能经 `system.build_delivery` 为 Tauri build/dev 流程提供脚本入口。它不得直接拥有 `system.entry.launch_scripts`、`system.desktop_shell.tauri_runtime`、`system.desktop_shell.tauri_config`、CI/release workflow、container proxy 或业务 runtime 状态。

后续若改变以下任一内容，必须重新提案并回到 S7:

1. `beforeBuildCommand` 或 `beforeDevCommand`。
2. `npm run build`、`npm run dev`、`--strictPort` 或 5173 dev server 语义。
3. `frontend/dist` 产物路径或 Tauri `frontendDist`。
4. `tauri_build::build()` 的 build.rs 行为。
5. 根启动脚本与 desktop build/dev scripts 的职责边界。

---

## 不继续细分理由

| 候选子叶 | 不继续拆的原因 |
| --- | --- |
| `src-tauri/build.rs` | 只有 `tauri_build::build()`，无独立 owner 或独立策略 |
| `src-tauri/build.bat` | 只有 `npm run build` wrapper，验证证据已足够 |
| `src-tauri/dev.bat` | 只有 `npm run dev -- --strictPort` wrapper，验证证据已足够 |
| `beforeBuildCommand` / `beforeDevCommand` | 配置引用已由 S4 登记，S7 只拥有脚本行为 |

继续拆会制造文档碎片，且无法产生更细的独立 public 入口，因此 S7 停止细分。

---

## 验收标准

1. S7 build/dev scripts 的真实文件、public 入口、关键内部实现和父级通信规则已登记。
2. `cargo check -p quantpilot-tauri`、`cmd /c src-tauri\build.bat` 和受控 `src-tauri\dev.bat` 5173 smoke 已通过。
3. `system.build_delivery.desktop_build_scripts` 模块树节点标记为单叶 closeout 完成。
4. v4.16 里程碑索引、落地记录、全量树和治理门禁能发现本 closeout 缺失。
5. 本批次没有修改 build/dev scripts，不继续细分，不进入整理或重构。
