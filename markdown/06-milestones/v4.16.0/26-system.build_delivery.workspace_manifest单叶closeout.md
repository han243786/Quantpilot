# v4.16.0 system.build_delivery.workspace_manifest 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`25-system.build_delivery.S6-S9恢复提案与适配性校验.md`。
> 执行档位: 重型。
> 判定: S6 `system.build_delivery.workspace_manifest` 完成文档级白箱 closeout；Cargo workspace/package manifest 和 lockfile 边界已登记，不改依赖、成员、feature 或 lockfile。

门禁标记: `S6 workspace manifest closeout complete`。

---

## 目标

本文件确认 S6 的当前真实边界:

1. 根 `Cargo.toml` 管理 workspace members、resolver、workspace dependencies、`quantpilot` package、features、dependencies 和 `executor` bin。
2. `src-tauri/Cargo.toml` 管理 `quantpilot-tauri` package、Tauri build dependency、runtime dependency 和 default feature。
3. `Cargo.lock` 固定当前 dependency resolution。
4. 本批次只做 closeout，不改变 manifest 语义。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S6 完成判定 | 落地 |
| 规范矩阵 | workspace manifest owner、lockfile 边界、依赖变更暂停条件 | 加固 |
| 引导矩阵 | 全量树、模块树、真实文件、cargo 门禁坐标 | 扩展 |
| 模块树 | `system.build_delivery.workspace_manifest` | 完成 S6 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.build_delivery.workspace_manifest` |
| 真实文件 | `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml` |
| public 方法 | Rust workspace manifest、root package manifest、Tauri package manifest、lockfile resolution |
| 关键内部实现 | workspace members、resolver 2、workspace dependency `sha2`、`quantpilot` dependencies/features、`executor` bin、`quantpilot-tauri` default `custom-protocol` feature |
| 测试/门禁 | `cargo metadata --format-version 1 --no-deps`、`cargo check --workspace`、manifest/lockfile diff clean、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 当前白箱基线

| 项 | 当前事实 |
| --- | --- |
| workspace members | `qrpc_core`、`qrpc_core_ir`、`qrpc_compiler`、`qrpc_runtime`、`qrpc_session`、`quantscript`、`src-tauri` |
| workspace resolver | `resolver = "2"` |
| workspace dependency | `sha2 = "0.10"` |
| root package | `quantpilot` v4.7.0, edition 2021 |
| root feature | `dev_tools = []` |
| root binary | `executor` -> `src-executor/main.rs` |
| Tauri package | `quantpilot-tauri` v4.7.0, edition 2021 |
| Tauri build dependency | `tauri-build = "2"` |
| Tauri default feature | `default = ["custom-protocol"]` |
| lockfile | `Cargo.lock` tracked and unchanged in this batch |

---

## 等价验证证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| workspace metadata | 通过 | `cargo metadata --format-version 1 --no-deps` 成功 |
| workspace compile graph | 通过 | `cargo check --workspace` 成功 |
| manifest diff | 通过 | `git diff -- Cargo.toml Cargo.lock src-tauri/Cargo.toml` 无输出 |
| package versions | 通过 | `quantpilot` 与 `quantpilot-tauri` 均为 v4.7.0 |
| 外部边界 | 通过 | 不改 dependencies、features、workspace members、lockfile 或 release workflow |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | root workspace/package manifest、Tauri manifest 和 lockfile 已登记 |
| 兼容入口 | 完成 | workspace 成员、package 名、crate 名和 feature 不变 |
| 编译图 | 完成 | cargo metadata 与 workspace check 通过 |
| 外部边界 | 完成 | 不拥有后端 API、Tauri runtime、CI/release、发布过渡或业务能力 |
| 等价证据 | 完成 | metadata/check/diff clean 已具备 |
| 继续细分 | 停止 | manifest 字段之间高度耦合，继续拆成字段级叶子没有独立 owner |

---

## 父子通信规则

`system.build_delivery.workspace_manifest` 只能经 `system.build_delivery` 管理编译图、依赖和 lockfile 边界。它不得直接改变后端 API、Tauri runtime、CI/release workflow、发布版本过渡或业务模块行为。

后续如果要改依赖、workspace 成员、feature 默认值、package metadata 或 lockfile，必须重新打开 S6，并给出 dependency diff、编译证据和回退点。

---

## 禁止事项

- 不把 S6 closeout 解释为依赖升级。
- 不改 `Cargo.toml`、`Cargo.lock` 或 `src-tauri/Cargo.toml`。
- 不把 lockfile 变化混入无关批次。
- 不借 manifest closeout 改 workflow、release 或 Docker。
- 不主动提出发布版本过渡。

---

## 验收标准

1. S6 的真实文件、public 入口、关键内部实现和父级通信规则已登记。
2. `cargo metadata --format-version 1 --no-deps` 与 `cargo check --workspace` 已通过。
3. manifest/lockfile diff clean。
4. `system.build_delivery.workspace_manifest` 模块树节点标记为单叶 closeout 完成。
5. 本批次不改 manifest、lockfile、依赖、feature 或 workspace 成员。
