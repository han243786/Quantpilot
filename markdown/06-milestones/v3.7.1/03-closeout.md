# v3.7.1 Closeout 基线

> 日期: 2026-05-22 | 类型: PATCH | 状态: 流程基线已落地，完整 closeout 绿灯待执行

---

## 一、结论

v3.7.1 不是功能扩张版本，而是 v3.7.0 后的回归修复与工作流收口版本。

已完成：

- S0 登录挂起修复：正确密码登录不再卡在 refresh token / JWT 生成路径。
- P1 凭证 DELETE 405 修复：Axum 0.7 路由参数语法已回到 `:service`。
- P2 测试进程文件锁修复：新增测试包装脚本，测试前停止本仓库运行进程。
- 三层门禁收口：pre-commit、CI、closeout/release 的检查项和版本口径重新对齐。
- executor 时间戳测试修复：OKX v5 时间戳按 RFC3339/ISO8601 格式校验，不再误按 Unix 秒解析。

未宣称：

- 不宣称完整 17 项 closeout 已全部通过。
- 不宣称 executor warning 债务已清零。
- 不新增 QuantScript、回测、交易、插件或 UI 平台能力。

## 二、交付清单

| 类别 | 文件 / 流程 | 状态 | 说明 |
|------|-------------|:--:|------|
| 测试包装 | `scripts/test.ps1` | ✅ | PowerShell 测试入口，默认 `cargo test --workspace` |
| 测试包装 | `scripts/test.sh` | ✅ | Git Bash / Unix 测试入口，默认 `cargo test --workspace` |
| 场景 smoke | `scripts/scenario-smoke.ps1` | ✅ | 构建后端、启动 DEV 服务、执行 QS 场景 |
| 版本一致性 | `tools/check-version-consistency.ps1` | ✅ | Cargo、Tauri、前端、lockfile 和用户可见入口统一校验 |
| warning 预算 | `tools/check-executor-warning-budget.ps1` | ✅ | 当前 executor warning budget 固定为 49 |
| closeout 门禁 | `tools/run-closeout-gates.bat` | ✅ | 收口为 17 项，包含执行端前端构建和 QS 场景 smoke |
| CI | `.github/workflows/ci.yml` | ✅ | 接入版本一致性、测试包装、warning 预算、执行端前端构建 |
| 场景 CI | `.github/workflows/scenario-test.yml` | ✅ | 复用 `scripts/scenario-smoke.ps1` |
| Release | `.github/workflows/release.yml` | ✅ | Windows PowerShell 打包路径修复，tag 才发布 |
| 单测修正 | `src-executor/okx_rest.rs` | ✅ | OKX timestamp test 与实现的 ISO8601 格式一致 |

## 三、当前验证状态

| 门禁 | 状态 | 记录 |
|------|:--:|------|
| `tools/check-utf8.ps1` | ✅ | 534 文件 UTF-8 校验通过 |
| `tools/check-user-facing-text.ps1` | ✅ | 336 个当前用户可见入口 / 活跃规范文件通过 |
| `tools/check-capability-governance.ps1` | ✅ | 快照已更新并复验通过 |
| `tools/check-i18n.ps1` | ✅ | 未发现英文用户可见字符串 |
| `tools/check-version-consistency.ps1` | ✅ | Cargo、Tauri、前端、lockfile 和关键文档均为 3.7.1 |
| `cargo check --workspace` | ✅ | 2026-05-22 本地通过，仍有 executor warning 债务 |
| `scripts/test.ps1 test --workspace --no-run` | ✅ | 测试编译包装器串行通过，覆盖 Windows 文件锁场景 |
| `tools/check-executor-warning-budget.ps1 -MaxWarnings 49` | ✅ | 49/49，通过；新增 warning 会阻断 |
| `frontend npm run build` | ✅ | 构建通过；保留既有 circular chunk warning |
| `frontend npm run test` | ✅ | 92 文件 / 269 测试通过 |
| `frontend npm audit --audit-level=moderate` | ✅ | `npm audit fix` 后 0 vulnerability |
| `frontend-executor npm run build` | ✅ | 构建通过，`dist/index.html` 已刷新到 v3.7.1 |
| `scripts/test.ps1 test --bin executor` | ✅ | 9/9 通过 |
| `tools/run-closeout-gates.bat` | 未执行 | 发布前必须完整跑通 17 项 |

## 四、剩余风险

| 风险 | 级别 | 处理 |
|------|:--:|------|
| executor warning 债务仍为 49 | P2 | 预算化阻断新增，后续单独清债 |
| 完整 closeout 尚未跑通 | P1 | 发布或打 tag 前必须执行 `tools\run-closeout-gates.bat` |
| 前端构建仍有 circular chunk warning | P3 | 当前不阻断，后续优化 manualChunks |
| release workflow 只做过静态修正 | P2 | 下一次发布前先用 workflow_dispatch dry-run 验证 |

## 五、下一步

1. 在发布前执行完整 `tools\run-closeout-gates.bat`，覆盖 E2E、workspace tests 和 QS scenario smoke。
2. 用 workflow_dispatch dry-run 验证 release workflow 的 Windows 打包路径。
3. 后续优化进入 V1 收口，不在 v3.7.1 内继续扩大功能范围。
