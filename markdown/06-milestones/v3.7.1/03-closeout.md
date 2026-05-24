# v3.7.1 Closeout 基线

> 日期: 2026-05-23 | 类型: PATCH | 状态: 流程基线已落地，完整 closeout 已通过

---

## 一、结论

v3.7.1 不是功能扩张版本，而是 v3.7.0 后的回归修复与工作流收口版本。

已完成：

- S0 登录挂起修复：正确密码登录不再卡在 refresh token / JWT 生成路径。
- P1 凭证 DELETE 405 修复：Axum 0.7 路由参数语法已回到 `:service`。
- P2 测试进程文件锁修复：新增测试包装脚本，测试前停止本仓库运行进程。
- 三层门禁收口：pre-commit、CI、closeout/release 的检查项和版本口径重新对齐。
- 元流水线功能演进通道：新增能力必须先登记能力边界、回归保护矩阵、兼容性与迁移说明。
- Rust 格式基线：全仓 `cargo fmt` 已落地，`cargo fmt --check` 纳入 pre-commit、CI 和 closeout。
- executor 时间戳测试修复：OKX v5 时间戳按 RFC3339/ISO8601 格式校验，不再误按 Unix 秒解析。
- 自由维度诱错 S0 收口：清理脚本不再触碰真实运行/图版本工件，图保存改为事务式提交，密钥/JWT secret 统一先原子落盘再进入内存缓存。
- E2E 视觉回归确定性收口：视觉截图固定 API fixture 并启用未声明 API 守卫，避免 alerts/snapshots/runbook 受本地后端状态漂移。
- executor warning 债务已清零，默认预算和 CI/closeout 显式预算均为 0。

未宣称：

- 不新增 QuantScript、回测、交易、插件或 UI 平台能力。

## 二、交付清单

| 类别 | 文件 / 流程 | 状态 | 说明 |
|------|-------------|:--:|------|
| 测试包装 | `scripts/test.ps1` | ✅ | PowerShell 测试入口，默认 `cargo test --workspace` |
| 测试包装 | `scripts/test.sh` | ✅ | Git Bash / Unix 测试入口，默认 `cargo test --workspace` |
| 场景 smoke | `scripts/scenario-smoke.ps1` | ✅ | 构建后端、启动 DEV 服务、执行 QS 场景 |
| 版本一致性 | `tools/check-version-consistency.ps1` | ✅ | Cargo、Tauri、前端、lockfile 和用户可见入口统一校验 |
| warning 预算 | `tools/check-executor-warning-budget.ps1` | ✅ | 当前 executor warning budget 固定为 0 |
| 功能演进契约 | `tools/check-feature-evolution.ps1` | ✅ | 新增能力需登记和提供回归保护矩阵 |
| Pre-commit hook 同步 | `tools/check-pre-commit-hook.ps1` | ✅ | 实际安装 hook 必须与版本化 `scripts/pre-commit` 一致 |
| 清理边界门禁 | `tools/check-cleanup-boundary.ps1` | ✅ | 清理脚本仅允许旧测试工件和日志，拒绝已废弃的运行工件清理开关 |
| Rust 格式基线 | `cargo fmt --check` | ✅ | 全仓 rustfmt drift 已清理，三层门禁阻断新增漂移 |
| closeout 门禁脚本 | `tools/run-closeout-gates.bat` | ✅ | 脚本已收口为 21 项，包含 hook 同步、清理边界、Rust 格式、执行端前端构建和 QS 场景 smoke |
| CI | `.github/workflows/ci.yml` | ✅ | 接入版本一致性、功能演进、Rust 格式、测试包装、warning 预算、执行端前端构建 |
| 场景 CI | `.github/workflows/scenario-test.yml` | ✅ | 复用 `scripts/scenario-smoke.ps1` |
| Release | `.github/workflows/release.yml` | ✅ | Windows PowerShell 打包路径修复，tag 才发布 |
| 单测修正 | `src-executor/okx_rest.rs` | ✅ | OKX timestamp test 与实现的 ISO8601 格式一致 |

## 三、当前验证状态

| 门禁 | 状态 | 记录 |
|------|:--:|------|
| `tools/check-utf8.ps1` | ✅ | 545 文件 UTF-8 校验通过 |
| `tools/check-user-facing-text.ps1` | ✅ | 348 个当前用户可见入口 / 活跃规范文件通过 |
| `tools/check-capability-governance.ps1` | ✅ | 快照已更新并复验通过 |
| `tools/check-i18n.ps1` | ✅ | 未发现英文用户可见字符串 |
| `tools/check-version-consistency.ps1` | ✅ | Cargo、Tauri、前端、lockfile 和关键文档均为 3.7.1 |
| `tools/check-feature-evolution.ps1` | ✅ | v3.7.1 声明不新增功能，功能演进契约、核心回归场景和治理入口均已校验 |
| `tools/check-pre-commit-hook.ps1` | ✅ | `.git/hooks/pre-commit` 与 `scripts/pre-commit` 已同步 |
| `tools/check-cleanup-boundary.ps1` | ✅ | 临时夹具确认旧测试工件会被清理，runs/backtests/experiments/graphs/version 工件不被枚举或删除 |
| `cargo fmt --check` | ✅ | 全仓 Rust 格式基线通过；pre-commit、CI、closeout 均已接入 |
| `cargo check --workspace` | ✅ | 2026-05-24 本地通过，executor warning 债务已清零 |
| `scripts/test.ps1 test --workspace` | ✅ | workspace 测试包装器串行通过，覆盖 Windows 文件锁场景 |
| `tools/check-executor-warning-budget.ps1 -MaxWarnings 0` | ✅ | 0/0，通过；新增 warning 会阻断 |
| `frontend npm run build` | ✅ | 构建通过；保留既有 circular chunk warning |
| `frontend npm run test` | ✅ | 92 文件 / 272 测试通过 |
| `frontend npm run test:e2e` | ✅ | 21 passed / 18 skipped；视觉回归已固定 API fixture，避免本地运维状态污染截图 |
| `frontend npm audit --audit-level=moderate` | ✅ | `npm audit fix` 后 0 vulnerability |
| `frontend-executor npm run build` | ✅ | 构建通过，`dist/index.html` 已刷新到 v3.7.1 |
| `cargo test --bin executor -- --test-threads=1` | ✅ | 21/21 通过 |
| `tools/run-closeout-gates.bat` | ✅ | 2026-05-23 本地复跑 21/21 通过；覆盖 Rust workspace、前端 build/test/e2e、executor、QS scenario smoke 与全部元流水线检查 |

## 四、剩余风险

| 风险 | 级别 | 处理 |
|------|:--:|------|
| executor warning 债务 | P2 | 已清零，预算 0 阻断新增 warning |
| 新增能力若缺少回归矩阵 | P1 | `tools/check-feature-evolution.ps1` 阻断 closeout |
| 前端构建仍有 circular chunk warning | P3 | 当前不阻断，后续优化 manualChunks |
| release workflow 只做过静态修正 | P2 | 下一次发布前先用 workflow_dispatch dry-run 验证 |

## 五、下一步

1. 发布或打 tag 前重新执行完整 `tools\run-closeout-gates.bat`，覆盖 E2E、workspace tests 和 QS scenario smoke。
2. 用 workflow_dispatch dry-run 验证 release workflow 的 Windows 打包路径。
3. 后续优化进入 V1 收口，不在 v3.7.1 内继续扩大功能范围。
