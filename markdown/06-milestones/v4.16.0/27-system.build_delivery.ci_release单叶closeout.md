# v4.16.0 system.build_delivery.ci_release 单叶 closeout

> 版本类型: MINOR architecture / governance。
> 基准: `12-system十叶模块等价基线.md`、`25-system.build_delivery.S6-S9恢复提案与适配性校验.md`。
> 执行档位: 重型。
> 判定: S9 `system.build_delivery.ci_release` 完成文档级白箱 closeout；CI/release/scenario/packaging/release manifest 边界已登记，不改 workflow、测试矩阵、artifact、release 权限或 packaging 语义。

门禁标记: `S9 ci release closeout complete`。
发布边界标记: `not release approval`。

---

## 目标

本文件确认 S9 的当前真实边界:

1. `.github/workflows/ci.yml` 管理常规 CI quality/build/test/audit/check-clean-worktree。
2. `.github/workflows/release.yml` 管理手动 dry-run、tag release、Windows package archive 和 GitHub Release 发布路径。
3. `.github/workflows/scenario-test.yml` 管理 scenario smoke 的 push/PR/manual/schedule 入口。
4. `packaging/windows/installer.nsi` 管理 Windows NSIS installer 脚本。
5. `release/release-manifest.yaml` 管理 release metadata、target、artifact、runtime 和 security 声明。

本批次只做 closeout，不改变 CI 或发布语义。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | v4.16 system 单叶 closeout、S9 完成判定 | 落地 |
| 规范矩阵 | CI/release owner、测试汰换隔离、release dry-run 边界 | 加固 |
| 引导矩阵 | 全量树、模块树、真实文件、workflow/release 门禁坐标 | 扩展 |
| 模块树 | `system.build_delivery.ci_release` | 完成 S9 基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根1.3、根7.6 |
| 模块树节点 | `system.build_delivery.ci_release` |
| 真实文件 | `.github/workflows/ci.yml`、`.github/workflows/release.yml`、`.github/workflows/scenario-test.yml`、`packaging/windows/installer.nsi`、`release/release-manifest.yaml` |
| public 方法 | GitHub Actions CI workflow、release workflow、scenario workflow、NSIS installer script、release manifest |
| 关键内部实现 | CI quality gates、Rust workspace check/test、frontend build/test/e2e/audit、release dry-run input、tag release publish、artifact upload、scenario smoke、Windows installer |
| 测试/门禁 | workflow static review、packaging/release file existence、release dry-run plan、target file diff clean、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 当前白箱基线

| 文件 | 当前事实 |
| --- | --- |
| `.github/workflows/ci.yml` | workflow name `CI`; triggers push to `main`/`master`/`codex/**` and pull_request; job `quality-and-build` runs on `windows-latest` |
| `.github/workflows/release.yml` | workflow name `package-and-release`; triggers workflow_dispatch with `dry_run` default true and push tag `v*`; `permissions.contents = write`; job `release` runs on `windows-latest` |
| `.github/workflows/scenario-test.yml` | workflow name `Scenario Test`; triggers push main/master, pull_request, workflow_dispatch and scheduled cron; job `scenarios` runs on `windows-latest` |
| `packaging/windows/installer.nsi` | NSIS installer writes to local app data, creates desktop shortcut and uninstall entry |
| `release/release-manifest.yaml` | release app name `quantpilot`, package id `com.quantpilot.app`, version `4.7.0`, targets Windows NSIS and Linux Docker |

---

## 等价验证证据

| 核查项 | 结果 | 证据 |
| --- | :--: | --- |
| workflow files exist | 通过 | `ci.yml`、`release.yml`、`scenario-test.yml` 均为 tracked file |
| packaging/release files exist | 通过 | `packaging/windows/installer.nsi`、`release/release-manifest.yaml` 均为 tracked file |
| workflow static review | 通过 | 当前触发条件、job 名、runs-on、关键 step 已登记 |
| target diff | 通过 | `git diff -- .github/workflows/ci.yml .github/workflows/release.yml .github/workflows/scenario-test.yml packaging/windows/installer.nsi release/release-manifest.yaml` 无输出 |
| 外部边界 | 通过 | 不改测试矩阵、release 权限、artifact、packaging、Docker runtime smoke 或发布过渡 |

---

## 白箱 closeout 判定

| 项 | 判定 | 说明 |
| --- | --- | --- |
| public 入口 | 完成 | CI/release/scenario workflow、NSIS script、release manifest 已登记 |
| 兼容入口 | 完成 | workflow 名、触发条件、job 名和 artifact 路径不变 |
| release dry-run 边界 | 完成 | release workflow 保留 `dry_run` input，但本批次不执行发布验收 |
| 外部边界 | 完成 | 不拥有测试资产汰换、业务测试语义、Docker runtime smoke 或发布版本过渡 |
| 等价证据 | 完成 | static review 与 target diff clean 已具备 |
| 继续细分 | 停止 | CI/release 文件是交付控制面；拆成 step 级叶子会制造过细文档碎片 |

---

## 父子通信规则

`system.build_delivery.ci_release` 只能经 `system.build_delivery` 管理 CI/release 交付边界。它不得直接改变测试资产汰换策略、业务测试语义、发布版本过渡、Docker runtime smoke 或运行时能力声明。

后续如果改 workflow 触发条件、测试矩阵、artifact 名称、release 权限、packaging 脚本或 release manifest，必须重新打开 S9，并给出 dry-run 方案、回退点和风险窗口。

---

## 禁止事项

- 不把 S9 closeout 解释为发布验收完成。
- 不改 `.github/workflows/*.yml`、`packaging/` 或 `release/`。
- 不借 S9 closeout 删除、替换或静默跳过旧测试程序。
- 不把 release dry-run input 当作已经执行 release dry-run。
- 不主动提出发布版本过渡或 Docker runtime smoke。

---

## 验收标准

1. S9 的真实文件、public 入口、关键内部实现和父级通信规则已登记。
2. workflow、packaging 和 release manifest target diff clean。
3. 文档明确 S9 closeout 不等于发布验收。
4. `system.build_delivery.ci_release` 模块树节点标记为单叶 closeout 完成。
5. 本批次不改 workflow、packaging、release manifest、测试矩阵或 release 权限。
