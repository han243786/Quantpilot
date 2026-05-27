# v4.16.0 system 大模块分层统计

> 职责: 在进入 `system` 抽离前，先确定 `root.system` 大约分多少层、多少叶子模块。
> 统计口径: 以现有 `system.entry`、全量树根1、`src-tauri/`、启动脚本、workspace 构建/配置文件为依据。
> 当前结论: `system` 建议分 3 层，10 个叶子模块。

---

## 分层结论

| 层级 | 名称 | 数量 | 说明 |
| --- | --- | ---: | --- |
| L0 | `root.system` | 1 | 系统级父模块，负责启动、进程拓扑、桌面壳、构建交付和运行配置 |
| L1 | system 二级域 | 4 | `entry`、`desktop_shell`、`build_delivery`、`runtime_profile` |
| L2 | system 叶子模块 | 10 | 每个叶子都能指向真实文件、public 入口或配置资产 |

本阶段不建议把 `system` 拆到 L3。Tauri icons、generated schemas、CI workflow 等资产先作为 L2 叶子内部文件管理，避免抽离一开始就过度碎片化。

---

## system 二级域

| 二级域 | 职责 | 叶子数 | 当前状态 |
| --- | --- | ---: | --- |
| `system.entry` | 本地启动、后端进程、开发进程拓扑 | 2 | 已有种子节点，需扩展 |
| `system.desktop_shell` | Tauri 桌面壳、窗口配置、CSP、图标和 capability schema | 3 | 需补白箱 |
| `system.build_delivery` | Rust workspace、桌面构建脚本、容器/代理、CI/release | 4 | 需补白箱 |
| `system.runtime_profile` | 环境变量模板、运行协议样例、strategy_ir schema/example | 1 | 需补白箱 |

---

## system 叶子模块

| # | 叶子模块 | 父域 | 真实文件 | 关键 public/入口 | 抽离备注 |
| --- | --- | --- | --- | --- | --- |
| S1 | `system.entry.launch_scripts` | `system.entry` | `start.bat`、`start.ps1` | 启动脚本入口 | 只编排，不拥有业务能力真源 |
| S2 | `system.entry.backend_process` | `system.entry` | `src/main.rs`、`src/lib.rs`、`src/app_runtime_helpers.rs`、`src/system/entry/backend_process.rs` | `run_server`、`new_app_state` | `run_server` 已试水抽离；不绕过 `build_app_router` |
| S3 | `system.desktop_shell.tauri_runtime` | `system.desktop_shell` | `src-tauri/src/main.rs` | Tauri main、后端 readiness wait | 不拥有后端状态 |
| S4 | `system.desktop_shell.tauri_config` | `system.desktop_shell` | `src-tauri/tauri.conf.json`、`src-tauri/capabilities/default.json` | Tauri config、CSP、capabilities | 改 CSP/窗口/权限时必须单独登记 |
| S5 | `system.desktop_shell.assets_schema` | `system.desktop_shell` | `src-tauri/icons/*`、`src-tauri/gen/schemas/*` | 桌面图标、generated schemas | 先作为资产叶子，不拆更细 |
| S6 | `system.build_delivery.workspace_manifest` | `system.build_delivery` | `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml` | Rust workspace/package manifest | 改依赖或 workspace 成员时登记 |
| S7 | `system.build_delivery.desktop_build_scripts` | `system.build_delivery` | `src-tauri/build.rs`、`src-tauri/build.bat`、`src-tauri/dev.bat` | desktop build/dev scripts | 不和启动脚本混成一批 |
| S8 | `system.build_delivery.container_proxy` | `system.build_delivery` | `Dockerfile`、`docker-compose.yml`、`nginx.conf` | 容器构建与反向代理 | 不作为当前桌面默认运行路径 |
| S9 | `system.build_delivery.ci_release` | `system.build_delivery` | `.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`packaging/`、`release/` | CI/release workflow | 只管交付流水线，不改业务测试语义 |
| S10 | `system.runtime_profile.config_examples` | `system.runtime_profile` | `.env.example`、`config/runtime_protocol.example.yaml`、`config/strategy_ir.v0.schema.json`、`config/strategy_ir.v0.example.json` | 环境和协议配置样例 | 不作为 runtime 行为真源 |

---

## 抽离顺序建议

| 顺序 | 叶子模块 | 理由 |
| --- | --- | --- |
| 1 | `system.entry.launch_scripts` | 风险低，能先把启动编排从业务能力中剥离 |
| 2 | `system.entry.backend_process` | 与 BE-001 后端接口边界相邻，但不进入 handler |
| 3 | `system.desktop_shell.tauri_runtime` | Tauri readiness wait 与桌面壳边界清晰 |
| 4 | `system.desktop_shell.tauri_config` | CSP/窗口/权限独立，适合单独登记 |
| 5 | `system.build_delivery.workspace_manifest` | 影响面大，必须晚于入口边界 |
| 6 | `system.build_delivery.ci_release` | 与测试汰换有关，需等测试资产策略更稳定 |

其余叶子可按实际变更触发，不必主动抽离。

---

## 与 BE-001 的关系

`system.entry.backend_process` 调用 `run_server`，而 `run_server` 依赖 `build_app_router`。因此 `system` 抽离不能绕过 BE-001:

```text
system.entry.backend_process
  -> run_server
  -> backend.interface_boundary
  -> build_app_router
```

`system` 只拥有启动和进程编排，不拥有 API route owner、runtime state、executor state 或 capability 真源。

---

## 覆盖缺口

| 缺口 | 处理 |
| --- | --- |
| 当前模块树只有 `system.entry` 种子节点 | 后续 system 抽离前，按本文件补 10 个叶子白箱 |
| `system.build_delivery` 跨 CI、容器、release | 不作为 v4.16 首批，避免和测试资产汰换纠缠 |
| `src-tauri/gen/schemas/*` 是生成资产 | 先作为资产叶子登记，不单独抽离 |
| `system.entry.backend_process` 已开始试水 | 仅完成 `run_server` 所属边界迁移；`run_api_server`、`new_app_state`、`build_app_router` 仍保留原处 |

---

## 验收标准

1. `system` 分层确定为 3 层。
2. `system` 叶子模块确定为 10 个。
3. 每个叶子模块都有真实文件和边界说明。
4. `system` 与 BE-001 的父子关系明确。
5. 不把启动编排误判为业务能力真源。
