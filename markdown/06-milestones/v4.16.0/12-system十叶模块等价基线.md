# v4.16.0 system 十叶模块等价基线

> 版本类型: MINOR architecture / governance。
> 基准: `08-system大模块分层统计.md`、`10-system抽离完成记录.md`、`11-system抽离经验回填.md`。
> 执行档位: 重型。
> 判定: 先铺好 `system` 10 个叶子模块的功能等价基线；不继续细分叶子，不进入整理和重构。

---

## 目标

本文件回答: 当前 `system` 10 个叶子模块是否能在抽离视角下证明“和之前一样工作”。

本阶段不继续往 L3 拆，不做目录美化，不删除旧入口。只为每个叶子建立以下基线:

1. 当前 owner 和真实文件。
2. public 入口、兼容入口、关键内部实现和保留外部边界。
3. 等价验证方式。
4. 是否可继续抽离。
5. 必须暂停讨论的决策点。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 后续抽离准入、决策暂停、closeout 口径 | 落地 |
| 规范矩阵 | 10 叶 owner 归属、public/内部实现分类、未迁移边界 | 落地 |
| 引导矩阵 | 全量树、模块树、真实文件、测试/门禁坐标 | 扩展 |
| 模块树 | `root.system` 10 个叶子模块 | 补基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.entry.launch_scripts`、`system.entry.backend_process`、`system.desktop_shell.tauri_runtime`、`system.desktop_shell.tauri_config`、`system.desktop_shell.assets_schema`、`system.build_delivery.workspace_manifest`、`system.build_delivery.desktop_build_scripts`、`system.build_delivery.container_proxy`、`system.build_delivery.ci_release`、`system.runtime_profile.config_examples` |
| 真实文件 | `start.bat`、`start.ps1`、`src/system/entry/backend_process.rs`、`src/main.rs`、`src/lib.rs`、`src/app_runtime_helpers.rs`、`src-tauri/src/main.rs`、`src-tauri/tauri.conf.json`、`src-tauri/capabilities/default.json`、`Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml`、`src-tauri/build.rs`、`src-tauri/build.bat`、`src-tauri/dev.bat`、`Dockerfile`、`docker-compose.yml`、`nginx.conf`、`.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`.env.example`、`config/runtime_protocol.example.yaml`、`config/strategy_ir.v0.schema.json`、`config/strategy_ir.v0.example.json` |
| public 方法 | `run_server`、`quantpilot::run_server`、启动脚本入口、Tauri main、workspace/package manifest、CI workflow、配置样例入口 |
| 关键内部实现 | `run_api_server`、Tauri readiness wait、desktop build script、container proxy config、release packaging steps |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`、`cargo check -p quantpilot-tauri`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 总体状态

| 叶子 | 当前状态 | 下一步建议 |
| --- | --- | --- |
| S1 `system.entry.launch_scripts` | 已完成单叶 closeout | `14-system.entry.launch_scripts单叶closeout.md` 已确认脚本入口等价，不继续细分 |
| S2 `system.entry.backend_process` | 已抽离完成 | 只维护，不继续扩大 owner |
| S3 `system.desktop_shell.tauri_runtime` | readiness 等价检查已完成，代码抽离仍需暂停 | `17-system.desktop_shell.tauri_runtime-readiness等价检查.md` 已确认 3000 wait 等价；S3 完整 closeout 前需补桌面启动 smoke/窗口生命周期核查 |
| S4 `system.desktop_shell.tauri_config` | 已完成单叶 closeout | `15-system.desktop_shell.tauri_config单叶closeout.md` 已确认 Tauri 配置等价，不继续细分 |
| S5 `system.desktop_shell.assets_schema` | 只需登记，不主动抽离 | 作为资产叶子管理 |
| S6 `system.build_delivery.workspace_manifest` | 需要决策暂停 | 依赖/workspace 影响大，不主动抽 |
| S7 `system.build_delivery.desktop_build_scripts` | 可低风险登记 | 不与启动脚本混批 |
| S8 `system.build_delivery.container_proxy` | 按变更触发 | 非桌面默认路径，不主动抽 |
| S9 `system.build_delivery.ci_release` | 需要决策暂停 | 与测试汰换/发布流程耦合 |
| S10 `system.runtime_profile.config_examples` | 已完成单叶 closeout | `16-system.runtime_profile.config_examples单叶closeout.md` 已确认配置样例等价，不继续细分 |

---

## 十叶等价基线

### S1 `system.entry.launch_scripts`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.entry` |
| 真实文件 | `start.bat`、`start.ps1` |
| public 入口 | 启动脚本入口 |
| 兼容入口 | 原脚本文件名和命令行调用方式不变 |
| 关键内部实现 | 环境准备、端口/进程启动编排、后端/前端启动顺序 |
| 保留外部边界 | 后端 API、前端 dev server、业务 capability、runtime state |
| 等价证据 | 脚本存在性检查、参数/环境变量人工核查、可选本地启动 smoke |
| 当前判定 | 已完成单叶 closeout；只登记脚本入口，不改脚本行为，不继续细分 |
| 暂停点 | 任何端口、进程启动顺序、默认环境变量或用户命令变化 |

### S2 `system.entry.backend_process`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.entry` |
| 真实文件 | `src/system/entry/backend_process.rs`、`src/system/entry/mod.rs`、`src/system/mod.rs`、`src/main.rs`、`src/lib.rs`、`src/app_runtime_helpers.rs` |
| public 入口 | `run_server` |
| 兼容入口 | `quantpilot::run_server` re-export |
| 关键内部实现 | `run_api_server`、启动目录创建、CORS、安全头、JSON rejection middleware、后台观察任务、优雅关闭、关闭刷盘 |
| 保留外部边界 | `new_app_state`、`build_app_router`、handler、response schema、runtime state、executor state |
| 等价证据 | `cargo check -p quantpilot`、`cargo test -p quantpilot defaults_to_server_when_no_cli_args_are_provided`、旧 public 入口检查 |
| 当前判定 | 已抽离完成；不继续扩大到 API route owner |
| 暂停点 | 任何 AppState 工厂、router、handler、schema 或锁顺序迁移 |

### S3 `system.desktop_shell.tauri_runtime`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.desktop_shell` |
| 真实文件 | `src-tauri/src/main.rs` |
| public 入口 | Tauri main |
| 兼容入口 | 桌面应用启动入口和窗口生命周期不变 |
| 关键内部实现 | 后端 readiness wait、桌面壳初始化、Tauri command/runtime wiring |
| 保留外部边界 | Rust 后端 API、AppState、前端路由、capability 真源 |
| 等价证据 | `cargo check -p quantpilot-tauri`、`17-system.desktop_shell.tauri_runtime-readiness等价检查.md`、readiness wait 人工核查、后续桌面启动 smoke |
| 当前判定 | readiness 等价检查已完成；代码抽离仍需暂停，S3 完整 closeout 前需补桌面启动 smoke/窗口生命周期核查 |
| 暂停点 | 后端 readiness wait、窗口生命周期、Tauri command 权限或后端启动关系变化 |

### S4 `system.desktop_shell.tauri_config`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.desktop_shell` |
| 真实文件 | `src-tauri/tauri.conf.json`、`src-tauri/capabilities/default.json` |
| public 入口 | Tauri config、CSP、capabilities |
| 兼容入口 | 应用标识、窗口配置、权限声明不变 |
| 关键内部实现 | CSP、capability allowlist、窗口/打包配置 |
| 保留外部边界 | 前端 capability 投影、后端 API 权限语义、业务 supported/unsupported 声明 |
| 等价证据 | JSON parse、`cargo check -p quantpilot-tauri`、人工核查 CSP/capability diff |
| 当前判定 | 已完成单叶 closeout；实际配置变更必须单独批次 |
| 暂停点 | CSP 放宽、权限新增、窗口行为变化或 capability 声明变化 |

### S5 `system.desktop_shell.assets_schema`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.desktop_shell` |
| 真实文件 | `src-tauri/icons/*`、`src-tauri/gen/schemas/*` |
| public 入口 | 桌面图标、generated schemas |
| 兼容入口 | 资产路径和生成物消费方式不变 |
| 关键内部实现 | 图标资源、Tauri generated schema 资产 |
| 保留外部边界 | UI 设计系统、业务 schema、API response schema |
| 等价证据 | 资产路径存在性、生成物未漂移检查、打包前人工核查 |
| 当前判定 | 只需登记，不主动抽离到更细叶子 |
| 暂停点 | 重新生成 schema、替换图标体系、改打包资产路径 |

### S6 `system.build_delivery.workspace_manifest`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.build_delivery` |
| 真实文件 | `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml` |
| public 入口 | Rust workspace/package manifest |
| 兼容入口 | workspace 成员、crate 名称、package metadata 和 lockfile 语义不变 |
| 关键内部实现 | 依赖版本、feature、workspace member、crate package 配置 |
| 保留外部边界 | 业务模块 API、编译链行为、发布版本过渡决策 |
| 等价证据 | `cargo metadata`、`cargo check -p quantpilot`、`cargo check -p quantpilot-tauri`、lockfile diff 人工核查 |
| 当前判定 | 需要决策暂停；影响面大，不主动抽离 |
| 暂停点 | 依赖升级、workspace 成员变化、feature 默认值变化、lockfile 大幅漂移 |

### S7 `system.build_delivery.desktop_build_scripts`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.build_delivery` |
| 真实文件 | `src-tauri/build.rs`、`src-tauri/build.bat`、`src-tauri/dev.bat` |
| public 入口 | desktop build/dev scripts |
| 兼容入口 | Tauri build/dev 命令入口不变 |
| 关键内部实现 | build.rs 编译期逻辑、Windows desktop build/dev 批处理 |
| 保留外部边界 | 根启动脚本、CI workflow、release packaging、业务构建产物语义 |
| 等价证据 | `cargo check -p quantpilot-tauri`、脚本参数人工核查、可选 desktop dev smoke |
| 当前判定 | 可低风险登记；不和 `system.entry.launch_scripts` 混成一批 |
| 暂停点 | 构建产物路径、dev 命令、环境变量或 Tauri bundling 行为变化 |

### S8 `system.build_delivery.container_proxy`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.build_delivery` |
| 真实文件 | `Dockerfile`、`docker-compose.yml`、`nginx.conf` |
| public 入口 | 容器构建与反向代理配置 |
| 兼容入口 | container build、compose service、nginx proxy 路径不变 |
| 关键内部实现 | 镜像构建步骤、服务编排、反向代理规则 |
| 保留外部边界 | 桌面默认运行路径、后端 API handler、前端路由语义 |
| 等价证据 | Docker/compose config 人工核查、可选 `docker compose config`、proxy route 对照 |
| 当前判定 | 按变更触发；不作为当前桌面默认路径主动抽离 |
| 暂停点 | 暴露端口、代理路径、镜像构建阶段、环境变量或服务依赖变化 |

### S9 `system.build_delivery.ci_release`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.build_delivery` |
| 真实文件 | `.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`packaging/`、`release/` |
| public 入口 | CI/release workflow、release packaging 入口 |
| 兼容入口 | GitHub Actions workflow 名称、触发条件、artifact 命名和 release 路径不变 |
| 关键内部实现 | CI job、release job、scenario job、打包脚本和 release 资产 |
| 保留外部边界 | 测试资产汰换策略、业务测试语义、发布版本过渡决策 |
| 等价证据 | workflow YAML 人工核查、pre-commit 本地门禁、release dry-run 方案 |
| 当前判定 | 需要决策暂停；与测试汰换/发布流程耦合 |
| 暂停点 | workflow 触发条件、测试矩阵、artifact 名称、release 权限或发布步骤变化 |

### S10 `system.runtime_profile.config_examples`

| 项 | 基线 |
| --- | --- |
| 当前 owner | `system.runtime_profile` |
| 真实文件 | `.env.example`、`config/runtime_protocol.example.yaml`、`config/strategy_ir.v0.schema.json`、`config/strategy_ir.v0.example.json` |
| public 入口 | 环境和协议配置样例 |
| 兼容入口 | 示例文件路径、schema 文件路径和配置字段语义不变 |
| 关键内部实现 | 环境变量模板、runtime protocol 示例、strategy_ir schema/example |
| 保留外部边界 | runtime 行为真源、编译器真源、能力声明真源 |
| 等价证据 | JSON/YAML parse、schema/example 人工对照、相关 compile/runtime 测试 |
| 当前判定 | 已完成单叶 closeout；不把示例配置当 runtime 行为真源 |
| 暂停点 | schema 字段变化、默认环境变量变化、协议样例改变 runtime 支持范围 |

---

## 下一步排序

| 优先级 | 叶子 | 动作 |
| --- | --- | --- |
| P1 | S3 `system.desktop_shell.tauri_runtime` | readiness wait 等价检查已完成；完整 closeout 前补桌面启动 smoke/窗口生命周期核查 |
| P1 | S7 `system.build_delivery.desktop_build_scripts` | 单独批次登记 desktop build/dev scripts |
| P2 | S6 `system.build_delivery.workspace_manifest` | 决策暂停后再碰 |
| P2 | S8 `system.build_delivery.container_proxy` | 按容器相关变更触发 |
| P2 | S9 `system.build_delivery.ci_release` | 等测试资产汰换策略更稳定后再推进 |
| P3 | S5 `system.desktop_shell.assets_schema` | 只登记，不主动细分 |

S2 已完成，不进入下一步抽离队列。

---

## 全局暂停条件

出现以下情况时，停止继续抽离并回到方案讨论:

1. 单批次同时触碰两个以上 system 父域。
2. 候选需要改变用户启动入口、Tauri 权限、CI 触发条件、容器端口或 schema 字段语义。
3. 候选无法给出等价证据。
4. 候选把配置样例、构建脚本或 generated schema 当作 runtime 行为真源。
5. 候选需要借“整理”“重构”或“发布态优化”解释为完成。

---

## 验收标准

1. 10 个 system 叶子都有等价基线。
2. 每个叶子都标明 public 入口、关键内部实现和保留外部边界。
3. 每个叶子都有继续抽离状态和暂停点。
4. S2 已完成抽离，其他叶子不被误宣称完成。
5. 后续实际抽离可按本文件逐个叶子 closeout。
