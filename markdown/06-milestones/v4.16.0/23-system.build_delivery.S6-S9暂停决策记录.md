# v4.16.0 system.build_delivery S6/S9 暂停决策记录

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、开发者最新决策。
> 执行档位: 重型。
> 判定: S6 `system.build_delivery.workspace_manifest` 与 S9 `system.build_delivery.ci_release` 暂停已采纳；后续已由开发者指令“继续动 S6/S9”解除，并在 `25-system.build_delivery.S6-S9恢复提案与适配性校验.md`、`26-system.build_delivery.workspace_manifest单叶closeout.md`、`27-system.build_delivery.ci_release单叶closeout.md` 中完成文档级 closeout。

---

## 目标

本文件只记录暂停决策的历史基线，防止后续把暂停期误判为已经完成。当前最新状态以 25-27 为准。

暂停原因:

1. S6 牵涉 workspace 成员、依赖版本、feature、crate metadata 和 lockfile，大概率会扩大到编译链与发布语义。
2. S9 牵涉 CI、release、scenario test、packaging 和测试资产汰换策略，不能在测试程序大规模汰换前顺手收口。
3. 两者都可能触发发布版本过渡讨论，而 AI 不允许主动提出发布版本过渡。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 决策暂停、单叶队列调整、R4/R6 完成条件 | 落地 |
| 规范矩阵 | workspace manifest、CI/release、发布过渡保护、测试汰换隔离 | 加固 |
| 引导矩阵 | 全量树、模块树、S6/S9 暂停坐标 | 扩展 |
| 模块树 | `system.build_delivery.workspace_manifest`、`system.build_delivery.ci_release` | 暂停登记，不完成 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.build_delivery.workspace_manifest`、`system.build_delivery.ci_release` |
| S6 真实文件 | `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml` |
| S9 真实文件 | `.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`packaging/windows/installer.nsi`、`release/release-manifest.yaml` |
| public 方法 | Rust workspace/package manifest、GitHub Actions workflow、release packaging entry |
| 测试/门禁 | `cargo metadata --format-version 1 --no-deps`、`cargo check -p quantpilot`、`cargo check -p quantpilot-tauri`、workflow YAML review、release dry-run 方案 |

---

## 暂停判定

门禁标记: `S6/S9 pause is not closeout`。

恢复标记: `S6/S9 pause superseded by closeout docs 25-27`。

| 叶子 | 暂停状态 | 暂停原因 | 解除条件 |
| --- | --- | --- | --- |
| S6 `system.build_delivery.workspace_manifest` | 暂停已采纳 | 依赖、workspace、feature 和 lockfile 影响面过大 | 开发者明确要求处理 manifest，且先给出依赖/成员/feature 变更范围 |
| S9 `system.build_delivery.ci_release` | 暂停已采纳 | 与测试资产汰换、release 权限、artifact 和 packaging 耦合 | 开发者明确要求处理 CI/release，且先确认测试汰换策略与 release dry-run 方式 |

---

## 当前可保留证据

| 证据 | 结果 |
| --- | --- |
| workspace manifest 可读取 | `cargo metadata --format-version 1 --no-deps` 成功 |
| S6 文件存在 | `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml` 均为 tracked file |
| S9 文件存在 | `.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`packaging/windows/installer.nsi`、`release/release-manifest.yaml` 均为 tracked file |
| 语义变更 | 本批次没有修改 manifest、lockfile、workflow、packaging 或 release 文件 |

这些证据只能说明“当前边界可定位”，不能说明 S6/S9 已 closeout。

---

## 禁止事项

- 不把 S6/S9 暂停写成完成。
- 不改 `Cargo.toml`、`Cargo.lock` 或 `src-tauri/Cargo.toml`。
- 不改 `.github/workflows/*.yml`、`packaging/` 或 `release/`。
- 不借 S9 closeout 删除、替换或静默跳过旧测试程序。
- 不主动提出发布版本过渡、横向连接或性能优化连接。
- 不把 `cargo metadata` 成功解释为依赖升级已验证。

---

## 后续恢复协议

恢复 S6 或 S9 时，必须重新走提案流程:

1. 提案: 明确具体 leaf、真实文件、public 入口、要改的字段和不改的字段。
2. 适配性校验: 判断是否触发发布过渡、测试汰换、lockfile 大漂移或 workflow 权限变化。
3. 方案优化: 至少再打磨一次，给出回退点和等价证据。
4. 回到适配性校验或继续。
5. 落下设计方案后再实现。

---

## 验收标准

1. S6/S9 暂停状态进入里程碑索引、落地记录、十叶基线、递归流程和模块树。
2. 治理门禁能发现本暂停记录缺失。
3. 文档明确 S6/S9 不算 closeout 完成。
4. 本批次没有修改 manifest、lockfile、workflow、packaging 或 release 语义。
