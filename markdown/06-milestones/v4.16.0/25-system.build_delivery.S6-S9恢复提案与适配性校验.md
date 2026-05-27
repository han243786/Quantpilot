# v4.16.0 system.build_delivery S6/S9 恢复提案与适配性校验

> 版本类型: MINOR architecture / governance。
> 基准: `23-system.build_delivery.S6-S9暂停决策记录.md`、开发者最新指令“继续动 S6/S9”。
> 执行档位: 重型。
> 判定: 开发者已明确解除 S6/S9 暂停；本批次只做文档级白箱 closeout 和等价登记，不改 manifest、lockfile、workflow、packaging 或 release 语义。

门禁标记: `S6/S9 resume proposal passed`。
流程标记: `proposal compatibility optimization continue design`。
编辑标记: `no real manifest workflow edits`。

---

## 目标

本文件按暂停恢复协议，完成 S6 `system.build_delivery.workspace_manifest` 和 S9 `system.build_delivery.ci_release` 的恢复提案、适配性校验、方案优化和落地设计。

恢复范围只包含:

1. S6 workspace/package manifest 的白箱边界和等价证据。
2. S9 CI/release workflow、packaging 和 release manifest 的白箱边界和等价证据。
3. 更新 `root.system` 顶层阶段性 closeout，使 system 10 叶全部进入当前抽离控制面。

本批次不做依赖升级、lockfile 改写、workflow 触发条件调整、测试矩阵改造、release 权限变更、artifact 名称变更或 Docker runtime smoke。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | 暂停恢复协议、提案状态机、S6/S9 closeout 准入 | 落地 |
| 规范矩阵 | workspace manifest、CI/release、测试汰换隔离、发布过渡保护 | 加固 |
| 引导矩阵 | 全量树、模块树、S6/S9 真实文件和门禁坐标 | 扩展 |
| 模块树 | `system.build_delivery.workspace_manifest`、`system.build_delivery.ci_release` | 从暂停登记转入 closeout |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.build_delivery.workspace_manifest`、`system.build_delivery.ci_release` |
| 真实文件 | `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml`、`.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`packaging/windows/installer.nsi`、`release/release-manifest.yaml` |
| public 方法 | Rust workspace/package manifest、GitHub Actions workflow、NSIS installer script、release manifest |
| 测试/门禁 | `cargo metadata --format-version 1 --no-deps`、`cargo check --workspace`、target file diff clean、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 提案流程执行

| 步骤 | 结论 |
| --- | --- |
| 1 提案 | 对 S6/S9 做文档级 closeout，不改真实 manifest/workflow/release 文件 |
| 2 适配性校验 | 不触发依赖升级、lockfile 漂移、workflow 权限变更、测试资产删除或发布过渡 |
| 3 方案优化 | 拆成 `26-system.build_delivery.workspace_manifest单叶closeout.md` 和 `27-system.build_delivery.ci_release单叶closeout.md`，避免把 manifest 与 release 混成一批 |
| 4 回到 2 或继续 | 优化后再次校验: 两个 closeout 都是静态白箱收束，证据来自读取、metadata、check 和 diff clean |
| 5 落下设计方案 | 先写恢复提案，再分别写 S6/S9 closeout，最后更新 system 顶层和门禁 |

---

## 适配性校验

| 风险 | 校验结论 | 处理 |
| --- | --- | --- |
| 依赖升级或 workspace 成员变化 | 不发生 | 不修改 `Cargo.toml`、`Cargo.lock`、`src-tauri/Cargo.toml` |
| lockfile 大幅漂移 | 不发生 | 不运行会写 lockfile 的命令，不提交 lockfile diff |
| workflow 触发条件变化 | 不发生 | 不修改 `.github/workflows/*.yml` |
| 测试资产汰换 | 不发生 | 不删除 E2E、scenario 或旧测试程序 |
| release 权限/artifact 改动 | 不发生 | 不修改 `release/`、`packaging/` 或 release workflow |
| 发布版本过渡 | 不发生 | 不主动提出横向连接、性能边或 Docker runtime smoke |

---

## 落地设计

| 批次 | 文件 | 目标 |
| --- | --- | --- |
| S6 closeout | `26-system.build_delivery.workspace_manifest单叶closeout.md` | 登记 Cargo workspace、package manifest、feature、lockfile 边界和 cargo metadata/check 证据 |
| S9 closeout | `27-system.build_delivery.ci_release单叶closeout.md` | 登记 CI、release、scenario、packaging、release manifest 边界和 workflow review 证据 |
| system 刷新 | `24-system顶层阶段性closeout.md` | 从“8 叶完成 + S6/S9 暂停”刷新为“10 叶 closeout 完成，但整理/重构仍未启动” |

---

## 禁止事项

- 不改真实 manifest、lockfile、workflow、packaging 或 release 文件。
- 不把本批次解释为依赖升级或发布验收。
- 不把 S9 closeout 当作测试资产汰换完成。
- 不触发 Docker runtime smoke。
- 不主动提出发布版本过渡或横向连接。

---

## 验收标准

1. 恢复提案进入里程碑索引、全量树、模块树和治理门禁。
2. S6/S9 closeout 分文件落地，不混成一个大批次。
3. `git diff -- Cargo.toml Cargo.lock src-tauri/Cargo.toml .github/workflows/ci.yml .github/workflows/release.yml .github/workflows/scenario-test.yml packaging/windows/installer.nsi release/release-manifest.yaml` 保持 clean。
4. 治理门禁能发现本恢复提案缺失。
